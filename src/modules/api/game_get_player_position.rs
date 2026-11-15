use std::{cell::RefCell, rc::Rc};

use wasmtime::{Caller, anyhow};

use crate::game_objects::world::World;

// TODO: IMPLEMENT PLAYERS IN WORLD
pub fn game_get_player_position(mut caller: Caller<'_, Rc<RefCell<World>>>, out_position_ptr: u32) {
    let position = {
        let data = caller.data();
        let world = data.borrow();

        let object = world.get_object(0);
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
