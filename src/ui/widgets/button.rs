use crate::{
    renderer,
    setup::fonts::load_font_uvs,
    ui::widget::Widget,
    world::{
        object::{Object, ObjectType},
        objects::text::create_plane_with_text,
    },
};

#[derive(Clone)]
pub struct Button {
    pub text: String,
    pub position: (f32, f32, f32),
    pub text_color: [f32; 3],
    object_id: usize,
}

impl Button {
    pub fn with_text(text: String) -> Self {
        Self {
            text,
            position: (0.0, 0.0, 0.0),
            text_color: [1.0, 1.0, 1.0],
            object_id: 0,
        }
    }
}

impl Widget for Button {
    fn set_position(&mut self, x: f32, y: f32, z: f32) {
        self.position = (x, y, z);
    }

    fn set_text_color(&mut self, r: f32, g: f32, b: f32) {
        self.text_color = [r, g, b];
    }

    fn get_id(&self) -> usize {
        self.object_id
    }

    fn build(&mut self, id: usize) -> Object {
        self.object_id = id;
        let font_uvs = load_font_uvs("fonts/NotoSansJP.ttf");
        let sentence = create_plane_with_text(
            self.position,
            (0.03, 0.03, 1.0),
            &font_uvs,
            self.text_color,
            &self.text,
        );
        let mut object = Object::create(
            ObjectType::Mesh,
            renderer::vertex::create_vertices(&sentence),
        );
        object.set_default_texture("fonts/NotoSansJP.ttf");
        object
    }
}
