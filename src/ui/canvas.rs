use crate::{ui::widget::Widget, world::world::World};

pub struct Canvas {
    pub children: Vec<Box<dyn Widget>>,
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn add_child<W: Widget + 'static>(&mut self, child: W) {
        self.children.push(Box::new(child));
    }

    pub fn build(&mut self, world: &mut World) {
        for child in self.children.iter_mut() {
            let object = child.build(world.get_objects().len());
            world.add_object_at_index(object, world.get_objects().len());
        }
    }
}
