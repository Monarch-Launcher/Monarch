use anyhow::{Context, Result};
use keyring::Entry;

/// Save credentials to OS's secure store.
pub fn set_credentials(platform: &str, username: &str, password: &str) -> Result<()> {
    let entry: Entry = Entry::new(platform, username).with_context(|| {
        "monarch_credentials::set_credentials() Failed to get/create entry in secure store! | Err: "
    })?;
    entry.set_password(password).with_context(|| "monarch_credentials::set_credentials() Something went wrong while setting password! | Err: ")
}

/// Retrieve password from OS's secure store.
pub fn get_password(platform: &str, username: &str) -> Result<String> {
    let entry: Entry = Entry::new(platform, username).with_context(|| {
        "monarch_credentials::get_password() Failed to get/create entry in secure store! | Err: "
    })?;
    entry
        .get_password()
        .with_context(|| "monarch_credentials::get_password() Could not get password! | Err: ")
}

/// Delete credentials from OS's secure store.
pub fn delete_credentials(platform: &str, username: &str) -> Result<()> {
    let entry: Entry = Entry::new(platform, username).with_context(|| "monarch_credentials::delete_credentials() Failed to get/create entry in secure store! | Err: ")?;
    entry.delete_password().with_context(|| "monarch_credentials::delete_credentials() Something went wrong while deleting password from secure store! | Err: ")
}
