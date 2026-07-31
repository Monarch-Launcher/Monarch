mod view;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueueStatus {
    Active,
    Queued,
    Paused,
}

#[derive(Clone, Debug)]
pub struct QueuedItem {
    pub id: usize,
    pub name: String,
    pub store: String,
    pub platform: String,
    pub progress: f32,
    pub status: QueueStatus,
    pub size_label: String,
}

const MOCK_ARTWORK_PATH: &str = "";

#[derive(Clone, Debug)]
pub struct ActiveDownload {
    pub id: usize,
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
}

#[derive(Clone, Debug)]
pub enum Message {
    Tick,
    SelectQueueItem(usize),
}

pub struct DownloadPage {
    pub queue: Vec<QueuedItem>,
    pub active: Option<ActiveDownload>,
    pub selected_id: usize,
    /// Download speed samples (MB/s), newest at the end.
    pub download_history: Vec<f32>,
    /// Write speed samples (MB/s), newest at the end.
    pub write_history: Vec<f32>,
    tick: u64,
}

const HISTORY_LEN: usize = 128;

impl DownloadPage {
    pub fn new() -> Self {
        let queue = mock_queue();
        let active = mock_active(0);
        let mut page = Self {
            queue,
            active: Some(active),
            selected_id: 0,
            download_history: Vec::with_capacity(HISTORY_LEN),
            write_history: Vec::with_capacity(HISTORY_LEN),
            tick: 0,
        };
        // Seed history so the graph looks populated on first open.
        for i in 0..HISTORY_LEN {
            let (dl, wr) = mock_speeds(i as u64);
            page.download_history.push(dl);
            page.write_history.push(wr);
        }
        page
    }

    pub fn is_downloading(&self) -> bool {
        self.active.is_some()
            && self
                .queue
                .iter()
                .any(|item| item.status == QueueStatus::Active)
    }

    pub fn current_download_speed(&self) -> Option<f64> {
        self.active
            .as_ref()
            .filter(|_| self.is_downloading())
            .map(|a| a.download_speed_mbps)
    }

    pub fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::Tick => {
                self.tick = self.tick.wrapping_add(1);
                let (dl, wr) = mock_speeds(self.tick);

                if let Some(active) = &mut self.active {
                    active.download_speed_mbps = dl as f64;
                    active.write_speed_mbps = wr as f64;
                    // Gentle mock progress crawl.
                    active.progress = (active.progress + 0.0015).min(0.99);
                    active.eta_secs = mock_eta_secs(active.progress, active.download_speed_mbps);
                    if let Some(item) = self.queue.iter_mut().find(|q| q.id == active.id) {
                        item.progress = active.progress;
                    }
                }

                push_sample(&mut self.download_history, dl);
                push_sample(&mut self.write_history, wr);
                iced::Task::none()
            }
            Message::SelectQueueItem(id) => {
                self.selected_id = id;
                if let Some(item) = self.queue.iter().find(|q| q.id == id) {
                    let download_speed_mbps = if item.status == QueueStatus::Active {
                        self.download_history.last().copied().unwrap_or(0.0) as f64
                    } else {
                        0.0
                    };
                    let write_speed_mbps = if item.status == QueueStatus::Active {
                        self.write_history.last().copied().unwrap_or(0.0) as f64
                    } else {
                        0.0
                    };
                    self.active = Some(ActiveDownload {
                        id: item.id,
                        name: item.name.clone(),
                        store: item.store.clone(),
                        platform: item.platform.clone(),
                        location: format!(
                            "/home/user/Games/{}",
                            item.name.replace(' ', "")
                        ),
                        artwork_path: MOCK_ARTWORK_PATH.to_string(),
                        download_speed_mbps,
                        write_speed_mbps,
                        eta_secs: if item.status == QueueStatus::Active {
                            mock_eta_secs(item.progress, download_speed_mbps)
                        } else {
                            0
                        },
                        progress: item.progress,
                        downloaded_label: format_progress_bytes(item.progress, &item.size_label),
                        total_label: item.size_label.clone(),
                    });
                }
                iced::Task::none()
            }
        }
    }
}

impl Default for DownloadPage {
    fn default() -> Self {
        Self::new()
    }
}

fn push_sample(history: &mut Vec<f32>, sample: f32) {
    if history.len() >= HISTORY_LEN {
        history.remove(0);
    }
    history.push(sample);
}

fn mock_speeds(tick: u64) -> (f32, f32) {
    // Jagged mock samples so the chart reads as hard angular peaks, not a soft sine.
    let step = (tick % 7) as f32;
    let burst = match tick % 11 {
        0 | 1 => 9.0,
        2 => -4.0,
        5 => 6.5,
        8 => -5.5,
        _ => 0.0,
    };
    let download = 16.0 + step * 1.8 + burst + ((tick % 3) as f32) * 1.2;
    let write_burst = match (tick + 3) % 9 {
        0 => 5.0,
        4 => -3.5,
        7 => 4.0,
        _ => 0.0,
    };
    let write = download * 0.68 + write_burst + ((tick % 4) as f32) * 0.9;
    (download.max(0.5), write.max(0.3))
}

fn mock_queue() -> Vec<QueuedItem> {
    vec![
        QueuedItem {
            id: 0,
            name: "Hollow Knight: Silksong".into(),
            store: "Steam".into(),
            platform: "Linux".into(),
            progress: 0.42,
            status: QueueStatus::Active,
            size_label: "18.4 GiB".into(),
        },
        QueuedItem {
            id: 1,
            name: "Hades II".into(),
            store: "Epic Games".into(),
            platform: "Windows".into(),
            progress: 0.0,
            status: QueueStatus::Queued,
            size_label: "22.1 GiB".into(),
        },
        QueuedItem {
            id: 2,
            name: "Balatro".into(),
            store: "Steam".into(),
            platform: "Linux".into(),
            progress: 0.0,
            status: QueueStatus::Queued,
            size_label: "312 MiB".into(),
        },
        QueuedItem {
            id: 3,
            name: "Celeste".into(),
            store: "itch.io".into(),
            platform: "Linux".into(),
            progress: 0.67,
            status: QueueStatus::Paused,
            size_label: "1.2 GiB".into(),
        },
    ]
}

fn mock_active(id: usize) -> ActiveDownload {
    let item = &mock_queue()[id];
    let download_speed_mbps = 24.6;
    ActiveDownload {
        id: item.id,
        name: item.name.clone(),
        store: item.store.clone(),
        platform: item.platform.clone(),
        location: "/home/user/Games/HollowKnightSilksong".into(),
        artwork_path: MOCK_ARTWORK_PATH.to_string(),
        download_speed_mbps,
        write_speed_mbps: 17.2,
        eta_secs: mock_eta_secs(item.progress, download_speed_mbps),
        progress: item.progress,
        downloaded_label: "7.7 GiB".into(),
        total_label: item.size_label.clone(),
    }
}

fn mock_eta_secs(progress: f32, speed_mbps: f64) -> u64 {
    // ~18.4 GiB mock payload; ETA drifts with speed + remaining progress.
    let total_mb = 18.4 * 1024.0;
    let remaining_mb = total_mb * (1.0 - progress).max(0.01) as f64;
    let speed = speed_mbps.max(0.5);
    (remaining_mb / speed).round().max(1.0) as u64
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
    // Display-only mock label derived from progress fraction.
    format!("{:.0}% of {}", (progress * 100.0).round(), total_label)
}
