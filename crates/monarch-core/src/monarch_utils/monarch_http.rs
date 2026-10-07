use std::sync::OnceLock;
use std::time::Duration;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

static API_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
static DOWNLOAD_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

/// Shared pooled HTTP client for API/JSON calls. Reuses connections and TLS
/// sessions across requests instead of doing a fresh handshake per call
/// (which reqwest::get does). Requests carry a hard total timeout so a hung
/// endpoint can never stall a caller indefinitely.
pub fn client() -> &'static reqwest::Client {
    API_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("monarch_http::client() Failed to build shared HTTP client")
    })
}

/// Shared pooled HTTP client for large file downloads. No total timeout (a big
/// download can legitimately take minutes), but connections still time out.
pub fn download_client() -> &'static reqwest::Client {
    DOWNLOAD_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .build()
            .expect("monarch_http::download_client() Failed to build shared HTTP client")
    })
}
