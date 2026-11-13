use crate::{game_objects::world::World, ui::widget::Widget};

pub struct Canvas {
    pub children: Vec<Box<dyn Widget>>,
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
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
            let object = child.build(world.get_objects().len() + 1);
            world.add_object_at_index(object, world.get_objects().len() + 1);
        }
    }
}
