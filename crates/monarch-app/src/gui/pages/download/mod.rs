mod view;

use std::time::Instant;

use monarch_core::monarch_utils::monarch_downloader::{
    DownloadJobInfo, DownloadSnapshot, JobState, MonarchDownloader,
};
use monarch_core::monarch_utils::monarch_state::MONARCH_STATE;
use monarch_egs::{DownloadPhase, DownloadProgress};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueueStatus {
    Active,
    Queued,
    Paused,
}

#[derive(Clone, Debug)]
pub struct QueuedItem {
    pub id: u64,
    pub name: String,
    pub store: String,
    pub platform: String,
    pub progress: f32,
    pub status: QueueStatus,
    /// True while the active job is scanning existing files before transfer.
    pub verifying: bool,
    pub size_label: String,
    pub location: String,
    pub artwork_path: String,
}

#[derive(Clone, Debug)]
pub struct ActiveDownload {
    pub id: u64,
    pub name: String,
    pub store: String,
    pub platform: String,
    pub location: String,
    pub artwork_path: String,
    pub download_speed_mbps: f64,
    pub write_speed_mbps: f64,
    pub eta_secs: u64,
    pub progress: f32,
    pub downloaded_label: String,
    pub total_label: String,
    /// True while comparing on-disk files against the manifest.
    pub verifying: bool,
    /// Files checked / total during the verify pass (for status text).
    pub verify_label: Option<String>,
}

#[derive(Clone, Debug)]
pub enum Message {
    Tick,
    SelectQueueItem(u64),
    PauseJob(u64),
    ResumeJob(u64),
    RemoveJob(u64),
    DownloadNow(u64),
    DragStarted(u64),
    DragMoved {
        y: f32,
    },
    DragEnded,
    /// A download just finished; carries the installed game's id so the
    /// library can upsert that one entry without a full store refresh.
    DownloadFinished(String),
    /// Drives smooth graph scrolling between samples by triggering a view
    /// rebuild; no download state advances on this message.
    AnimationFrame,
}

/// State for an in-progress drag-to-reorder gesture.
#[derive(Clone, Debug)]
pub struct DragState {
    pub id: u64,
}

pub struct DownloadPage {
    pub queue: Vec<QueuedItem>,
    pub active: Option<ActiveDownload>,
    pub selected_id: u64,
    /// Download speed samples (MB/s), newest at the end.
    pub download_history: Vec<f32>,
    /// Write speed samples (MB/s), newest at the end.
    pub write_history: Vec<f32>,
    /// Job the current graph samples belong to; the history is cleared
    /// whenever a different job starts so each download graphs from zero.
    graph_job: Option<u64>,
    /// When the newest graph samples were taken, used to interpolate the
    /// graph's scroll position between samples.
    last_sample_at: Option<Instant>,
    pub drag: Option<DragState>,
    tick: u64,
    /// Whether a download was active on the previous poll, used to detect the
    /// Downloading -> Completed transition and trigger a library refresh.
    was_downloading: bool,
    last_write_sample: Option<(Instant, u64)>,
    /// Exponentially smoothed download speed (bytes/s) feeding the ETA.
    eta_speed_bps: f64,
    /// When the displayed ETA was last published and its value; the estimate
    /// only advances every [`ETA_UPDATE_INTERVAL_SECS`].
    last_eta: Option<(Instant, u64)>,
}

pub const HISTORY_LEN: usize = 128;
/// Interval between graph samples in seconds. Must match the download poll
/// tick in `gui::App::subscription`.
pub const SAMPLE_INTERVAL_SECS: f32 = 0.25;
/// Weight of each new sample in the graph's exponential moving average.
/// Lower values smooth out small speed fluctuations at the cost of slower
/// reaction to real changes.
const SMOOTHING_ALPHA: f32 = 0.35;
/// Weight of each new speed sample in the ETA's exponential moving average.
/// Much lower than the graph smoothing so a single fast or slow poll doesn't
/// swing the estimate around.
const ETA_SMOOTHING_ALPHA: f64 = 0.15;
/// Minimum time between visible ETA updates, so the countdown advances in
/// calm steps instead of flickering on every poll.
const ETA_UPDATE_INTERVAL_SECS: f64 = 1.5;
/// Height of one queue item (must match the fixed height used in the view).
pub const QUEUE_ITEM_HEIGHT: f32 = 84.0;
/// Vertical gap between queue items (must match the view's spacing).
pub const QUEUE_ITEM_SPACING: f32 = 10.0;

