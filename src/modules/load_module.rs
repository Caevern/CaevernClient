use std::{cell::RefCell, rc::Rc, sync::mpsc::Sender};
use wasmtime::{Engine, Linker, Module, Store};

use crate::{
    game::engine_command::EngineCommand,
    game_objects::world::World,
    modules::{
        api::{
            game_create_mesh::game_create_mesh, game_create_mesh_indexed::game_create_mesh_indexed,
            game_get_element_by_name::game_get_element_by_name,
            game_get_player_position::game_get_player_position,
            game_get_position::game_get_position, game_get_time::game_get_time, game_log::game_log,
            game_set_position::game_set_position, network_connect::network_connect,
        },
        host::HostState,
        module,
    },
    setup::assets::Assets,
};

pub fn load_module(
    path: &str,
    world_rc: Rc<RefCell<World>>,
    engine_tx: Sender<EngineCommand>,
) -> module::Module {
    let engine = Engine::default();
    let asset = Assets::get(path).unwrap_or_else(|| panic!("Embedded asset not found: {}", path));
    let module = Module::new(&engine, asset.data.as_ref()).expect("Failed to load module");

    let mut linker = Linker::new(&engine);
    linker
        .func_wrap("env", "log", game_log)
        .expect("Failed to add function log");
    linker
        .func_wrap("env", "get_time", game_get_time)
        .expect("Failed to add function get_time");
    linker
        .func_wrap("env", "get_element_by_name", game_get_element_by_name)
        .expect("Failed to add function get_element_by_name");
    linker
        .func_wrap("env", "create_mesh", game_create_mesh)
        .expect("Failed to add function create_mesh");
    linker
        .func_wrap("env", "create_mesh_indexed", game_create_mesh_indexed)
        .expect("Failed to add function create_mesh_indexed");
    linker
        .func_wrap("env", "get_position", game_get_position)
        .expect("Failed to add function game_get_position");
    linker
        .func_wrap("env", "get_player_position", game_get_player_position)
        .expect("Failed to add function game_get_player_position");
    linker
        .func_wrap("env", "set_position", game_set_position)
        .expect("Failed to add function game_set_position");
    linker
        .func_wrap("env", "connect", network_connect)
        .expect("Failed to add function network_connect");

    let host_state = HostState {
        world: world_rc,
        engine_tx,
    };

    let mut store = Store::new(&engine, host_state);
    let instance = linker
        .instantiate(&mut store, &module)
        .expect("Failed to instantiate module");

    let init = instance
        .get_typed_func::<(), ()>(&mut store, "init")
        .expect("Failed to get init function");
    let update = instance
        .get_typed_func::<(f32,), ()>(&mut store, "update")
        .expect("Failed to get update function");

    let mut module = module::Module::new(init, update, store);
    module.init();

    module
}
