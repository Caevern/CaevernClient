use std::cell::RefCell;
use std::rc::Rc;

use wasmtime::Store;
use wasmtime::TypedFunc;

use crate::world::world::World;

pub struct Module {
    init: TypedFunc<(), ()>,
    store: Store<Rc<RefCell<World>>>,
}
impl Module {
    pub fn new(init: TypedFunc<(), ()>, store: Store<Rc<RefCell<World>>>) -> Self {
        Self { init, store }
    }

    pub fn init(&mut self) {
        self.init
            .call(&mut self.store, ())
            .expect("Failed to call init function");
    }
}
