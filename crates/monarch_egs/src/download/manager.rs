use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use reqwest::Client;
use sha1::{Digest, Sha1};
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::sync::{Notify, mpsc};
use tokio::task::JoinSet;
use tracing::{error, info};

use super::chunk::process_chunk;
use super::{FileManifest, Manifest};
use crate::utils::err::MonarchEgsError;

/// Tuning knobs for a download run.
#[derive(Debug, Clone, Copy)]
pub struct DownloaderOptions {
    /// Number of chunks downloaded in parallel. Defaults to 8.
    pub max_workers: usize,
    /// Maximum number of decompressed chunks kept in memory at once. Each
    /// chunk is at most ~1 MiB. Defaults to 128 (~128 MiB).
    pub max_cached_chunks: usize,
}

impl Default for DownloaderOptions {
    fn default() -> Self {
        Self {
            max_workers: 8,
            max_cached_chunks: 128,
        }
    }
}

/// Shared download speed limit in bytes per second; 0 means unlimited.
pub type SpeedLimit = Arc<AtomicU64>;

/// Token-bucket rate limiter shared by all download workers. Workers reserve
/// bandwidth before fetching a chunk and sleep until their reservation is
/// covered by the refill budget. The bucket starts empty and banks at most
/// [`MAX_BURST_SECS`] worth of budget, so neither run start nor idle periods
/// can produce a full-speed burst. The limit can be changed at any time
/// through the shared [`SpeedLimit`] atomic; 0 disables limiting entirely.
struct SpeedLimiter {
    limit: SpeedLimit,
    state: Mutex<LimiterState>,
}

/// How much bandwidth credit the bucket may bank relative to the limit.
/// Kept small so bursts stay imperceptible next to the configured speed.
const MAX_BURST_SECS: f64 = 0.25;

struct LimiterState {
    last_refill: Instant,
    /// Refilled bandwidth credit in bytes, capped at [`MAX_BURST_SECS`]
    /// seconds' worth so idle periods never accumulate a large burst.
    allowance: f64,
}

impl SpeedLimiter {
    fn new(limit: SpeedLimit) -> Self {
        Self {
            limit,
            state: Mutex::new(LimiterState {
                last_refill: Instant::now(),
                // Start empty: no free full-speed burst at run start.
                allowance: 0.0,
            }),
        }
    }

    /// Reserve bandwidth for `bytes`. Returns [`Duration::ZERO`] when the
    /// request is granted (credit is then deducted in full); otherwise
    /// returns how long to wait before retrying, leaving the bucket
    /// untouched. Callers must retry via [`SpeedLimiter::consume`] — granting
    /// partial credit up front would let concurrent callers borrow against
    /// the same future bandwidth many times over and massively exceed the
    /// limit.
    fn reserve(&self, bytes: u64) -> Duration {
        let max = self.limit.load(Ordering::Relaxed);
        if max == 0 || bytes == 0 {
            return Duration::ZERO;
        }

        let mut state = self.state.lock().unwrap();
        let now = Instant::now();
        let elapsed = now.duration_since(state.last_refill).as_secs_f64();
        let burst_cap = max as f64 * MAX_BURST_SECS;
        state.allowance = (state.allowance + elapsed * max as f64).min(burst_cap);
        state.last_refill = now;

        let bytes = bytes as f64;
        if state.allowance >= bytes {
            state.allowance -= bytes;
            return Duration::ZERO;
        }
        Duration::from_secs_f64((bytes - state.allowance) / max as f64)
    }

    /// Reserve bandwidth and wait until the reservation is covered.
    async fn consume(&self, bytes: u64) {
        loop {
            let wait = self.reserve(bytes);
            if wait.is_zero() {
                return;
            }
            tokio::time::sleep(wait).await;
        }
    }
}

/// A snapshot of download progress. All byte counters are cumulative.
#[derive(Debug, Clone, Default)]
pub struct DownloadProgress {
    pub downloaded_bytes: u64,
    pub decompressed_bytes: u64,
    pub total_download_bytes: u64,
    pub chunks_completed: u64,
    pub total_chunks: u64,
    pub files_completed: u64,
    pub total_files: u64,
    /// Bytes per second, estimated between snapshots.
    pub download_speed_bps: f64,
}

impl DownloadProgress {
    /// Overall completion percentage in the 0.0..1.0 range.
    pub fn fraction(&self) -> f64 {
        if self.total_files == 0 {
            return 1.0;
        }
        (self.files_completed as f64 / self.total_files as f64).min(1.0)
    }
}

/// Fine-grained events emitted during a download run.
#[derive(Debug, Clone)]
pub enum DownloadEvent {
    Progress(DownloadProgress),
    ChunkDownloaded {
        guid: u128,
        downloaded: u64,
        decompressed: u64,
    },
    FileWritten {
        filename: String,
        bytes: u64,
    },
    FileSkipped {
        filename: String,
    },
    FileFailed {
        filename: String,
        error: String,
    },
}

/// Outcome of a background file finalization (rename + permission bits): the
/// file's name plus its result (bytes written on success).
type FinalizeResult = (String, Result<u64, MonarchEgsError>);

