use cgmath::Vector3;

use crate::game_objects::world::World;
use crate::ui::widget::Widget;
use crate::{
    game_objects::{
        object::{Object, ObjectType},
        parsers::fbx_parser::parse,
    },
    renderer::{self, transform::Transform},
    setup::fonts::load_font_uvs,
    ui::{
        canvas::Canvas,
        widgets::{button::Button, label::Label},
    },
};

pub fn create_tablet_frame(world: &mut World) {
    let mut tablet_canvas = Canvas::new();

    let font_uvs = load_font_uvs("fonts/NotoSansJP.ttf");

    let mut tablet_frame_text_label = Label::with_text("[CLOCK]".to_string(), font_uvs.clone());
    tablet_frame_text_label.set_position(-0.5, 0.3, -0.005);
    tablet_frame_text_label.update();
    tablet_canvas.add_child(tablet_frame_text_label);

    let mut tablet_frame_chat_button = Button::with_text("Chat".to_string(), font_uvs.clone());
    tablet_frame_chat_button.set_position(-0.5, 0.0, -0.005);
    tablet_canvas.add_child(tablet_frame_chat_button);

    let mut tablet_frame_fps_label = Label::with_text("FPS: [FPS]".to_string(), font_uvs.clone());
    tablet_frame_fps_label.set_position(-0.5, -0.3, -0.005);
    tablet_frame_fps_label.update();
    tablet_canvas.add_child(tablet_frame_fps_label);

    let mut tablet_frame_ram_label = Label::with_text("RAM: [RAM]".to_string(), font_uvs.clone());
    tablet_frame_ram_label.set_position(-0.5, -0.2, -0.005);
    tablet_frame_ram_label.update();
    tablet_canvas.add_child(tablet_frame_ram_label);

    let tablet = parse(
        "models/tablet.fbx",
        Transform::new(
            Vector3::new(0.0, 0.0, 0.05),
            Vector3::new(0.0, 90.0, 0.0),
            Vector3::new(0.001, 0.005, 0.005),
        ),
        [1.0, 1.0, 1.0],
    );
    let mut tablet_object = Object::create(
        ObjectType::TabletMenu,
        renderer::vertex::create_vertices_skinned(&tablet.vertices),
    );
    tablet_object.set_position(0.0, -10.0, 0.0);
    tablet_object.set_default_texture("textures/tablet.png");
    tablet_object.set_canvas(tablet_canvas);
    let last_object = world.get_objects().len();
    tablet_object.build_canvas(world);
    world.add_object_at_index(tablet_object, last_object);
}