impl DownloadPage {
    pub fn new() -> Self {
        Self {
            queue: Vec::new(),
            active: None,
            selected_id: 0,
            download_history: Vec::with_capacity(HISTORY_LEN),
            write_history: Vec::with_capacity(HISTORY_LEN),
            graph_job: None,
            last_sample_at: None,
            drag: None,
            tick: 0,
            was_downloading: false,
            last_write_sample: None,
            eta_speed_bps: 0.0,
            last_eta: None,
        }
    }

    pub fn is_downloading(&self) -> bool {
        poll_downloader(|downloader| {
            downloader
                .status_snapshot()
                .map_or(false, |snap| snap.state == JobState::Downloading)
        })
        .unwrap_or(false)
    }

    pub fn current_download_speed(&self) -> Option<f64> {
        if self.is_downloading() {
            self.active.as_ref().map(|a| a.download_speed_mbps)
        } else {
            None
        }
    }

    pub fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::Tick => {
                self.tick = self.tick.wrapping_add(1);

                // Capture a completed job *before* advancing the queue. `poll()`
                // may immediately start the next download and overwrite the
                // Completed snapshot, which would hide the finish event.
                let finished_game_id = if self.was_downloading {
                    poll_snapshot().and_then(|snap| match snap.state {
                        JobState::Completed => Some(snap.job.game_id),
                        _ => None,
                    })
                } else {
                    None
                };

                // Advance the backend queue if the active download finished.
                let _ = mutate_downloader(|d| d.poll());

                let snapshot = poll_snapshot();
                let queue_jobs = poll_downloader(|d| d.queue_job_infos()).unwrap_or_default();

                self.rebuild_queue(&queue_jobs, snapshot.as_ref());

                let is_downloading =
                    matches!(&snapshot, Some(snap) if snap.state == JobState::Downloading);
                self.was_downloading = is_downloading;

                match &snapshot {
                    Some(snap) if is_downloading => {
                        // Always track the real download on the speed graph,
                        // even when the user is viewing a queued item.
                        if self.graph_job != Some(snap.job.id) {
                            // A new download started: start its graph from zero.
                            self.graph_job = Some(snap.job.id);
                            self.download_history.clear();
                            self.write_history.clear();
                            self.eta_speed_bps = 0.0;
                            self.last_eta = None;
                        }
                        let write_speed = self.sample_write_speed(snap);
                        let eta_secs = self.next_eta(snap);
                        let active = build_active(snap, write_speed, eta_secs);
                        push_smoothed(
                            &mut self.download_history,
                            active.download_speed_mbps as f32,
                        );
                        push_smoothed(&mut self.write_history, active.write_speed_mbps as f32);
                        self.last_sample_at = Some(Instant::now());

                        // Auto-follow the ongoing download unless the user has
                        // selected a specific queued item.
                        let auto_follow = self.selected_id == 0 || self.selected_id == snap.job.id;
                        if auto_follow {
                            self.selected_id = snap.job.id;
                            self.active = Some(active);
                        }
                    }
                    _ => {
                        // Completed / failed / no snapshot: clear the hero.
                        self.selected_id = 0;
                        self.active = None;
                        self.last_write_sample = None;
                        self.graph_job = None;
                        self.last_sample_at = None;
                        self.eta_speed_bps = 0.0;
                        self.last_eta = None;
                    }
                }

                if let Some(game_id) = finished_game_id {
                    return iced::Task::done(Message::DownloadFinished(game_id));
                }
                iced::Task::none()
            }
            Message::SelectQueueItem(id) => {
                self.selected_id = id;
                if let Some(item) = self.queue.iter().find(|q| q.id == id) {
                    // The active job is already shown by Tick; only rebuild the
                    // hero when the user picks a queued item.
                    let is_active = item.status == QueueStatus::Active
                        && self.active.as_ref().map(|a| a.id) == Some(id);
                    if !is_active {
                        self.active = Some(ActiveDownload {
                            id: item.id,
                            name: item.name.clone(),
                            store: item.store.clone(),
                            platform: item.platform.clone(),
                            location: item.location.clone(),
                            artwork_path: item.artwork_path.clone(),
                            download_speed_mbps: 0.0,
                            write_speed_mbps: 0.0,
                            eta_secs: 0,
                            progress: item.progress,
                            downloaded_label: format_progress_bytes(
                                item.progress,
                                &item.size_label,
                            ),
                            total_label: item.size_label.clone(),
                            verifying: item.verifying,
                            verify_label: None,
                        });
                    }
                }
                iced::Task::none()
            }
            Message::PauseJob(id) => {
                let _ = mutate_downloader(|d| d.pause_download(id));
                self.refresh();
                iced::Task::none()
            }
            Message::ResumeJob(id) => {
                let _ = mutate_downloader(|d| d.resume_download(id));
                self.refresh();
                iced::Task::none()
            }
            Message::RemoveJob(id) => {
                let _ = mutate_downloader(|d| d.remove_download(id));
                self.refresh();
                iced::Task::none()
            }
            Message::DownloadNow(id) => {
                let _ = mutate_downloader(|d| d.download_now(id));
                self.refresh();
                iced::Task::none()
            }
            Message::DragStarted(id) => {
                self.drag = Some(DragState { id });
                iced::Task::none()
            }
            Message::DragMoved { y } => {
                let Some(drag) = &self.drag else {
                    return iced::Task::none();
                };
                let id = drag.id;
                if self.queue.len() < 2 {
                    return iced::Task::none();
                }
                let has_active = matches!(
                    self.queue.first().map(|q| q.status),
                    Some(QueueStatus::Active)
                );
                let stride = QUEUE_ITEM_HEIGHT + QUEUE_ITEM_SPACING;
                let raw = ((y / stride).floor().max(0.0)) as usize;
                let min_target = usize::from(has_active);
                let target = raw.clamp(min_target, self.queue.len() - 1);
                let current = self.queue.iter().position(|q| q.id == id);
                if let Some(current) = current {
                    // The active download is pinned to the top and cannot move.
                    if current != 0 || !has_active {
                        if target != current {
                            move_item(&mut self.queue, current, target);
                            // The backend queue excludes the active job, so UI
                            // indices are one ahead of backend indices while a
                            // download is active.
                            let backend_index = target.saturating_sub(usize::from(has_active));
                            let _ = mutate_downloader(|d| d.reorder_download(id, backend_index));
                        }
                    }
                }
                iced::Task::none()
            }
            Message::DragEnded => {
                self.drag = None;
                iced::Task::none()
            }
            // Handled by the parent App (upserts the installed game); this arm
            // only exists so the page-level match stays exhaustive.
            Message::DownloadFinished(_) => iced::Task::none(),
            // Purely a redraw trigger for the graph; nothing to do here.
            Message::AnimationFrame => iced::Task::none(),
        }
    }

    /// Re-read the snapshot + queue from the backend so the list reflects an
    /// action immediately instead of waiting for the next [`Message::Tick`].
    fn refresh(&mut self) {
        let snapshot = poll_snapshot();
        let queue_jobs = poll_downloader(|d| d.queue_job_infos()).unwrap_or_default();
        self.rebuild_queue(&queue_jobs, snapshot.as_ref());

        match &snapshot {
            Some(snap) if snap.state == JobState::Downloading => {
                self.selected_id = snap.job.id;
                // Keep the published ETA so acting on a job doesn't make the
                // estimate jump; fall back to the raw value when there isn't
                // one yet.
                let eta_secs = self
                    .last_eta
                    .map_or_else(|| raw_eta_secs(&snap.progress), |(_, secs)| secs);
                self.active = Some(build_active(snap, 0.0, eta_secs));
            }
            _ => {
                self.selected_id = 0;
                self.active = None;
                self.last_write_sample = None;
            }
        }
    }

    fn rebuild_queue(
        &mut self,
        queue_jobs: &[DownloadJobInfo],
        snapshot: Option<&DownloadSnapshot>,
    ) {
        let mut items: Vec<QueuedItem> = Vec::with_capacity(queue_jobs.len() + 1);

        if let Some(snap) = snapshot {
            if snap.state == JobState::Downloading {
                let job = &snap.job;
                let verifying = snap.progress.phase == DownloadPhase::VerifyingExisting;
                items.push(QueuedItem {
                    id: job.id,
                    name: job.name.clone(),
                    store: job.store.clone(),
                    platform: job.os.clone(),
                    progress: progress_fraction(&snap.progress),
                    status: QueueStatus::Active,
                    verifying,
                    size_label: if verifying {
                        "Checking files…".into()
                    } else {
                        format_bytes(snap.progress.total_download_bytes)
                    },
                    location: install_dir(job),
                    artwork_path: artwork_path_for(&job.game_id),
                });
            }
        }

        for job in queue_jobs {
            items.push(QueuedItem {
                id: job.id,
                name: job.name.clone(),
                store: job.store.clone(),
                platform: job.os.clone(),
                progress: 0.0,
                status: if job.paused {
                    QueueStatus::Paused
                } else {
                    QueueStatus::Queued
                },
                verifying: false,
                size_label: "—".into(),
                location: install_dir(job),
                artwork_path: artwork_path_for(&job.game_id),
            });
        }

        self.queue = items;
    }

    /// Fraction of the current sampling interval that has elapsed, in
    /// `[0, 1]`; used to slide the graph smoothly between samples.
    pub fn graph_scroll_phase(&self) -> f32 {
        match self.last_sample_at {
            Some(at) => (at.elapsed().as_secs_f32() / SAMPLE_INTERVAL_SECS).min(1.0),
            None => 0.0,
        }
    }

    /// Compute write speed (MB/s) from the delta in decompressed bytes since
    /// the previous poll.
    fn sample_write_speed(&mut self, snap: &DownloadSnapshot) -> f64 {
        let now = Instant::now();
        let bytes = snap.progress.decompressed_bytes;

        let mbps = match self.last_write_sample {
            Some((then, last_bytes)) => {
                let dt = now.duration_since(then).as_secs_f64();
                if dt > 0.0 && dt <= 2.0 {
                    bytes.saturating_sub(last_bytes) as f64 / dt / 1_000_000.0
                } else {
                    0.0
                }
            }
            None => 0.0,
        };

        self.last_write_sample = Some((now, bytes));
        mbps
    }

    /// Advance the ETA's smoothed speed and return the ETA to display this
    /// tick. Speed is averaged before dividing (rather than averaging ETAs)
    /// so fast periods are weighted correctly, then the result is only
    /// republished every [`ETA_UPDATE_INTERVAL_SECS`] to keep it readable.
    fn next_eta(&mut self, snap: &DownloadSnapshot) -> u64 {
        let progress = &snap.progress;
        let speed_bps = progress.download_speed_bps;
        self.eta_speed_bps = if self.eta_speed_bps > 0.0 {
            self.eta_speed_bps + ETA_SMOOTHING_ALPHA * (speed_bps - self.eta_speed_bps)
        } else {
            speed_bps
        };

        let remaining_bytes = progress
            .total_download_bytes
            .saturating_sub(progress.downloaded_bytes);
        let eta_secs = if self.eta_speed_bps > 0.0 {
            (remaining_bytes as f64 / self.eta_speed_bps).round() as u64
        } else {
            0
        };

        let due = match self.last_eta {
            Some((at, _)) => at.elapsed().as_secs_f64() >= ETA_UPDATE_INTERVAL_SECS,
            None => true,
        };
        if due {
            self.last_eta = Some((Instant::now(), eta_secs));
        }
        self.last_eta.map_or(eta_secs, |(_, secs)| secs)
    }
}