/// Summary of what a download run did.
#[derive(Debug, Clone, Default)]
pub struct DownloadReport {
    pub files_written: u64,
    pub files_skipped: u64,
    pub chunks_downloaded: u64,
    pub downloaded_bytes: u64,
    pub decompressed_bytes: u64,
    pub install_bytes: u64,
}

/// Result of checking a single installed file against the manifest.
#[derive(Debug, Clone)]
pub struct FileVerification {
    pub filename: String,
    pub expected_size: u64,
    pub status: VerifyStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyStatus {
    Ok,
    Missing,
    HashMismatch,
}

/// Result of verifying an existing installation against the manifest.
#[derive(Debug, Clone, Default)]
pub struct VerifyReport {
    pub ok: u64,
    pub missing: u64,
    pub mismatched: u64,
    pub total_bytes: u64,
}

/// What a download run intends to do, computed without touching the network.
#[derive(Debug, Clone, Default)]
pub struct DownloadPlan {
    pub files_to_download: u64,
    pub files_skipped: u64,
    pub chunks_to_download: u64,
    pub download_bytes: u64,
    pub install_bytes: u64,
}

/// Result of a single pre-download pass over the manifest and install dir.
struct PreparedDownload {
    plan: DownloadPlan,
    /// Filenames whose on-disk contents already match the manifest; these are
    /// skipped by the writer without being re-hashed.
    matched: HashSet<String>,
    /// Reference counts per chunk guid across all files that need writing.
    refs: HashMap<u128, usize>,
    queue: VecDeque<u128>,
}

/// Downloads, writes and verifies an Epic Games installation from a manifest.
pub struct DownloadManager {
    manifest: Arc<Manifest>,
    install_dir: PathBuf,
    client: Client,
    options: DownloaderOptions,
    /// Set to signal the download run to stop. Workers, the writer and the
    /// progress reporter all observe it, mirroring error-triggered cancels.
    cancel: Arc<AtomicBool>,
    /// Optional shared download speed limit in bytes/s (0 = unlimited).
    /// Shared so the limit can be changed while a download is running.
    speed_limit: Option<SpeedLimit>,
}

/// Shared counters used by workers, the writer and the progress task.
struct DownloadStats {
    downloaded_bytes: AtomicU64,
    decompressed_bytes: AtomicU64,
    chunks_completed: AtomicU64,
    files_completed: AtomicU64,
    total_download_bytes: u64,
    total_chunks: u64,
    total_files: u64,
    /// Chunks already counted towards progress. A chunk evicted from the cache
    /// while still needed is re-downloaded; only its first download counts, so
    /// progress never exceeds the manifest's actual totals.
    counted_chunks: Mutex<HashSet<u128>>,
}

impl DownloadStats {
    fn snapshot(&self) -> DownloadProgress {
        DownloadProgress {
            downloaded_bytes: self.downloaded_bytes.load(Ordering::Relaxed),
            decompressed_bytes: self.decompressed_bytes.load(Ordering::Relaxed),
            total_download_bytes: self.total_download_bytes,
            chunks_completed: self.chunks_completed.load(Ordering::Relaxed),
            total_chunks: self.total_chunks,
            files_completed: self.files_completed.load(Ordering::Relaxed),
            total_files: self.total_files,
            download_speed_bps: 0.0,
        }
    }
}

/// In-memory LRU cache of decompressed chunks. The reference counts keep the
/// writer and the workers in sync: a chunk is cached only while at least one
/// file still needs it, and evicted entries that are still needed are
/// re-enqueued for download by the caller of [`ChunkStore::put`].
struct ChunkStore {
    inner: Mutex<ChunkStoreInner>,
    max_cached: usize,
    /// Signalled every time a chunk is inserted into the cache.
    new_chunk: Notify,
}

struct ChunkStoreInner {
    /// guid -> (data, insertion order).
    chunks: HashMap<u128, (Arc<Vec<u8>>, u64)>,
    /// Remaining parts across all unwritten files that reference each chunk.
    refs: HashMap<u128, usize>,
    order: u64,
}

impl ChunkStore {
    fn new(max_cached: usize, refs: HashMap<u128, usize>) -> Self {
        Self {
            inner: Mutex::new(ChunkStoreInner {
                chunks: HashMap::new(),
                refs,
                order: 0,
            }),
            max_cached,
            new_chunk: Notify::new(),
        }
    }

    /// Whether a chunk still has outstanding references (parts to write).
    fn is_needed(&self, guid: u128) -> bool {
        self.inner
            .lock()
            .unwrap()
            .refs
            .get(&guid)
            .copied()
            .unwrap_or(0)
            > 0
    }

    /// Insert a chunk. Never blocks. Drops the data if the chunk is no longer
    /// needed; otherwise caches it, evicting the oldest entry when at capacity
    /// and returning its guid so the caller can re-download it if still needed.
    fn put(&self, guid: u128, data: Vec<u8>) -> Option<u128> {
        let mut inner = self.inner.lock().unwrap();
        if inner.refs.get(&guid).copied().unwrap_or(0) == 0 {
            return None;
        }

        let order = inner.order;
        inner.order = inner.order.wrapping_add(1);
        inner.chunks.insert(guid, (Arc::new(data), order));

        let mut evicted = None;
        if inner.chunks.len() > self.max_cached
            && let Some((&oldest, _)) = inner.chunks.iter().min_by_key(|(_, (_, order))| *order)
        {
            inner.chunks.remove(&oldest);
            if inner.refs.contains_key(&oldest) {
                evicted = Some(oldest);
            }
        }
        drop(inner);
        self.new_chunk.notify_waiters();
        evicted
    }

