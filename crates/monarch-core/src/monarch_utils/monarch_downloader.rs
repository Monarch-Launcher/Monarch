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

use std::{collections::HashMap, path::PathBuf, sync::Arc};
use std::any::Any;
use std::fmt::Debug;
use anyhow::{bail, Result};

use crate::monarch_games::{monarchgame::MonarchGame, stores::DownloadOptions};

pub trait DownloadHandler: Debug + Send + Sync {
    fn download(&self, job: &DownloadJob) -> Result<(), String>;
    fn cancel(&self, job: &DownloadJob) -> Result<(), String>;
    fn is_downloading(&self) -> bool;
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
}

impl DownloadJob {
    pub fn new<T: Any + Send + Sync>(game: &MonarchGame, options: DownloadOptions, manifest: T) -> Self {
        Self {
            id: 0,
            name: game.name.clone(),
            game_id: game.id.clone(),
            path: PathBuf::from(options.folder),
            store: options.store.clone(),
            os: options.os.clone(),
            manifest: Arc::new(manifest),
        }
    }
}

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

#[derive(Debug)]
pub struct MonarchDownloader {
    ongoing: Option<DownloadJob>,
    queue: Vec<DownloadJob>,
    download_handlers: HashMap<String, Box<dyn DownloadHandler>>,
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
        }
    }

    /// Starts a new download job.
    pub fn start_download(&mut self, job: DownloadJob) {
        // Cancel ongoing download if there is one
        if let Some(ongoing) = &self.ongoing {
            if let Some(handler) = self.download_handlers.get(&ongoing.store) {
                handler.cancel(ongoing).unwrap();
            }
            self.queue_download(ongoing.clone());
        }

        // Remove the job from the queue if it exists there
        if self.queue.contains(&job) {
            self.queue.retain(|j| j.id != job.id);
        }

        let store = job.store.clone();
        self.ongoing = Some(job);

        if let Some(handler) = self.download_handlers.get(&store) {
            if let Some(ongoing) = self.ongoing.as_ref() {
                let _ = handler.download(ongoing);
            }
        }
    }

    /// Queues a new download job.
    pub fn queue_download(&mut self, job: DownloadJob) {
        self.queue.push(job);
    }

    /// Cancels a download job.
    pub fn cancel_download(&mut self, job_id: u64) {
        if let Some(job) = &self.ongoing {
            if job.id == job_id {
                self.download_handlers.get(&job.store).unwrap().cancel(job).unwrap();
                self.ongoing = None;
                return
            }
        }
        self.queue.retain(|job| job.id != job_id);
    }

    /// Checks if a download is ongoing.
    pub fn is_downloading(&self) -> bool {
        self.ongoing.is_some()
    }

    /// Gets the ongoing download job.
    pub fn get_ongoing(&self) -> Option<&DownloadJob> {
        self.ongoing.as_ref()
    }

    /// Gets the queue of download jobs.
    pub fn get_queue(&self) -> &Vec<DownloadJob> {
        self.queue.as_ref()
    }

    /// Checks if a given job is queued.
    pub fn is_queued(&self, game: &MonarchGame) -> bool {
        self.queue.iter().any(|job| job.name == game.name)
    }

    /// Registers a new download handler for a given store.id.
    pub fn register_download_handler(&mut self, store: String, handler: Box<dyn DownloadHandler>) -> Result<()> {
        if self.download_handlers.contains_key(&store) {
            bail!("Download handler for store {} already registered", store)
        }
        self.download_handlers.insert(store, handler);
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

    /// Starts the next download job in the queue.
    fn start_next_download(&mut self) {
        if self.queue.is_empty() {
            return;
        }

        let job = self.queue.remove(0);
        self.ongoing = Some(job);
    }

    /// Marks the ongoing download as completed.
    fn mark_as_completed(&mut self) {
        self.ongoing = None;
    }
}