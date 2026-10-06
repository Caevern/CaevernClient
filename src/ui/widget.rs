use crate::world::object::Object;

pub trait Widget {
    fn set_position(&mut self, x: f32, y: f32, z: f32);
    fn set_text_color(&mut self, r: f32, g: f32, b: f32);
    fn build(&mut self, id: usize) -> Object;
}
