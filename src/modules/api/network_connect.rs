use wasmtime::Caller;

use crate::modules::host::HostState;

pub fn network_connect(caller: Caller<'_, HostState>, world_id: u32) {
    let data = caller.data();
    let _ = data
        .engine_tx
        .send(crate::modules::host::EngineCommand::ConnectWorld(world_id));
}
