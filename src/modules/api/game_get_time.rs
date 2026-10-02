use std::{cell::RefCell, rc::Rc, time::{SystemTime, UNIX_EPOCH}};

use wasmtime::Caller;

use crate::world::world::World;

pub fn game_get_time(mut _caller: Caller<'_, Rc<RefCell<World>>>) -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u32
}
