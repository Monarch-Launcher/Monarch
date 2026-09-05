/*
 * Monarchs game downloading implementation.
 *
 * This file abstracts the game downloading implemtation from the underlying OS and game store.
 * This also allows for other QoL features, such as checking for ongoing downloads on start and exit,
 * cancelling and more.
 *
 * NOTE: Work in progress as more platforms are integrated, this will likely have to adapt to fit
 * more platforms needs ands quirks.
 */

use anyhow::{bail, Result};
use std::any::Any;
use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, RwLock};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tracing::{debug, error, info};

use crate::monarch_games::{monarchgame::MonarchGame, stores::DownloadOptions};

use monarch_egs::{DownloadEvent, DownloadManager, DownloadProgress};

pub trait DownloadHandler: Debug + Send + Sync {
    fn download(&self, job: &DownloadJob) -> Result<(), String>;
    fn cancel(&self, job: &DownloadJob) -> Result<(), String>;
    fn is_downloading(&self) -> bool;
    /// Applies a maximum download speed in bytes/s (0 = unlimited). Handlers
    /// should share it with their running downloads so changes take effect
    /// immediately. Defaults to a no-op for stores without throttling support.
    fn set_max_speed_bps(&self, _bps: u64) {}
}

#[derive(Debug, Clone)]
pub struct DownloadJob {
    id: u64,
    name: String,
    game_id: String,
    path: PathBuf,
    store: String,
    os: String,
    // Depending on the store the manifest will likely have different shapes and therefore be different types.
    // Arc so DownloadJob can Clone without requiring the concrete manifest to be Clone.
    manifest: Arc<dyn Any + Send + Sync>,
    // Original game the job was created for, so a finished download can be
    // added back into the library as an installed game.
    game: MonarchGame,
}

impl DownloadJob {
    pub fn new<T: Any + Send + Sync>(
        game: &MonarchGame,
        options: DownloadOptions,
        manifest: T,
    ) -> Self {
        // Carry the selected compatibility layer on the job's game copy so it
        // is applied when the install finishes and the game is registered.
        let mut game = game.clone();
        if let Some(compat) = options.compatibility.filter(|c| !c.is_empty()) {
            game.compatibility = Some(compat);
        }

        // Folder names must not contain Windows-invalid characters (`:`, etc.)
        // since the install dir maps to a Windows path on all platforms,
        // including Wine/Proton after download.
        let folder_name = {
            use crate::monarch_utils::monarch_fs::sanitize_install_folder_name_wine;
            sanitize_install_folder_name_wine(&game.name)
        };

        Self {
            id: NEXT_JOB_ID.fetch_add(1, Ordering::Relaxed),
            name: folder_name,
            game_id: game.id.clone(),
            path: PathBuf::from(options.folder),
            store: options.store.clone(),
            os: options.os.clone(),
            manifest: Arc::new(manifest),
            game,
        }
    }
}

static NEXT_JOB_ID: AtomicU64 = AtomicU64::new(1);

// Required for the contains method to work.
impl PartialEq for DownloadJob {
    fn eq(&self, other: &Self) -> bool {
        return self.id == other.id;
    }
}

#[derive(Debug)]
pub struct DownloadStats {
    pub ongoing: bool,
    pub parts_done: u32,
    pub parts_total: u32,
    pub game_id: String,
    pub download_speed: f32,
    pub write_speed: f32,
}

/// UI-friendly, store-agnostic description of a download job.
#[derive(Debug, Clone)]
pub struct DownloadJobInfo {
    pub id: u64,
    pub name: String,
    pub game_id: String,
    pub path: PathBuf,
    pub store: String,
    pub os: String,
    /// Whether the job has been paused by the user.
    pub paused: bool,
}

impl From<&DownloadJob> for DownloadJobInfo {
    fn from(job: &DownloadJob) -> Self {
        Self {
            id: job.id,
            name: job.name.clone(),
            game_id: job.game_id.clone(),
            path: job.path.clone(),
            store: job.store.clone(),
            os: job.os.clone(),
            paused: false,
        }
    }
}

/// Lifecycle state of a download job.
#[derive(Debug, Clone, PartialEq)]
pub enum JobState {
    Queued,
    Downloading,
    Completed,
    Failed(String),
}