    /// Return decompressed data for a chunk, or `None` if not yet cached.
    fn try_get(&self, guid: u128) -> Option<Arc<Vec<u8>>> {
        self.inner
            .lock()
            .unwrap()
            .chunks
            .get(&guid)
            .map(|(data, _)| Arc::clone(data))
    }

    /// Decrement a chunk's reference count, evicting it once it reaches zero.
    fn release(&self, guid: u128) {
        let mut inner = self.inner.lock().unwrap();
        let freed = match inner.refs.get_mut(&guid) {
            Some(remaining) if *remaining > 0 => {
                *remaining -= 1;
                *remaining == 0
            }
            _ => false,
        };
        if freed {
            inner.chunks.remove(&guid);
            inner.refs.remove(&guid);
        }
    }
}

impl DownloadManager {
    pub fn new(manifest: Manifest, install_dir: impl Into<PathBuf>) -> Self {
        Self::with_options(manifest, install_dir, DownloaderOptions::default())
    }

    pub fn with_options(
        manifest: Manifest,
        install_dir: impl Into<PathBuf>,
        options: DownloaderOptions,
    ) -> Self {
        Self {
            manifest: Arc::new(manifest),
            install_dir: install_dir.into(),
            client: Client::new(),
            options,
            cancel: Arc::new(AtomicBool::new(false)),
            speed_limit: None,
        }
    }

    /// Share a download speed limit with the caller. The atomic holds the
    /// maximum speed in bytes/s; 0 means unlimited. Changes take effect
    /// while the download is running.
    pub fn with_max_speed_bps(mut self, speed_limit: SpeedLimit) -> Self {
        self.speed_limit = Some(speed_limit);
        self
    }

    /// Share a cancellation handle with the caller. Calling [`cancel`] on the
    /// returned handle stops the running download.
    ///
    /// [`cancel`]: Self::cancel
    pub fn with_cancel_handle(mut self, cancel: Arc<AtomicBool>) -> Self {
        self.cancel = cancel;
        self
    }

    /// Request the current download run to stop as soon as possible.
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    /// Where the game is being installed to.
    pub fn install_dir(&self) -> &Path {
        &self.install_dir
    }

    /// The manifest this manager operates on.
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// Determine which files are missing or outdated without downloading.
    pub fn analyze(&self) -> Result<DownloadPlan, MonarchEgsError> {
        self.prepare().map(|prepared| prepared.plan)
    }

    /// Hash the existing installation once, producing everything a download
    /// run needs: the plan, the set of files that can be skipped, and the
    /// refcounted chunk queue. Every file on disk is read at most one time.
    fn prepare(&self) -> Result<PreparedDownload, MonarchEgsError> {
        let mut prepared = PreparedDownload {
            plan: DownloadPlan::default(),
            matched: HashSet::new(),
            refs: HashMap::new(),
            queue: VecDeque::new(),
        };

        for file in self.manifest.files() {
            if self.file_matches(file.filename(), file.sha1())? {
                prepared.plan.files_skipped += 1;
                prepared.matched.insert(file.filename().to_string());
                continue;
            }
            prepared.plan.files_to_download += 1;
            prepared.plan.install_bytes += file.file_size();
            for part in file.chunk_parts() {
                let guid = part.guid_num();
                let entry = prepared.refs.entry(guid).or_insert(0);
                if *entry == 0 {
                    prepared.queue.push_back(guid);
                }
                *entry += 1;
            }
        }

        prepared.plan.chunks_to_download = prepared.refs.len() as u64;
        prepared.plan.download_bytes = self
            .manifest
            .chunks()
            .iter()
            .filter(|c| prepared.refs.contains_key(&c.guid_num()))
            .map(|c| c.file_size())
            .sum();

        Ok(prepared)
    }

    /// Download, write and verify all missing files.
    pub async fn download(&self) -> Result<DownloadReport, MonarchEgsError> {
        self.run_download(None).await
    }

    /// Same as [`DownloadManager::download`] but streams progress events.
    pub async fn download_with_events(
        &self,
        tx: mpsc::Sender<DownloadEvent>,
    ) -> Result<DownloadReport, MonarchEgsError> {
        self.run_download(Some(tx)).await
    }

    /// Verify an existing installation against the manifest. Files that are
    /// missing or whose SHA1 does not match are reported; nothing is changed.
    pub async fn verify(&self) -> Result<VerifyReport, MonarchEgsError> {
        let files = self.verify_files()?;
        let mut report = VerifyReport::default();
        for verification in &files {
            match verification.status {
                VerifyStatus::Ok => report.ok += 1,
                VerifyStatus::Missing => report.missing += 1,
                VerifyStatus::HashMismatch => report.mismatched += 1,
            }
            report.total_bytes += verification.expected_size;
        }
        Ok(report)
    }

