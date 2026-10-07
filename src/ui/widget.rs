use crate::world::object::Object;

pub trait Widget {
    fn set_position(&mut self, x: f32, y: f32, z: f32);
    fn set_text_color(&mut self, r: f32, g: f32, b: f32);
    fn get_text_placeholder(&self) -> &str;
    fn update(&mut self);
    fn needs_update(&self) -> bool;
    fn get_id(&self) -> usize;
    fn get_meshes(&self) -> Vec<(Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, String)>;
    fn get_meshes_from_text(&self, text: &str) -> Vec<(Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, String)>;
    fn build(&mut self, id: usize) -> Object;
}
