use std::{cell::RefCell, rc::Rc, sync::mpsc::Sender};

use crate::{game::engine_command::EngineCommand, game_objects::world::World};

pub struct HostState {
    pub world: Rc<RefCell<World>>,
    pub engine_tx: Sender<EngineCommand>,
}