impl Default for DownloadPage {
    fn default() -> Self {
        Self::new()
    }
}

/// Read some state out of the global downloader, locking both shared guards in
/// a consistent order (MONARCH_STATE, then the downloader).
fn poll_downloader<T>(f: impl FnOnce(&MonarchDownloader) -> T) -> Option<T> {
    let state = MONARCH_STATE.read().ok()?;
    let downloader = state.get_downloader_ptr();
    let downloader = downloader.read().ok()?;
    Some(f(&downloader))
}

/// Mutate the global downloader, locking in the same consistent order.
fn mutate_downloader<T>(f: impl FnOnce(&mut MonarchDownloader) -> T) -> Option<T> {
    let state = MONARCH_STATE.read().ok()?;
    let downloader = state.get_downloader_ptr();
    let mut downloader = downloader.write().ok()?;
    Some(f(&mut downloader))
}

/// Move `from` to `to`, shifting the rest in between.
fn move_item<T: Clone>(items: &mut Vec<T>, from: usize, to: usize) {
    let item = items.remove(from);
    items.insert(to.min(items.len()), item);
}

fn poll_snapshot() -> Option<DownloadSnapshot> {
    poll_downloader(|d| d.status_snapshot()).flatten()
}

fn install_dir(job: &DownloadJobInfo) -> String {
    job.path.join(&job.name).to_string_lossy().to_string()
}

