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
use tracing::{debug, error, info};

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

/// Token-bucket rate limiter shared by all download workers.
///
/// Workers call [`consume`] for each slice of response body they read. Tokens
/// refill at the configured byte rate and are capped at [`MAX_BURST_SECS`] of
/// credit so idle periods cannot bank a large full-speed burst. Pacing at
/// read granularity keeps connections feeding steadily under a speed cap 
/// instead of oscillating between transfer bursts and long sleeps.
struct SpeedLimiter {
    inner: Mutex<SpeedLimiterInner>,
    limit: SpeedLimit,
}

/// How much bandwidth credit the bucket may bank relative to the limit.
const MAX_BURST_SECS: f64 = 0.25;

struct SpeedLimiterInner {
    /// Available download budget in bytes. Grows at `limit` B/s, shrinks when
    /// a worker consumes body bytes.
    tokens: f64,
    /// Wall-clock moment when [`tokens`] was last refreshed.
    last: Instant,
}

impl SpeedLimiter {
    fn new(limit: SpeedLimit) -> Self {
        Self {
            inner: Mutex::new(SpeedLimiterInner {
                tokens: 0.0,
                last: Instant::now(),
            }),
            limit,
        }
    }

    /// Block until the limiter has budget for `bytes`. Returns immediately
    /// when the limit is 0 (unlimited) or `bytes` is 0.
    /// Consume `bytes` from the shared budget, blocking (asynchronously) until
    /// enough bandwidth is available. Returns immediately when the limit is
    /// 0 (unlimited) or `bytes` is 0. The mutex is only ever held within a
    /// single non-awaiting block so the future stays `Send`.
    async fn consume(&self, bytes: u64) {
        // No data || unlimited speed
        if bytes == 0 || self.limit.load(Ordering::Relaxed) == 0 {
            return;
        }

        loop {
            // Do the bookkeeping and decide how long to wait under one lock,
            // releasing the guard before any `.await`.
            let wait = {
                let mut inner = self.inner.lock().unwrap();
                let bps = self.limit.load(Ordering::Relaxed);
                if bps == 0 {
                    return;
                }

                let now = Instant::now();
                let elapsed = now.duration_since(inner.last).as_secs_f64();
                inner.last = now;

                // Cap idle banking at MAX_BURST_SECS, but never below the
                // current request — otherwise a single consume larger than
                // the burst window could never be granted.
                let burst_cap = (bps as f64 * MAX_BURST_SECS).max(bytes as f64);
                inner.tokens = (inner.tokens + elapsed * bps as f64).min(burst_cap);

                if inner.tokens >= bytes as f64 {
                    inner.tokens -= bytes as f64;
                    return;
                }

                // Not enough tokens — sleep until the deficit is covered,
                // capped so limit changes are picked up promptly.
                let (bps, deficit) = {
                    let bps = self.limit.load(Ordering::Relaxed);
                    if bps == 0 {
                        return;
                    }
                    let now = Instant::now();
                    let elapsed = now.duration_since(inner.last).as_secs_f64();
                    let burst_cap = (bps as f64 * MAX_BURST_SECS).max(bytes as f64);
                    let covered = (inner.tokens + elapsed * bps as f64).min(burst_cap);
                    (bps, bytes as f64 - covered)
                };
                if deficit <= 0.0 {
                    continue;
                }
                Duration::from_secs_f64(deficit / bps as f64).min(Duration::from_millis(50))
            };
            tokio::time::sleep(wait).await;
        }
    }
}

/// High-level stage of an in-progress download run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DownloadPhase {
    /// Comparing on-disk files against the manifest before transferring.
    VerifyingExisting,
    /// Fetching and writing missing chunks/files.
    #[default]
    Downloading,
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
    /// Whether this snapshot is from the pre-download verify pass or the
    /// actual transfer.
    pub phase: DownloadPhase,
}

