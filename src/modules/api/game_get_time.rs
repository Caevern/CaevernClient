use std::time::{SystemTime, UNIX_EPOCH};

use wasmtime::Caller;

use crate::modules::host::HostState;

pub fn game_get_time(mut _caller: Caller<'_, HostState>) -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u32
}
