use std::{cell::RefCell, rc::Rc};

use wasmtime::Caller;

use crate::game_objects::world::World;

pub fn game_set_position(
    caller: Caller<'_, Rc<RefCell<World>>>,
    object_id: u32,
    x: f32,
    y: f32,
    z: f32,
) {
    let data = caller.data();
    let mut objects = data.borrow_mut();
    let object = objects.get_object_mut(object_id as usize);
    object.set_position(x, y, z);
    object.set_needs_update(true);
}