fn artwork_path_for(game_id: &str) -> String {
    MONARCH_STATE
        .read()
        .ok()
        .and_then(|s| s.get_game(game_id))
        .map(|g| g.artwork_path)
        .unwrap_or_default()
}

fn progress_fraction(progress: &DownloadProgress) -> f32 {
    if progress.total_download_bytes > 0 {
        (progress.downloaded_bytes as f64 / progress.total_download_bytes as f64).min(1.0) as f32
    } else {
        progress.fraction() as f32
    }
}

fn build_active(snap: &DownloadSnapshot, write_speed_mbps: f64, eta_secs: u64) -> ActiveDownload {
    let progress = &snap.progress;
    let verifying = progress.phase == DownloadPhase::VerifyingExisting;
    let download_speed_mbps = if verifying {
        0.0
    } else {
        progress.download_speed_bps / 1_000_000.0
    };

    let (downloaded_label, total_label, verify_label) = if verifying {
        let checked = progress.files_completed;
        let total = progress.total_files;
        (
            format!("{checked} files"),
            format!("{total} files"),
            Some(format!("Checking existing files… {checked} / {total}")),
        )
    } else {
        (
            format_bytes(progress.downloaded_bytes),
            format_bytes(progress.total_download_bytes),
            None,
        )
    };

    ActiveDownload {
        id: snap.job.id,
        name: snap.job.name.clone(),
        store: snap.job.store.clone(),
        platform: snap.job.os.clone(),
        location: install_dir(&snap.job),
        artwork_path: artwork_path_for(&snap.job.game_id),
        download_speed_mbps,
        write_speed_mbps: if verifying { 0.0 } else { write_speed_mbps },
        eta_secs: if verifying { 0 } else { eta_secs },
        progress: progress_fraction(progress),
        downloaded_label,
        total_label,
        verifying,
        verify_label,
    }
}

