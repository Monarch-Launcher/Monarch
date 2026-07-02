use super::DownloadManifest;

pub struct DownloadManager {}

impl DownloadManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start_download(&self, manifest: &DownloadManifest) {}
}