    /// Per-file verification results for the current installation.
    pub fn verify_files(&self) -> Result<Vec<FileVerification>, MonarchEgsError> {
        let files = self.manifest.files();
        let mut results = Vec::with_capacity(files.len());

        for file in files {
            let path = safe_join(&self.install_dir, file.filename()).ok_or_else(|| {
                MonarchEgsError::ParsingError(format!(
                    "Invalid filename in manifest: {}",
                    file.filename()
                ))
            })?;

            let status = if !path.exists() {
                VerifyStatus::Missing
            } else if file_sha1(&path).map(|h| h == *file.sha1()).unwrap_or(false) {
                VerifyStatus::Ok
            } else {
                VerifyStatus::HashMismatch
            };

            results.push(FileVerification {
                filename: file.filename().to_string(),
                expected_size: file.file_size(),
                status,
            });
        }

        Ok(results)
    }

    async fn run_download(
        &self,
        tx: Option<mpsc::Sender<DownloadEvent>>,
    ) -> Result<DownloadReport, MonarchEgsError> {
        let prepared = self.prepare()?;
        let plan = prepared.plan;
        let mut report = DownloadReport {
            files_skipped: plan.files_skipped,
            ..Default::default()
        };

        if plan.files_to_download == 0 {
            if let Some(tx) = &tx {
                let _ = tx
                    .send(DownloadEvent::Progress(DownloadProgress {
                        total_download_bytes: 0,
                        total_chunks: 0,
                        files_completed: plan.files_skipped,
                        total_files: plan.files_skipped,
                        ..Default::default()
                    }))
                    .await;
            }
            report.install_bytes = plan.install_bytes;
            return Ok(report);
        }

        std::fs::create_dir_all(&self.install_dir).map_err(|e| {
            MonarchEgsError::WebRequestError(format!(
                "Failed to create install dir {}: {e}",
                self.install_dir.display()
            ))
        })?;

        // GUID -> index lookup for the workers.
        let mut chunk_index: HashMap<u128, usize> = HashMap::new();
        for (idx, chunk) in self.manifest.chunks().iter().enumerate() {
            chunk_index.insert(chunk.guid_num(), idx);
        }
        let chunk_index = Arc::new(chunk_index);

        let stats = Arc::new(DownloadStats {
            downloaded_bytes: AtomicU64::new(0),
            decompressed_bytes: AtomicU64::new(0),
            chunks_completed: AtomicU64::new(0),
            files_completed: AtomicU64::new(0),
            total_download_bytes: plan.download_bytes,
            total_chunks: plan.chunks_to_download,
            total_files: plan.files_to_download,
            counted_chunks: Mutex::new(HashSet::new()),
        });

        let store = Arc::new(ChunkStore::new(
            self.options.max_cached_chunks,
            prepared.refs,
        ));
        let speed_limiter = Arc::new(SpeedLimiter::new(
            self.speed_limit
                .clone()
                .unwrap_or_else(|| Arc::new(AtomicU64::new(0))),
        ));
        info!(
            "Download run starting with speed limit of {} B/s (0 = unlimited)",
            speed_limiter.limit.load(Ordering::Relaxed)
        );
        let queue = Arc::new(Mutex::new(prepared.queue));
        let cancel = self.cancel.clone();
        let first_error: Arc<Mutex<Option<MonarchEgsError>>> = Arc::new(Mutex::new(None));
        let done = Arc::new(AtomicBool::new(false));

        // Spawn download workers.
        let mut workers = JoinSet::new();
        for _ in 0..self.options.max_workers.max(1) {
            let client = self.client.clone();
            let manifest = Arc::clone(&self.manifest);
            let chunk_index = Arc::clone(&chunk_index);
            let queue = Arc::clone(&queue);
            let store = Arc::clone(&store);
            let stats = Arc::clone(&stats);
            let cancel = Arc::clone(&cancel);
            let first_error = Arc::clone(&first_error);
            let speed_limiter = Arc::clone(&speed_limiter);
            let tx = tx.clone();

            workers.spawn(async move {
                worker_loop(
                    client,
                    manifest,
                    chunk_index,
                    queue,
                    store,
                    stats,
                    cancel,
                    first_error,
                    speed_limiter,
                    tx,
                )
                .await;
            });
        }

        // Progress reporter: periodically recompute speed and emit snapshots.
        let progress_tx = tx.clone();
        let progress_stats = Arc::clone(&stats);
        let progress_done = Arc::clone(&done);
        let progress_cancel = Arc::clone(&cancel);
        let progress_task = tokio::spawn(async move {
            let mut last = Instant::now();
            let mut last_bytes = 0u64;
            while !progress_done.load(Ordering::Relaxed) {
                tokio::time::sleep(Duration::from_millis(250)).await;
                let now = Instant::now();
                let mut snap = progress_stats.snapshot();
                let delta = now.duration_since(last).as_secs_f64();
                if delta > 0.0 {
                    snap.download_speed_bps =
                        (snap.downloaded_bytes.saturating_sub(last_bytes)) as f64 / delta;
                }
                last = now;
                last_bytes = snap.downloaded_bytes;
                if progress_cancel.load(Ordering::Relaxed) {
                    if let Some(progress_tx) = &progress_tx {
                        let _ = progress_tx.send(DownloadEvent::Progress(snap)).await;
                    }
                    break;
                }
                if let Some(progress_tx) = &progress_tx {
                    let _ = progress_tx.send(DownloadEvent::Progress(snap)).await;
                }
            }
        });

        // Write files, consuming chunks from the cache as they arrive. Each
        // file is hashed while it is assembled and then renamed on a
        // background thread, so writing and downloading never stall.
        let mut finalized: JoinSet<FinalizeResult> = JoinSet::new();
        let write_result = self
            .write_files(
                &store,
                &stats,
                tx.as_ref(),
                &cancel,
                &first_error,
                &prepared.matched,
                &mut finalized,
            )
            .await;

        // On success, wait for every background finalization to complete. On
        // an error/cancel path we drop the set instead: blocking tasks cannot
        // be aborted, they simply finish on their own (renaming their temp
        // files) while the run returns promptly.
        if write_result.is_none() && !cancel.load(Ordering::Relaxed) {
            while let Some(result) = finalized.join_next().await {
                handle_finalized(result, &stats, tx.as_ref(), &first_error, &cancel).await;
            }
        }
        finalized.abort_all();
        drop(finalized);

        done.store(true, Ordering::Relaxed);
        let _ = progress_task.await;

        // Wait for all workers to drain the queue.
        while let Some(result) = workers.join_next().await {
            if let Err(e) = result {
                error!("download worker panicked: {e}");
            }
        }

        if let Some(err) = write_result {
            if matches!(err, MonarchEgsError::Cancelled) {
                return Err(MonarchEgsError::Cancelled);
            }
            return Err(err);
        }

        if let Some(err) = first_error.lock().unwrap().take() {
            return Err(err);
        }

        // Distinguish an external stop request from a real error: cancel gets
        // set by file/chunk failures too, but those are returned above.
        if cancel.load(Ordering::Relaxed) {
            return Err(MonarchEgsError::Cancelled);
        }

        report.files_written = stats.files_completed.load(Ordering::Relaxed);
        report.chunks_downloaded = stats.chunks_completed.load(Ordering::Relaxed);
        report.downloaded_bytes = stats.downloaded_bytes.load(Ordering::Relaxed);
        report.decompressed_bytes = stats.decompressed_bytes.load(Ordering::Relaxed);
        report.install_bytes = plan.install_bytes;

        Ok(report)
    }

