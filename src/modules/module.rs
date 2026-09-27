use wasmtime::Store;
use wasmtime::TypedFunc;

pub struct Module {
    init: TypedFunc<(), ()>,
    store: Store<()>,
}
impl Module {
    pub fn new(init: TypedFunc<(), ()>, store: Store<()>) -> Self {
        Self { init, store }
    }

    pub fn init(&mut self) {
        self.init
            .call(&mut self.store, ())
            .expect("Failed to call init function");
    }
}