/// Latest observable status of the ongoing download. Download handlers publish
/// this as progress events stream in; the UI polls it on a regular cadence.
#[derive(Debug, Clone)]
pub struct DownloadSnapshot {
    pub job: DownloadJobInfo,
    pub state: JobState,
    pub progress: DownloadProgress,
}

/// How the download handler reports a new snapshot to the shared status slot.
pub fn publish_download_status(
    status: &Arc<RwLock<Option<DownloadSnapshot>>>,
    job: &DownloadJobInfo,
    state: JobState,
    progress: &DownloadProgress,
) {
    if let Ok(mut guard) = status.write() {
        *guard = Some(DownloadSnapshot {
            job: job.clone(),
            state,
            progress: progress.clone(),
        });
    }
}

/// Publish a status update only while `job_id` is still the job the shared
/// status slot describes. Prevents a cancelled/stale download handler from
/// clobbering the status of whatever job replaced it.
fn publish_current_status(
    status: &Arc<RwLock<Option<DownloadSnapshot>>>,
    job: &DownloadJobInfo,
    job_id: u64,
    state: JobState,
    progress: &DownloadProgress,
) {
    let is_current = status
        .read()
        .ok()
        .map(|s| s.as_ref().map_or(true, |snap| snap.job.id == job_id))
        .unwrap_or(false);
    if is_current {
        publish_download_status(status, job, state, progress);
    }
}

#[derive(Debug)]
pub struct MonarchDownloader {
    ongoing: Option<DownloadJob>,
    queue: Vec<DownloadJob>,
    download_handlers: HashMap<String, Box<dyn DownloadHandler>>,
    /// Shared slot for the latest status of the ongoing download, polled by the
    /// UI. Updated by download handlers as progress events stream in.
    status: Arc<RwLock<Option<DownloadSnapshot>>>,
    /// Job ids that the user has paused. Paused jobs stay in the queue but are
    /// skipped when the downloader advances to the next job.
    paused: std::collections::HashSet<u64>,
    /// Maximum download speed in bytes/s (0 = unlimited), shared with every
    /// registered handler so updates propagate to running downloads.
    speed_limit_bps: Arc<AtomicU64>,
}

