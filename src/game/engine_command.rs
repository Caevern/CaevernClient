use crate::game_objects::world::World;

pub enum EngineCommand {
    ConnectWorld(u32),
    LoadWorldData(Box<World>),
}