impl DownloadProgress {
    /// Overall completion percentage in the 0.0..1.0 range.
    pub fn fraction(&self) -> f64 {
        if self.total_files == 0 {
            // Unknown / not started — do not treat as complete (that made the
            // UI look finished while prepare() was still hashing on disk).
            return 0.0;
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

/// Live progress snapshot of a verification run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerifyProgress {
    /// Files hashed so far.
    pub files_checked: u64,
    /// Total files listed in the manifest.
    pub total_files: u64,
}

impl VerifyProgress {
    /// Completion as a whole percentage, floored to the nearest percent.
    pub fn percent(&self) -> u64 {
        if self.total_files == 0 {
            return 100;
        }
        ((self.files_checked.min(self.total_files) as f64 / self.total_files as f64) * 100.0)
            .floor() as u64
    }
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
            phase: DownloadPhase::Downloading,
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

    /// Determine which files are missing or outdated without downloading.
    pub fn analyze(&self) -> Result<DownloadPlan, MonarchEgsError> {
        self.prepare_inner(|_checked, _total| Ok(false))
            .map(|prepared| prepared.plan)
    }

    /// Hash the existing installation once, producing everything a download
    /// run needs: the plan, the set of files that can be skipped, and the
    /// refcounted chunk queue. Every file on disk is read at most one time.
    /// Returns [`Cancelled`](MonarchEgsError::Cancelled) if the run is stopped.
    ///
    /// Hashing runs on the blocking pool so a large resume/partial install
    /// cannot freeze the async runtime (which previously left the UI at
    /// 0 Mbps with no progress events).
    async fn prepare_async(
        &self,
        tx: Option<&mpsc::Sender<DownloadEvent>>,
    ) -> Result<PreparedDownload, MonarchEgsError> {
        let total_files = self.manifest.files().len() as u64;
        if let Some(tx) = tx {
            let _ = tx
                .send(DownloadEvent::Progress(DownloadProgress {
                    total_files,
                    files_completed: 0,
                    phase: DownloadPhase::VerifyingExisting,
                    ..Default::default()
                }))
                .await;
        }

        let cancel = self.cancel.clone();
        let install_dir = self.install_dir.clone();
        let mut last_percent: Option<u64> = None;

        // Build the plan file-by-file so we can cancel and report progress.
        let mut prepared = PreparedDownload {
            plan: DownloadPlan::default(),
            matched: HashSet::new(),
            refs: HashMap::new(),
            queue: VecDeque::new(),
        };

        let files: Vec<_> = self.manifest.files().iter().cloned().collect();
        for file in files {
            if cancel.load(Ordering::Relaxed) {
                return Err(MonarchEgsError::Cancelled);
            }

            let filename = file.filename().to_string();
            let expected = *file.sha1();
            let expected_size = file.file_size();
            let dir = install_dir.clone();
            let matches = tokio::task::spawn_blocking(move || {
                file_matches_path(&dir, &filename, &expected, expected_size)
            })
            .await
            .map_err(|e| {
                MonarchEgsError::WebRequestError(format!("prepare hash task failed: {e}"))
            })??;

            // Always count total install size so a fully-skipped run still
            // reports a meaningful size_on_disk to the library.
            prepared.plan.install_bytes += file.file_size();

            if matches {
                prepared.plan.files_skipped += 1;
                prepared.matched.insert(file.filename().to_string());
            } else {
                prepared.plan.files_to_download += 1;
                for part in file.chunk_parts() {
                    let guid = part.guid_num();
                    let entry = prepared.refs.entry(guid).or_insert(0);
                    if *entry == 0 {
                        prepared.queue.push_back(guid);
                    }
                    *entry += 1;
                }
            }

            let checked = prepared.plan.files_skipped + prepared.plan.files_to_download;
            if let Some(tx) = tx {
                let snapshot = DownloadProgress {
                    total_files,
                    files_completed: checked,
                    phase: DownloadPhase::VerifyingExisting,
                    ..Default::default()
                };
                let percent = if total_files == 0 {
                    100
                } else {
                    ((checked as f64 / total_files as f64) * 100.0).floor() as u64
                };
                if last_percent != Some(percent) || checked == total_files {
                    last_percent = Some(percent);
                    let _ = tx.send(DownloadEvent::Progress(snapshot)).await;
                }
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

    /// Shared sync prepare used by [`analyze`](Self::analyze).
    ///
    /// `on_file` is invoked after each file with `(checked, total)` and may
    /// return `true` to abort with [`Cancelled`](MonarchEgsError::Cancelled).
    fn prepare_inner(
        &self,
        mut on_file: impl FnMut(u64, u64) -> Result<bool, MonarchEgsError>,
    ) -> Result<PreparedDownload, MonarchEgsError> {
        let mut prepared = PreparedDownload {
            plan: DownloadPlan::default(),
            matched: HashSet::new(),
            refs: HashMap::new(),
            queue: VecDeque::new(),
        };
        let total_files = self.manifest.files().len() as u64;

        for file in self.manifest.files() {
            // Prefer a size check before SHA1 so partial/corrupt files are
            // rejected cheaply on resume.
            if file_matches_path(
                &self.install_dir,
                file.filename(),
                file.sha1(),
                file.file_size(),
            )? {
                prepared.plan.files_skipped += 1;
                prepared.matched.insert(file.filename().to_string());
            } else {
                prepared.plan.files_to_download += 1;
                for part in file.chunk_parts() {
                    let guid = part.guid_num();
                    let entry = prepared.refs.entry(guid).or_insert(0);
                    if *entry == 0 {
                        prepared.queue.push_back(guid);
                    }
                    *entry += 1;
                }
            }
            prepared.plan.install_bytes += file.file_size();

            let checked = prepared.plan.files_skipped + prepared.plan.files_to_download;
            if on_file(checked, total_files)? {
                return Err(MonarchEgsError::Cancelled);
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

    /// Same as [`DownloadManager::download`] but streams progress events.
    pub async fn download_with_events(
        &self,
        tx: mpsc::Sender<DownloadEvent>,
    ) -> Result<DownloadReport, MonarchEgsError> {
        self.run_download(Some(tx)).await
    }

    /// Verify an existing installation against the manifest. Files that are
    /// missing or whose SHA1 does not match are reported; nothing is changed.
    pub async fn verify_with_progress(
        &self,
        tx: mpsc::Sender<VerifyProgress>,
    ) -> Result<VerifyReport, MonarchEgsError> {
        self.run_verification(Some(&tx)).await
    }

    async fn run_verification(
        &self,
        tx: Option<&mpsc::Sender<VerifyProgress>>,
    ) -> Result<VerifyReport, MonarchEgsError> {
        let files = self.manifest.files();
        let total_files = files.len() as u64;
        let mut report = VerifyReport::default();
        let mut files_checked = 0u64;
        let mut last_percent: Option<u64> = None;

        for file in files {
            let verification = self.check_file(file)?;

            match verification.status {
                VerifyStatus::Ok => report.ok += 1,
                VerifyStatus::Missing => report.missing += 1,
                VerifyStatus::HashMismatch => report.mismatched += 1,
            }
            report.total_bytes += verification.expected_size;

            if let Some(tx) = tx {
                files_checked += 1;
                let snapshot = VerifyProgress {
                    files_checked,
                    total_files,
                };

                // Throttle: emit only when the floored percentage changes (or
                // on completion) so huge installs do not flood the channel.
                let percent = snapshot.percent();
                if last_percent != Some(percent) || files_checked == total_files {
                    last_percent = Some(percent);
                    debug!(
                        "Verification {}% ({}/{} files)",
                        percent, files_checked, total_files
                    );
                    let _ = tx.send(snapshot).await;
                }
            }
        }

        Ok(report)
    }

    /// Per-file verification results for the current installation.
    pub fn verify_files(&self) -> Result<Vec<FileVerification>, MonarchEgsError> {
        self.manifest
            .files()
            .iter()
            .map(|file| self.check_file(file))
            .collect()
    }

    /// Checks a single installed file against its manifest hash.
    fn check_file(&self, file: &FileManifest) -> Result<FileVerification, MonarchEgsError> {
        let path = safe_join(&self.install_dir, file.filename()).ok_or_else(|| {
            MonarchEgsError::ParsingError(format!(
                "Invalid filename in manifest: {}",
                file.filename()
            ))
        })?;

        let status = if !path.exists() {
            VerifyStatus::Missing
        } else if file_matches_path(
            &self.install_dir,
            file.filename(),
            file.sha1(),
            file.file_size(),
        )? {
            VerifyStatus::Ok
        } else {
            VerifyStatus::HashMismatch
        };

        Ok(FileVerification {
            filename: file.filename().to_string(),
            expected_size: file.file_size(),
            status,
        })
    }

    async fn run_download(
        &self,
        tx: Option<mpsc::Sender<DownloadEvent>>,
    ) -> Result<DownloadReport, MonarchEgsError> {
        let prepared = self.prepare_async(tx.as_ref()).await?;
        let plan = prepared.plan;
        let mut report = DownloadReport {
            files_skipped: plan.files_skipped,
            ..Default::default()
        };

        if plan.files_to_download == 0 {
            if self.cancel.load(Ordering::Relaxed) {
                return Err(MonarchEgsError::Cancelled);
            }
            if let Some(tx) = &tx {
                let _ = tx
                    .send(DownloadEvent::Progress(DownloadProgress {
                        total_download_bytes: 0,
                        total_chunks: 0,
                        files_completed: plan.files_skipped,
                        total_files: plan.files_skipped.max(1),
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
        let speed_limit = self.speed_limit
            .clone()
            .unwrap_or_else(|| Arc::new(AtomicU64::new(0)));
        let speed_limiter = Arc::new(SpeedLimiter::new(speed_limit.clone()));
        info!(
            "Download run starting with speed limit of {} B/s (0 = unlimited)",
            speed_limit.load(Ordering::Relaxed)
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
        // Speed is an EMA over the 250 ms samples so brief gaps between body
        // reads (or between chunks) do not flash the UI as 0 B/s.
        let progress_tx = tx.clone();
        let progress_stats = Arc::clone(&stats);
        let progress_done = Arc::clone(&done);
        let progress_cancel = Arc::clone(&cancel);
        let progress_task = tokio::spawn(async move {
            const SPEED_EMA_ALPHA: f64 = 0.35;
            let mut last = Instant::now();
            let mut last_bytes = 0u64;
            let mut smoothed_bps = 0.0f64;
            while !progress_done.load(Ordering::Relaxed) {
                tokio::time::sleep(Duration::from_millis(250)).await;
                let now = Instant::now();
                let mut snap = progress_stats.snapshot();
                let delta = now.duration_since(last).as_secs_f64();
                if delta > 0.0 {
                    let instant =
                        (snap.downloaded_bytes.saturating_sub(last_bytes)) as f64 / delta;
                    smoothed_bps = if smoothed_bps <= 0.0 {
                        instant
                    } else {
                        smoothed_bps + SPEED_EMA_ALPHA * (instant - smoothed_bps)
                    };
                    // Drop residual noise once transfers have truly stopped.
                    if instant == 0.0 && smoothed_bps < 1_000.0 {
                        smoothed_bps = 0.0;
                    }
                    snap.download_speed_bps = smoothed_bps;
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
}

/// True when `install_dir/filename` exists and matches the expected SHA1.
///
/// When `expected_size` is not [`u64::MAX`], a mismatched file length returns
/// `false` without hashing — important for resume of partial downloads.
fn file_matches_path(
    install_dir: &Path,
    filename: &str,
    expected: &[u8; 20],
    expected_size: u64,
) -> Result<bool, MonarchEgsError> {
    let path = match safe_join(install_dir, filename) {
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
    if expected_size != u64::MAX {
        match std::fs::metadata(&path) {
            Ok(meta) if meta.len() != expected_size => return Ok(false),
            Ok(_) => {}
            Err(_) => return Ok(false),
        }
    }
    Ok(match file_sha1(&path) {
        Ok(hash) => hash == *expected,
        Err(_) => false,
    })
}

/// Download a single chunk, trying each base URL in turn. Body bytes are
/// paced through `speed_limiter` as they arrive so a speed cap keeps the
/// connection fed instead of reserving the whole chunk before the request.
///
/// When `downloaded_bytes` is provided, each body slice is added to the
/// counter as it arrives so progress/speed stay continuous under a cap.
/// On a failed read the added bytes are rolled back.
async fn download_chunk(
    client: &Client,
    manifest: &Manifest,
    chunk: &super::ChunkInfo,
    speed_limiter: &SpeedLimiter,
    downloaded_bytes: Option<&AtomicU64>,
) -> Result<Vec<u8>, MonarchEgsError> {
    let path = chunk.path(manifest.feature_level());
    let mut last_err: Option<MonarchEgsError> = None;

    for base in manifest.base_urls() {
        let url = format!("{base}/{path}");
        let mut response = match client.get(&url).send().await {
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

        let mut bytes = Vec::with_capacity(chunk.file_size() as usize);
        let mut streamed = 0u64;
        let mut read_err = None;
        loop {
            match response.chunk().await {
                Ok(Some(piece)) => {
                    let n = piece.len() as u64;
                    speed_limiter.consume(n).await;
                    if let Some(counter) = downloaded_bytes {
                        counter.fetch_add(n, Ordering::Relaxed);
                        streamed += n;
                    }
                    bytes.extend_from_slice(&piece);
                }
                Ok(None) => break,
                Err(e) => {
                    read_err = Some(MonarchEgsError::WebRequestError(format!(
                        "Failed to read body of {url}: {e}"
                    )));
                    break;
                }
            }
        }
        if let Some(err) = read_err {
            if let Some(counter) = downloaded_bytes {
                counter.fetch_sub(streamed, Ordering::Relaxed);
            }
            last_err = Some(err);
            continue;
        }

        // Decrypt/decompress/hash on a blocking thread: this is pure CPU work
        // (AES + zlib + SHA1) and must not occupy a runtime worker, so the
        // other workers keep hitting the network while it runs.
        let chunk = chunk.clone();
        return match tokio::task::spawn_blocking(move || process_chunk(&bytes, &chunk)).await {
            Ok(Ok(data)) => Ok(data),
            Ok(Err(e)) => {
                if let Some(counter) = downloaded_bytes {
                    counter.fetch_sub(streamed, Ordering::Relaxed);
                }
                Err(e)
            }
            Err(e) => {
                if let Some(counter) = downloaded_bytes {
                    counter.fetch_sub(streamed, Ordering::Relaxed);
                }
                Err(MonarchEgsError::WebRequestError(format!(
                    "Chunk processing task failed: {e}"
                )))
            }
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

        // Claim the chunk before streaming so concurrent re-downloads of an
        // evicted guid cannot double-count. Failed downloads release the claim.
        let first_download = {
            let mut counted = stats.counted_chunks.lock().unwrap();
            counted.insert(guid)
        };
        let progress_counter = first_download.then_some(&stats.downloaded_bytes);

        match download_chunk(&client, &manifest, chunk, &speed_limiter, progress_counter).await {
            Ok(data) => {
                let decompressed = data.len() as u64;
                let downloaded = chunk.file_size();
                if first_download {
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
                if first_download {
                    stats.counted_chunks.lock().unwrap().remove(&guid);
                }
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