    /// Returns an error if any file failed to write or verify.
    #[allow(clippy::too_many_arguments)]
    async fn write_files(
        &self,
        store: &Arc<ChunkStore>,
        stats: &Arc<DownloadStats>,
        tx: Option<&mpsc::Sender<DownloadEvent>>,
        cancel: &Arc<AtomicBool>,
        first_error: &Arc<Mutex<Option<MonarchEgsError>>>,
        matched: &HashSet<String>,
        finalized: &mut JoinSet<FinalizeResult>,
    ) -> Option<MonarchEgsError> {
        for file in self.manifest.files() {
            // Harvest finalizations that completed while this file was being
            // written so their failures stop the run as early as possible.
            while let Some(result) = finalized.try_join_next() {
                handle_finalized(result, stats, tx, first_error, cancel).await;
            }
            if cancel.load(Ordering::Relaxed) {
                return first_error.lock().unwrap().clone();
            }
            if matched.contains(file.filename()) {
                if let Some(tx) = tx {
                    let _ = tx
                        .send(DownloadEvent::FileSkipped {
                            filename: file.filename().to_string(),
                        })
                        .await;
                }
                continue;
            }

            match self.write_file(store, file, cancel, finalized).await {
                Ok(()) => {}
                Err(e) => {
                    if matches!(e, MonarchEgsError::Cancelled) {
                        return Some(e);
                    }
                    {
                        let mut err = first_error.lock().unwrap();
                        if err.is_none() {
                            *err = Some(e.clone());
                        }
                    }
                    cancel.store(true, Ordering::Relaxed);
                    if let Some(tx) = tx {
                        let _ = tx
                            .send(DownloadEvent::FileFailed {
                                filename: file.filename().to_string(),
                                error: e.to_string(),
                            })
                            .await;
                    }
                    return Some(e);
                }
            }
        }
        None
    }

