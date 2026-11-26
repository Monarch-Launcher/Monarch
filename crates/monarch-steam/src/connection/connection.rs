use serde::Deserialize;

use crate::MonarchSteamError;

const EMSG_SERVICE_METHOD_CALL:          u32 = 151;
const EMSG_SERVICE_METHOD_CALL_NONAUTH:  u32 = 9804;
const EMSG_SERVICE_METHOD_RESPONSE:      u32 = 147;
const EMSG_SERVICE_METHOD_NOTIFY:        u32 = 146;
const PROTO_FLAG: u32 = 0x8000_0000;

#[derive(Debug, Deserialize, Clone)]
pub struct CmConnection {
}

impl CmConnection {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn call() -> Result<(), MonarchSteamError> {
        Ok(())
    }

    pub async fn send_framed() -> Result<(), MonarchSteamError> {
        Ok(())
    }

    pub async fn on_message() -> Result<(), MonarchSteamError> {
        Ok(())
    }
}