/// ETA from the instantaneous reported speed, before any smoothing.
fn raw_eta_secs(progress: &DownloadProgress) -> u64 {
    let remaining_bytes = progress
        .total_download_bytes
        .saturating_sub(progress.downloaded_bytes);
    if progress.download_speed_bps > 0.0 {
        (remaining_bytes as f64 / progress.download_speed_bps).round() as u64
    } else {
        0
    }
}

fn push_sample(history: &mut Vec<f32>, sample: f32) {
    if history.len() >= HISTORY_LEN {
        history.remove(0);
    }
    history.push(sample);
}

/// Push a new sample smoothed with an exponential moving average over the
/// recent history, so minor measurement noise doesn't jitter the graph.
fn push_smoothed(history: &mut Vec<f32>, sample: f32) {
    let smoothed = match history.last().copied() {
        Some(prev) => prev + SMOOTHING_ALPHA * (sample - prev),
        None => sample,
    };
    push_sample(history, smoothed);
}

/// `mm:ss` under one hour, otherwise `hh:mm:ss`.
pub fn format_eta(secs: u64) -> String {
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    if hours > 0 {
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

fn format_progress_bytes(progress: f32, total_label: &str) -> String {
    format!("{:.0}% of {}", (progress * 100.0).round(), total_label)
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;
    let value = bytes as f64;
    if value >= TB {
        format!("{:.2} TiB", value / TB)
    } else if value >= GB {
        format!("{:.2} GiB", value / GB)
    } else if value >= MB {
        format!("{:.1} MiB", value / MB)
    } else if value >= KB {
        format!("{:.1} KiB", value / KB)
    } else {
        format!("{bytes} B")
    }
}