    async fn write_file(
        &self,
        store: &Arc<ChunkStore>,
        file: &FileManifest,
        cancel: &Arc<AtomicBool>,
        finalized: &mut JoinSet<FinalizeResult>,
    ) -> Result<(), MonarchEgsError> {
        let final_path = safe_join(&self.install_dir, file.filename()).ok_or_else(|| {
            MonarchEgsError::ParsingError(format!(
                "Invalid filename in manifest: {}",
                file.filename()
            ))
        })?;
        if let Some(parent) = final_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                MonarchEgsError::WebRequestError(format!(
                    "Failed to create directory {}: {e}",
                    parent.display()
                ))
            })?;
        }

        // Files with no chunk parts are simply created empty.
        if file.chunk_parts().is_empty() {
            let tmp = final_path.with_extension("monarch_tmp");
            std::fs::write(&tmp, []).map_err(|e| {
                MonarchEgsError::WebRequestError(format!(
                    "Failed to create empty file {}: {e}",
                    tmp.display()
                ))
            })?;
            spawn_finalize(finalized, tmp, final_path, file);
            return Ok(());
        }

        let tmp = final_path.with_extension("monarch_tmp");
        let mut out = tokio::fs::File::create(&tmp).await.map_err(|e| {
            MonarchEgsError::WebRequestError(format!("Failed to create {}: {e}", tmp.display()))
        })?;

        // Running hash of the assembled file. Every chunk was already SHA1
        // verified against the manifest during download, so hashing the exact
        // bytes handed to the OS gives the same end-to-end guarantee as a
        // readback pass without re-reading the file from disk afterwards.
        let mut hasher = Sha1::new();

        let mut pending: VecDeque<usize> = (0..file.chunk_parts().len()).collect();

        while let Some(&part_idx) = pending.front() {
            let part = &file.chunk_parts()[part_idx];
            let guid = part.guid_num();

            // Wait until this chunk is cached, reacting to cancellation so we
            // do not hang forever if every worker fails.
            let data = loop {
                if let Some(data) = store.try_get(guid) {
                    break data;
                }
                if cancel.load(Ordering::Relaxed) {
                    return Err(MonarchEgsError::Cancelled);
                }
                let notified = store.new_chunk.notified();
                if let Some(data) = store.try_get(guid) {
                    break data;
                }
                notified.await;
            };

            let offset = part.offset() as usize;
            let size = part.size() as usize;
            if offset + size > data.len() {
                return Err(MonarchEgsError::HashMismatchError(format!(
                    "Chunk part out of bounds for {}! | offset {offset}, size {size}, data {}",
                    file.filename(),
                    data.len()
                )));
            }

            let slice = &data[offset..offset + size];
            hasher.update(slice);
            out.seek(tokio::io::SeekFrom::Start(part.file_offset() as u64))
                .await
                .map_err(io_err)?;
            out.write_all(slice).await.map_err(io_err)?;

            store.release(guid);
            pending.pop_front();
        }

        out.flush().await.map_err(io_err)?;
        out.sync_all().await.map_err(io_err)?;
        drop(out);

        // Verify the assembled file and fail like a readback mismatch would.
        let actual: [u8; 20] = hasher.finalize().into();
        if actual != *file.sha1() {
            let _ = std::fs::remove_file(&tmp);
            return Err(MonarchEgsError::HashMismatchError(format!(
                "File verification failed for {} | Computed: {}, Expected: {}",
                file.filename(),
                hex(&actual),
                hex(file.sha1())
            )));
        }

        // Hand the file off to a background thread for the rename and
        // permission bits; the writer moves on to the next file immediately.
        spawn_finalize(finalized, tmp, final_path, file);
        Ok(())
    }

    /// True if the file on disk already matches the manifest hash.
    fn file_matches(&self, filename: &str, expected: &[u8; 20]) -> Result<bool, MonarchEgsError> {
        let path = match safe_join(&self.install_dir, filename) {
            Some(p) => p,
            None => {
                return Err(MonarchEgsError::ParsingError(format!(
                    "Invalid filename in manifest: {filename}"
                )));
            }
        };
        if !path.exists() {
            return Ok(false);
        }
        Ok(match file_sha1(&path) {
            Ok(hash) => hash == *expected,
            Err(_) => false,
        })
    }
}

/// Download a single chunk, trying each base URL in turn.
async fn download_chunk(
    client: &Client,
    manifest: &Manifest,
    chunk: &super::ChunkInfo,
) -> Result<Vec<u8>, MonarchEgsError> {
    let path = chunk.path(manifest.feature_level());
    let mut last_err: Option<MonarchEgsError> = None;

    for base in manifest.base_urls() {
        let url = format!("{base}/{path}");
        let response = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                last_err = Some(MonarchEgsError::WebRequestError(format!(
                    "Failed to request {url}: {e}"
                )));
                continue;
            }
        };
        if !response.status().is_success() {
            last_err = Some(MonarchEgsError::WebRequestError(format!(
                "Failed to download chunk from {url} | Status: {}",
                response.status()
            )));
            continue;
        }
        let bytes = match response.bytes().await {
            Ok(b) => b.to_vec(),
            Err(e) => {
                last_err = Some(MonarchEgsError::WebRequestError(format!(
                    "Failed to read body of {url}: {e}"
                )));
                continue;
            }
        };

        // Decrypt/decompress/hash on a blocking thread: this is pure CPU work
        // (AES + zlib + SHA1) and must not occupy a runtime worker, so the
        // other workers keep hitting the network while it runs.
        let chunk = chunk.clone();
        return match tokio::task::spawn_blocking(move || process_chunk(&bytes, &chunk)).await {
            Ok(result) => result,
            Err(e) => Err(MonarchEgsError::WebRequestError(format!(
                "Chunk processing task failed: {e}"
            ))),
        };
    }

    Err(last_err.unwrap_or_else(|| {
        MonarchEgsError::WebRequestError("No CDN base URLs available".to_string())
    }))
}