/// The main downloader struct.
///
/// This struct is responsible for managing the download queue and ongoing downloads.
/// It also handles the registration of new download handlers for different stores.
impl MonarchDownloader {
    /// Creates a new downloader instance.
    pub fn new() -> Self {
        Self {
            ongoing: None,
            queue: Vec::new(),
            download_handlers: HashMap::new(),
            status: Arc::new(RwLock::new(None)),
            paused: std::collections::HashSet::new(),
            speed_limit_bps: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Sets the maximum download speed in bytes per second; 0 means
    /// unlimited. Applies immediately to running and queued downloads.
    pub fn set_max_download_speed_bps(&mut self, bps: u64) {
        info!("Download speed limit set to {bps} B/s");
        self.speed_limit_bps.store(bps, Ordering::Relaxed);
        for handler in self.download_handlers.values() {
            handler.set_max_speed_bps(bps);
        }
    }

    /// Starts a new download job.
    pub fn start_download(&mut self, job: DownloadJob) {
        // Cancel ongoing download if there is one
        if let Some(ongoing) = &self.ongoing {
            if let Some(handler) = self.download_handlers.get(&ongoing.store) {
                let _ = handler.cancel(ongoing);
            }
            // Requeue it at the front so it resumes right after the new one.
            self.queue.insert(0, ongoing.clone());
        }

        // Remove the job from the queue if it exists there
        if self.queue.contains(&job) {
            self.queue.retain(|j| j.id != job.id);
        }

        self.paused.remove(&job.id);
        self.begin_job(job);
    }

    /// Make a job the active download immediately, requeuing the current one.
    pub fn download_now(&mut self, job_id: u64) -> Result<(), String> {
        let pos = self
            .queue
            .iter()
            .position(|j| j.id == job_id)
            .ok_or("Job is not in the queue")?;
        let job = self.queue.remove(pos);
        self.paused.remove(&job_id);

        if let Some(ongoing) = &self.ongoing {
            if let Some(handler) = self.download_handlers.get(&ongoing.store) {
                handler.cancel(ongoing)?;
            }
            self.queue.insert(0, ongoing.clone());
        }

        self.begin_job(job);
        Ok(())
    }

    /// Pause a download. The active job is stopped and moved back to the front
    /// of the queue; queued jobs are just marked paused.
    pub fn pause_download(&mut self, job_id: u64) -> Result<(), String> {
        if let Some(ongoing) = self.ongoing.take() {
            if ongoing.id == job_id {
                if let Some(handler) = self.download_handlers.get(&ongoing.store) {
                    handler.cancel(&ongoing)?;
                }
                self.paused.insert(job_id);
                self.queue.insert(0, ongoing);
                self.clear_status();
                return Ok(());
            }
            self.ongoing = Some(ongoing);
        }

        if self.queue.iter().any(|j| j.id == job_id) {
            self.paused.insert(job_id);
            return Ok(());
        }

        Err("Job not found in queue or active".to_string())
    }

    /// Resume a paused download. If nothing is currently downloading, the first
    /// unpaused job in the queue is started.
    pub fn resume_download(&mut self, job_id: u64) -> Result<(), String> {
        if !self.paused.remove(&job_id) {
            return Err("Job is not paused".to_string());
        }
        self.start_next_non_paused();
        Ok(())
    }

    /// Whether the given job is currently paused.
    pub fn is_paused(&self, job_id: u64) -> bool {
        self.paused.contains(&job_id)
    }

    /// Move a queued job to a new position. Index 0 is downloaded first.
    pub fn reorder_download(&mut self, job_id: u64, new_index: usize) -> Result<(), String> {
        let pos = self
            .queue
            .iter()
            .position(|j| j.id == job_id)
            .ok_or("Job is not in the queue")?;
        let job = self.queue.remove(pos);
        let target = new_index.min(self.queue.len());
        self.queue.insert(target, job);
        Ok(())
    }

    /// Queues a new download job.
    pub fn queue_download(&mut self, job: DownloadJob) {
        self.queue.push(job);
    }

    /// Cancels a download job.
    pub fn cancel_download(&mut self, job_id: u64) {
        if let Some(job) = &self.ongoing {
            if job.id == job_id {
                if let Some(handler) = self.download_handlers.get(&job.store) {
                    let _ = handler.cancel(job);
                }
                self.ongoing = None;
                self.clear_status();
                return;
            }
        }
        self.queue.retain(|job| job.id != job_id);
        self.paused.remove(&job_id);
    }

    /// Removes a download from the queue or stops it if it is active.
    pub fn remove_download(&mut self, job_id: u64) {
        self.cancel_download(job_id);
    }

    /// Called periodically (e.g. by the UI poller). If the active download has
    /// finished, this advances the queue to the next unpaused job.
    pub fn poll(&mut self) {
        let finished = self
            .status
            .read()
            .ok()
            .and_then(|s| s.clone())
            .map(|snap| !matches!(snap.state, JobState::Downloading))
            .unwrap_or(false);

        if self.ongoing.is_some() && finished {
            self.ongoing = None;
            self.start_next_non_paused();
        }
    }

    /// Starts the first unpaused job in the queue, if any.
    fn start_next_non_paused(&mut self) {
        if self.ongoing.is_some() {
            return;
        }
        if let Some(pos) = self.queue.iter().position(|j| !self.paused.contains(&j.id)) {
            let job = self.queue.remove(pos);
            self.begin_job(job);
        }
    }

    /// Promote a job to the active download and kick off its handler.
    fn begin_job(&mut self, job: DownloadJob) {
        let info: DownloadJobInfo = (&job).into();
        let store = job.store.clone();
        self.ongoing = Some(job);

        // Publish immediately so the UI has something to show before the first
        // progress event arrives from the background downloader.
        publish_download_status(
            &self.status,
            &info,
            JobState::Downloading,
            &DownloadProgress::default(),
        );

        if let Some(handler) = self.download_handlers.get(&store) {
            if let Some(ongoing) = self.ongoing.as_ref() {
                let _ = handler.download(ongoing);
            }
        }
    }

    /// Checks if a download is ongoing.
    pub fn is_downloading(&self) -> bool {
        self.ongoing.is_some()
    }

    /// Checks if a download for the given game is currently ongoing.
    pub fn is_downloading_game(&self, game: &MonarchGame) -> bool {
        self.ongoing
            .as_ref()
            .map(|job| job.name == game.name)
            .unwrap_or(false)
    }

    /// Number of jobs either downloading or waiting in the queue.
    pub fn pending_job_count(&self) -> usize {
        self.queue.len() + usize::from(self.ongoing.is_some())
    }

    /// Gets the ongoing download job.
    pub fn get_ongoing(&self) -> Option<&DownloadJob> {
        self.ongoing.as_ref()
    }

    /// Gets the queue of download jobs.
    pub fn get_queue(&self) -> &Vec<DownloadJob> {
        self.queue.as_ref()
    }

    /// Returns the queue as store-agnostic job infos for the UI.
    pub fn queue_job_infos(&self) -> Vec<DownloadJobInfo> {
        self.queue
            .iter()
            .map(|job| DownloadJobInfo {
                id: job.id,
                name: job.name.clone(),
                game_id: job.game_id.clone(),
                path: job.path.clone(),
                store: job.store.clone(),
                os: job.os.clone(),
                paused: self.paused.contains(&job.id),
            })
            .collect()
    }

    /// Returns a copy of the shared status slot, used by download handlers to
    /// publish progress snapshots.
    pub fn get_status_ptr(&self) -> Arc<RwLock<Option<DownloadSnapshot>>> {
        self.status.clone()
    }

    /// Clones the latest published snapshot, if any.
    pub fn status_snapshot(&self) -> Option<DownloadSnapshot> {
        self.status.read().ok().and_then(|s| s.clone())
    }

    /// Clears the published snapshot (e.g. on cancel).
    pub fn clear_status(&self) {
        if let Ok(mut status) = self.status.write() {
            *status = None;
        }
    }

    /// Checks if a given job is queued.
    pub fn is_queued(&self, game: &MonarchGame) -> bool {
        self.queue.iter().any(|job| job.name == game.name)
    }

    /// Registers a new download handler for a given store.id.
    pub fn register_download_handler(
        &mut self,
        store: String,
        handler: Box<dyn DownloadHandler>,
    ) -> Result<()> {
        if self.download_handlers.contains_key(&store) {
            bail!("Download handler for store {} already registered", store)
        }
        self.download_handlers.insert(store, handler);
        Ok(())
    }

    /// Registers the Epic Games download handler if it is not already
    /// registered. Idempotent, safe to call before every download.
    pub fn register_egs_handler(&mut self) -> Result<()> {
        if self.download_handlers.contains_key("epicgames") {
            return Ok(());
        }
        self.download_handlers.insert(
            "epicgames".to_string(),
            Box::new(EgsDownloadHandler::new(
                self.status.clone(),
                Arc::clone(&self.speed_limit_bps),
            )),
        );
        Ok(())
    }

    pub fn get_download_stats(&self) -> DownloadStats {
        DownloadStats {
            ongoing: self.ongoing.is_some(),
            parts_done: 0,
            parts_total: 0,
            game_id: self.ongoing.as_ref().unwrap().game_id.clone(),
            download_speed: 0.0,
            write_speed: 0.0,
        }
    }
}

/// Adds a successfully installed game back into the library, deriving the
/// executable path and launch arguments from the store manifest. If the game
/// is already present (e.g. a re-download or repair), its properties are
/// updated in place instead.
async fn add_installed_game_to_library(
    game: MonarchGame,
    install_dir: PathBuf,
    launch_exe: String,
    launch_command: String,
    build_version: String,
    install_bytes: u64,
) {
    let mut installed = game;
    installed.is_installed = true;
    installed.managed_by_monarch = true;
    installed.properties.install_dir = install_dir.to_string_lossy().to_string();
    installed.properties.size_on_disk = install_bytes;

    if !build_version.is_empty() {
        installed.properties.version = build_version;
    }
    if !launch_exe.is_empty() {
        installed.executable_path =
            Some(install_dir.join(&launch_exe).to_string_lossy().to_string());
    }
    if !launch_command.is_empty() {
        installed.launch_args = Some(launch_command);
    }

    let already_installed = MONARCH_STATE
        .read()
        .ok()
        .map(|state| state.get_game(&installed.id).is_some())
        .unwrap_or(false);

    let result = if already_installed {
        crate::monarch_library::library::update_game_properties(&installed).await
    } else {
        crate::monarch_library::library::add_game(&installed).await
    };

    if let Err(e) = result {
        error!(
            "egs_download::Failed to add {} to library | Err: {e}",
            installed.name
        );
    }
}

/// Downloads Epic Games titles using `monarch_egs` directly, no external CLI
/// required. The store-specific manifest is carried in the [`DownloadJob`].
/// Progress events are published to the shared status slot for the UI to poll.
#[derive(Debug)]
pub struct EgsDownloadHandler {
    status: Arc<RwLock<Option<DownloadSnapshot>>>,
    /// Per-job cancellation handles keyed by job id. Each resume bumps
    /// `generation` so a stale run's cleanup cannot remove the active handle.
    cancels: Arc<Mutex<HashMap<u64, CancelSlot>>>,
    /// Join handles for the latest run of each job, so a resume can wait for
    /// the paused run to exit before hashing/writing the same install dir.
    runs: Arc<Mutex<HashMap<u64, RunSlot>>>,
    /// Maximum download speed in bytes/s (0 = unlimited), shared with the
    /// downloader so limit changes apply to running downloads immediately.
    speed_limit_bps: Arc<AtomicU64>,
}

#[derive(Debug)]
struct CancelSlot {
    generation: u64,
    cancel: Arc<std::sync::atomic::AtomicBool>,
}

struct RunSlot {
    generation: u64,
    handle: tokio::task::JoinHandle<()>,
}

impl std::fmt::Debug for RunSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunSlot")
            .field("generation", &self.generation)
            .finish_non_exhaustive()
    }
}

