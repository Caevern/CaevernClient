use wasmtime::{Engine, Instance, Linker, Module, Store};

use crate::modules::{api::game_log::game_log, module};

pub fn load_module(path: &str) -> module::Module {
    let engine = Engine::default();
    let module = Module::from_file(&engine, path).expect("Failed to load module");

    let mut linker = Linker::new(&engine);
    linker
        .func_wrap("env", "log", game_log)
        .expect("Failed to add function");

    let mut store = Store::new(&engine, ());
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
