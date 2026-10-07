use std::collections::HashMap;

use crate::{
    renderer,
    ui::widget::Widget,
    world::{
        object::{Object, ObjectType},
        objects::text::create_plane_with_text,
    },
};

#[derive(Clone)]
pub struct Label {
    pub text: String,
    pub text_placeholder: String,
    pub position: (f32, f32, f32),
    pub text_color: [f32; 3],
    pub needs_update: bool,
    object_id: usize,
    font_uvs: HashMap<String, (f32, f32, f32, f32, f32)>,
}

impl Label {
    pub fn with_text(text: String, font_uvs: HashMap<String, (f32, f32, f32, f32, f32)>) -> Self {
        Self {
            text: text.clone(),
            text_placeholder: text,
            position: (0.0, 0.0, 0.0),
            text_color: [1.0, 1.0, 1.0],
            needs_update: false,
            object_id: 0,
            font_uvs: font_uvs,
        }
    }
}

impl Widget for Label {
    fn set_position(&mut self, x: f32, y: f32, z: f32) {
        self.position = (x, y, z);
    }
    fn set_text_color(&mut self, r: f32, g: f32, b: f32) {
        self.text_color = [r, g, b];
    }

    fn get_text_placeholder(&self) -> &str {
        &self.text_placeholder
    }

    fn needs_update(&self) -> bool {
        self.needs_update
    }
    fn update(&mut self) {
        self.needs_update = true;
    }

    fn get_id(&self) -> usize {
        self.object_id
    }

    fn get_meshes(&self) -> Vec<(Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, String)> {
        create_plane_with_text(
            self.position,
            (0.02, 0.02, 1.0),
            &self.font_uvs,
            self.text_color,
            &self.text,
        )
    }
    fn get_meshes_from_text(&self, text: &str) -> Vec<(Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, String)> {
        create_plane_with_text(
            self.position,
            (0.02, 0.02, 1.0),
            &self.font_uvs,
            self.text_color,
            text,
        )
    }

    fn build(&mut self, id: usize) -> Object {
        self.object_id = id;
        let sentence = self.get_meshes();
        let mut object = Object::create(
            ObjectType::Mesh,
            renderer::vertex::create_vertices(&sentence),
        );
        object.set_default_texture("fonts/NotoSansJP.ttf");
        object
    }
}
