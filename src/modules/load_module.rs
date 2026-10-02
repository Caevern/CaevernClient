use std::{cell::RefCell, rc::Rc};

use wasmtime::{Engine, Linker, Module, Store};

use crate::{modules::{api::{game_create_mesh::game_create_mesh, game_get_time::game_get_time, game_log::game_log}, module}, world::world::World};

pub fn load_module(path: &str, world_rc: Rc<RefCell<World>>) -> module::Module {
    let engine = Engine::default();
    let module = Module::from_file(&engine, path).expect("Failed to load module");

    let mut linker = Linker::new(&engine);
    linker
        .func_wrap("env", "log", game_log)
        .expect("Failed to add function");
    linker
        .func_wrap("env", "get_time", game_get_time)
        .expect("Failed to add function");
    linker
        .func_wrap("env", "create_mesh", game_create_mesh)
        .expect("Failed to add function");

    let mut store = Store::new(&engine, world_rc);
    let instance = linker
        .instantiate(&mut store, &module)
        .expect("Failed to instantiate module");

    let init = instance
        .get_typed_func::<(), ()>(&mut store, "init")
        .expect("Failed to get init function");

    let mut module = module::Module::new(init, store);
    module.init();

    module
}