#[allow(clippy::too_many_arguments)]
async fn worker_loop(
    client: Client,
    manifest: Arc<Manifest>,
    chunk_index: Arc<HashMap<u128, usize>>,
    queue: Arc<Mutex<VecDeque<u128>>>,
    store: Arc<ChunkStore>,
    stats: Arc<DownloadStats>,
    cancel: Arc<AtomicBool>,
    first_error: Arc<Mutex<Option<MonarchEgsError>>>,
    speed_limiter: Arc<SpeedLimiter>,
    tx: Option<mpsc::Sender<DownloadEvent>>,
) {
    loop {
        if cancel.load(Ordering::Relaxed) {
            // Wake the writer in case it is parked on `new_chunk`; it will
            // observe the cancel and bail out with [`MonarchEgsError::Cancelled`].
            store.new_chunk.notify_waiters();
            return;
        }
        let guid = queue.lock().unwrap().pop_front();
        let Some(guid) = guid else { return };

        // Skip chunks that have already been fully consumed by the writer.
        if !store.is_needed(guid) {
            continue;
        }

        let Some(&idx) = chunk_index.get(&guid) else {
            let mut err = first_error.lock().unwrap();
            if err.is_none() {
                *err = Some(MonarchEgsError::ParsingError(format!(
                    "Manifest references unknown chunk guid {guid}"
                )));
            }
            cancel.store(true, Ordering::Relaxed);
            return;
        };

        let chunk = &manifest.chunks()[idx];

        // Respect the configured speed limit before hitting the network.
        speed_limiter.consume(chunk.file_size()).await;

        match download_chunk(&client, &manifest, chunk).await {
            Ok(data) => {
                let decompressed = data.len() as u64;
                let downloaded = chunk.file_size();
                // Only the first download of a chunk counts towards progress.
                // Chunks evicted from the cache while still needed are fetched
                // again, but re-downloading them must not inflate the totals.
                let first_download = {
                    let mut counted = stats.counted_chunks.lock().unwrap();
                    counted.insert(guid)
                };
                if first_download {
                    stats
                        .downloaded_bytes
                        .fetch_add(downloaded, Ordering::Relaxed);
                    stats
                        .decompressed_bytes
                        .fetch_add(decompressed, Ordering::Relaxed);
                }

                let evicted = store.put(guid, data);
                // A chunk evicted from the cache that is still needed must be
                // downloaded again.
                if let Some(evicted) = evicted
                    && store.is_needed(evicted)
                {
                    queue.lock().unwrap().push_front(evicted);
                }

                if first_download {
                    stats.chunks_completed.fetch_add(1, Ordering::Relaxed);
                    if let Some(tx) = &tx {
                        let _ = tx
                            .send(DownloadEvent::ChunkDownloaded {
                                guid,
                                downloaded,
                                decompressed,
                            })
                            .await;
                    }
                }
            }
            Err(e) => {
                let mut err = first_error.lock().unwrap();
                if err.is_none() {
                    *err = Some(e);
                }
                cancel.store(true, Ordering::Relaxed);
                return;
            }
        }
    }
}

/// Queue a fully-written temp file for finalization on a background thread.
fn spawn_finalize(
    finalized: &mut JoinSet<FinalizeResult>,
    tmp: PathBuf,
    final_path: PathBuf,
    file: &FileManifest,
) {
    let filename = file.filename().to_string();
    let file = file.clone();
    finalized.spawn_blocking(move || {
        let result = finalize_file(tmp, final_path, file);
        (filename, result)
    });
}

/// Process a completed background finalization: update stats and emit events.
/// A failed finalization records the error and cancels the run, matching the
/// behaviour of inline write failures.
async fn handle_finalized(
    result: Result<FinalizeResult, tokio::task::JoinError>,
    stats: &Arc<DownloadStats>,
    tx: Option<&mpsc::Sender<DownloadEvent>>,
    first_error: &Arc<Mutex<Option<MonarchEgsError>>>,
    cancel: &Arc<AtomicBool>,
) {
    let (filename, result) = match result {
        Ok(pair) => pair,
        Err(e) => {
            error!("file verification task panicked: {e}");
            return;
        }
    };
    match result {
        Ok(bytes) => {
            stats.files_completed.fetch_add(1, Ordering::Relaxed);
            if let Some(tx) = tx {
                let _ = tx
                    .send(DownloadEvent::FileWritten { filename, bytes })
                    .await;
            }
        }
        Err(e) => {
            {
                let mut err = first_error.lock().unwrap();
                if err.is_none() {
                    *err = Some(e.clone());
                }
            }
            cancel.store(true, Ordering::Relaxed);
            if let Some(tx) = tx {
                let _ = tx
                    .send(DownloadEvent::FileFailed {
                        filename,
                        error: e.to_string(),
                    })
                    .await;
            }
        }
    }
}

/// Promote a verified temp file into place: atomic rename plus the executable
/// bit if requested. Content was already checked while the file was assembled
/// (see [`DownloadManager::write_file`]), so this only finalizes filesystem
/// state. Runs on a blocking thread to keep the writer responsive.
fn finalize_file(
    tmp: PathBuf,
    final_path: PathBuf,
    file: FileManifest,
) -> Result<u64, MonarchEgsError> {
    std::fs::rename(&tmp, &final_path).map_err(|e| {
        MonarchEgsError::WebRequestError(format!(
            "Failed to rename {} to {}: {e}",
            tmp.display(),
            final_path.display()
        ))
    })?;

    if file.is_executable() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = std::fs::metadata(&final_path) {
                let mode = meta.permissions().mode() | 0o111;
                let _ =
                    std::fs::set_permissions(&final_path, std::fs::Permissions::from_mode(mode));
            }
        }
    }

    Ok(file.file_size())
}

