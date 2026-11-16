use wasmtime::{Caller, anyhow};

use crate::modules::host::HostState;

pub fn game_get_position(mut caller: Caller<'_, HostState>, object_id: u32, out_position_ptr: u32) {
    let position = {
        let data = caller.data();
        let world = data.world.borrow();

        let object = world.get_object(object_id as usize);
        object.get_position()
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
