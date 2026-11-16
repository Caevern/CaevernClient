use wasmtime::Store;
use wasmtime::TypedFunc;

use crate::modules::host::HostState;

pub struct Module {
    init: TypedFunc<(), ()>,
    update: TypedFunc<(f32,), ()>,
    store: Store<HostState>,
}
impl Module {
    pub fn new(
        init: TypedFunc<(), ()>,
        update: TypedFunc<(f32,), ()>,
        store: Store<HostState>,
    ) -> Self {
        Self {
            init,
            update,
            store,
        }
    }

    pub fn init(&mut self) {
        self.init
            .call(&mut self.store, ())
            .expect("Failed to call init function");
    }
    pub fn update(&mut self, dt: f32) {
        self.update
            .call(&mut self.store, (dt,))
            .expect("Failed to call update function");
    }
}