impl EgsDownloadHandler {
    fn new(status: Arc<RwLock<Option<DownloadSnapshot>>>, speed_limit_bps: Arc<AtomicU64>) -> Self {
        Self {
            status,
            cancels: Arc::new(Mutex::new(HashMap::new())),
            runs: Arc::new(Mutex::new(HashMap::new())),
            speed_limit_bps,
        }
    }
}

impl DownloadHandler for EgsDownloadHandler {
    fn download(&self, job: &DownloadJob) -> Result<(), String> {
        let manifest: monarch_egs::Manifest = job
            .manifest
            .downcast_ref::<monarch_egs::Manifest>()
            .ok_or("EgsDownloadHandler::download() Job manifest is not a monarch_egs Manifest!")?
            .clone();

        let install_dir = job.path.join(&job.name);
        let game_name = job.name.clone();
        let job_id = job.id;
        let job_info: DownloadJobInfo = job.into();
        let status = self.status.clone();
        let cancels = self.cancels.clone();
        let runs = self.runs.clone();

        // Capture the original game plus the manifest launch info before the
        // manifest is moved into the DownloadManager, so a finished download
        // can be registered in the library as an installed game.
        let original_game = job.game.clone();
        let launch_exe = manifest.launch_exe().to_string();
        let launch_command = manifest.launch_command().to_string();
        let build_version = manifest.build_version().to_string();

        let cancel = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let generation = {
            let mut map = self.cancels.lock().unwrap();
            // Stop any previous run for this job id (pause/resume reuse ids).
            if let Some(prev) = map.get_mut(&job_id) {
                prev.cancel.store(true, Ordering::Relaxed);
            }
            let generation = map.get(&job_id).map(|s| s.generation + 1).unwrap_or(1);
            map.insert(
                job_id,
                CancelSlot {
                    generation,
                    cancel: cancel.clone(),
                },
            );
            generation
        };

        // Take the previous JoinHandle so this run can await it before touching
        // the install directory (avoids overlapping prepare hashes).
        let previous_run = self
            .runs
            .lock()
            .unwrap()
            .remove(&job_id)
            .map(|slot| slot.handle);

        let speed_limit = Arc::clone(&self.speed_limit_bps);

        let handle = tokio::spawn(async move {
            if let Some(previous) = previous_run {
                // Previous run was cancelled above; wait so we do not race on
                // the same install dir / cancel slot.
                let _ = previous.await;
            }

            let manager = DownloadManager::new(manifest, install_dir.clone())
                .with_cancel_handle(cancel.clone())
                .with_max_speed_bps(speed_limit);
            let (tx, mut rx) = tokio::sync::mpsc::channel(256);

            let report_task = tokio::spawn(async move { manager.download_with_events(tx).await });

            let last_progress: Arc<Mutex<Option<DownloadProgress>>> = Arc::new(Mutex::new(None));

            let event_last_progress = last_progress.clone();
            let event_status = status.clone();
            let event_job_info = job_info.clone();
            let event_job_id = job_id;

            let event_task_cancel = cancel.clone();
            let event_task = tokio::spawn(async move {
                while let Some(event) = rx.recv().await {
                    if event_task_cancel.load(Ordering::Relaxed) {
                        // The job was cancelled: stop publishing progress events.
                        continue;
                    }
                    match event {
                        DownloadEvent::Progress(progress) => {
                            if let Ok(mut last) = event_last_progress.lock() {
                                *last = Some(progress.clone());
                            }
                            publish_current_status(
                                &event_status,
                                &event_job_info,
                                event_job_id,
                                JobState::Downloading,
                                &progress,
                            );
                            debug!(
                                "egs_download::Progress {:.1}% ({}/{} files, {}/{} chunks)",
                                progress.fraction() * 100.0,
                                progress.files_completed,
                                progress.total_files,
                                progress.chunks_completed,
                                progress.total_chunks,
                            );
                        }
                        DownloadEvent::ChunkDownloaded { .. } => {}
                        DownloadEvent::FileWritten { filename, bytes } => {
                            info!("egs_download::Wrote {filename} ({bytes} bytes)");
                        }
                        DownloadEvent::FileSkipped { filename } => {
                            info!("egs_download::Skipped {filename}");
                        }
                        DownloadEvent::FileFailed { filename, error } => {
                            error!("egs_download::Failed {filename} | {error}");
                        }
                    }
                }
            });

            let outcome = report_task.await;
            let _ = event_task.await;

            let final_progress = last_progress.lock().unwrap().clone().unwrap_or_default();
            let cancelled = cancel.load(Ordering::Relaxed);

            match outcome {
                Ok(Ok(report)) => {
                    if cancelled {
                        info!("egs_download::Cancelled {game_name}");
                    } else {
                        info!(
                            "Finished installing {game_name}: {} files written, {} chunks downloaded, {} bytes",
                            report.files_written,
                            report.chunks_downloaded,
                            report.install_bytes,
                        );

                        // Register the game in the library *before* publishing
                        // completion, so the UI's completion handling can rely
                        // on the game already being present (and installed).
                        add_installed_game_to_library(
                            original_game,
                            install_dir,
                            launch_exe,
                            launch_command,
                            build_version,
                            report.install_bytes,
                        )
                        .await;

                        publish_current_status(
                            &status,
                            &job_info,
                            job_id,
                            JobState::Completed,
                            &final_progress,
                        );
                    }
                }
                Ok(Err(e)) => {
                    if cancelled {
                        info!("egs_download::Cancelled {game_name}");
                    } else {
                        publish_current_status(
                            &status,
                            &job_info,
                            job_id,
                            JobState::Failed(e.to_string()),
                            &final_progress,
                        );
                        error!("egs_download::Failed to install {game_name} | Err: {e}");
                    }
                }
                Err(e) => {
                    publish_current_status(
                        &status,
                        &job_info,
                        job_id,
                        JobState::Failed(format!("download task panicked: {e}")),
                        &final_progress,
                    );
                    error!("egs_download::Download task panicked for {game_name} | Err: {e}");
                }
            }

            // Only clear slots if we are still the active generation — a newer
            // resume already owns them and must not be wiped by this cleanup.
            if let Ok(mut cancels) = cancels.lock() {
                if cancels
                    .get(&job_id)
                    .is_some_and(|slot| slot.generation == generation)
                {
                    cancels.remove(&job_id);
                }
            }
            if let Ok(mut runs) = runs.lock() {
                if runs
                    .get(&job_id)
                    .is_some_and(|slot| slot.generation == generation)
                {
                    runs.remove(&job_id);
                }
            }
        });

        self.runs.lock().unwrap().insert(
            job_id,
            RunSlot {
                generation,
                handle,
            },
        );

        Ok(())
    }

    fn cancel(&self, job: &DownloadJob) -> Result<(), String> {
        if let Some(slot) = self.cancels.lock().unwrap().get(&job.id) {
            slot.cancel.store(true, Ordering::Relaxed);
        }
        Ok(())
    }

    fn is_downloading(&self) -> bool {
        false
    }

    fn set_max_speed_bps(&self, bps: u64) {
        self.speed_limit_bps.store(bps, Ordering::Relaxed);
    }
}
