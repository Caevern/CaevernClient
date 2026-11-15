use wasmtime::{Caller, anyhow};

use crate::modules::host::HostState;

pub fn game_get_player_position(mut caller: Caller<'_, HostState>, out_position_ptr: u32) {
    let position = {
        let data = caller.data();
        let world = data.world.borrow();

        world.get_player_position()
    };

    let memory = caller
        .get_export("memory")
        .and_then(|export| export.into_memory())
        .ok_or_else(|| anyhow::anyhow!("WASM memory export not found"))
        .expect("WASM memory export not found");

    let values = [position.x, position.y, position.z];
    let bytes = bytemuck::cast_slice(&values);

    memory
        .write(&mut caller, out_position_ptr as usize, bytes)
        .expect("Failed to write position to WASM memory");
}