fn file_sha1(path: &Path) -> std::io::Result<[u8; 20]> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha1::new();
    let mut buf = [0u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buf)?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
    }
    Ok(hasher.finalize().into())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn io_err(e: std::io::Error) -> MonarchEgsError {
    MonarchEgsError::WebRequestError(format!("I/O error: {e}"))
}

/// Join a manifest path onto the install directory, rejecting anything that
/// would escape it (`..`, absolute paths, drive prefixes).
fn safe_join(base: &Path, filename: &str) -> Option<PathBuf> {
    let normalized = filename.replace('\\', "/");
    // Reject absolute paths (leading '/'), UNC-style paths and Windows drive
    // prefixes outright instead of silently normalizing them to relative.
    if normalized.starts_with('/') || normalized.len() >= 2 && &normalized[1..2] == ":" {
        return None;
    }

    let mut out = base.to_path_buf();
    for component in normalized.split('/') {
        match component {
            "" | "." => continue,
            ".." => return None,
            c => out.push(c),
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::{SpeedLimiter, safe_join};
    use std::path::Path;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{Duration, Instant};
    use tokio::task::JoinSet;

    #[test]
    fn joins_relative_paths() {
        assert_eq!(
            safe_join(Path::new("/games/Fortnite"), "Engine/Binaries/foo.exe"),
            Some(Path::new("/games/Fortnite/Engine/Binaries/foo.exe").to_path_buf())
        );
    }

    #[test]
    fn normalizes_windows_separators() {
        assert_eq!(
            safe_join(Path::new("/games"), r"Engine\Binaries\foo.dll"),
            Some(Path::new("/games/Engine/Binaries/foo.dll").to_path_buf())
        );
    }

    #[test]
    fn rejects_traversal() {
        assert_eq!(safe_join(Path::new("/games"), "../etc/passwd"), None);
        assert_eq!(safe_join(Path::new("/games"), "a/../../b"), None);
    }

    #[test]
    fn rejects_absolute_paths() {
        assert_eq!(safe_join(Path::new("/games"), "/etc/passwd"), None);
        assert_eq!(safe_join(Path::new("/games"), "C:/Windows/system32"), None);
    }

    #[test]
    fn ignores_dot_segments() {
        assert_eq!(
            safe_join(Path::new("/games"), "./Engine/./foo"),
            Some(Path::new("/games/Engine/foo").to_path_buf())
        );
    }

    #[test]
    fn unlimited_limit_never_waits() {
        let limiter = SpeedLimiter::new(Arc::new(AtomicU64::new(0)));
        assert_eq!(limiter.reserve(u64::MAX), Duration::ZERO);
    }

    #[test]
    fn cold_start_has_no_full_speed_burst() {
        let limiter = SpeedLimiter::new(Arc::new(AtomicU64::new(1_000_000)));
        // A full second's worth must NOT be free on a fresh limiter.
        assert!(!limiter.reserve(1_000_000).is_zero());
    }

    #[test]
    fn small_burst_within_burst_window_is_free() {
        let limiter = SpeedLimiter::new(Arc::new(AtomicU64::new(1_000_000)));
        // Let the bucket bank its MAX_BURST_SECS worth of credit.
        std::thread::sleep(Duration::from_millis(260));
        assert_eq!(limiter.reserve(200_000), Duration::ZERO);
    }

    #[test]
    fn exceeding_budget_requires_waiting() {
        let limiter = SpeedLimiter::new(Arc::new(AtomicU64::new(1_000_000)));
        std::thread::sleep(Duration::from_millis(260));
        // A quarter second's worth is available; drains the bucket fully.
        assert_eq!(limiter.reserve(250_000), Duration::ZERO);
        // 500 KiB more at 1 MiB/s => ~0.5s wait before it can be granted.
        let wait = limiter.reserve(500_000);
        assert!(wait >= Duration::from_millis(400) && wait <= Duration::from_millis(600));
    }

    /// The user-facing property: several workers pulling through one limiter
    /// together sustain no more than roughly the configured rate over time.
    #[tokio::test]
    async fn concurrent_workers_sustain_the_limit() {
        let limit_bps: u64 = 4_000_000;
        let limiter = Arc::new(SpeedLimiter::new(Arc::new(AtomicU64::new(limit_bps))));

        let started = Instant::now();
        let mut tasks = JoinSet::new();
        for _ in 0..4 {
            let limiter = Arc::clone(&limiter);
            tasks.spawn(async move {
                for _ in 0..8 {
                    limiter.consume(500_000).await;
                }
            });
        }
        while tasks.join_next().await.is_some() {}

        // 16 MB total minus the 1 MB burst window => ~3.75s at 4 MB/s.
        // Assert with slack below that so slow scheduling cannot flake.
        let elapsed = started.elapsed();
        assert!(elapsed >= Duration::from_secs_f64(3.0), "took {elapsed:?}");
    }

    #[test]
    fn raising_the_limit_takes_effect_immediately() {
        let limit = Arc::new(AtomicU64::new(1_000));
        let limiter = SpeedLimiter::new(Arc::clone(&limit));
        assert!(!limiter.reserve(2_000).is_zero());

        limit.store(10_000_000, Ordering::Relaxed);
        // Let the refill budget recover under the raised limit.
        std::thread::sleep(Duration::from_millis(2));
        assert_eq!(limiter.reserve(2_000), Duration::ZERO);
    }
}
