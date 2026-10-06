// Copyright 2026 Charli van Nood

pub mod game;
pub mod interract;
pub mod modules;
pub mod network;
pub mod physics;
pub mod renderer;
pub mod setup;
pub mod ui;
pub mod world;
pub mod xr;

use cap::Cap;
use cgmath::Vector3;
use std::alloc;
use world::object::Object;

use crate::renderer::transform::Transform;
use crate::setup::fonts::load_font_uvs;
use crate::ui::canvas::Canvas;
use crate::ui::widget::Widget;
use crate::ui::widgets::button::Button;
use crate::ui::widgets::label::Label;
use crate::world::{object::ObjectType, parsers::fbx_parser::parse};

#[global_allocator]
static ALLOCATOR: Cap<alloc::System> = Cap::new(alloc::System, usize::max_value());

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let no_vr = args.contains(&"--no-vr".to_string());

    if no_vr {
        println!("Running desktop mode");
    } else {
        println!("Running VR mode");
    }

    let mut world = world::world::create_world();

    let skybox = parse(
        "models/skybox.fbx",
        Transform::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.5, 1.5, 1.5),
        ),
    );
    let mut skybox_object = Object::create(
        ObjectType::Skybox,
        renderer::vertex::create_vertices_skinned(&skybox.0),
    );
    world.textures.insert("textures/skybox_2.png".to_string());
    skybox_object.set_default_texture("textures/skybox_2.png");
    skybox_object.set_movable(true);
    world.add_object(skybox_object);

    let mut camera = Object::create(ObjectType::Camera, Vec::new());
    camera.set_position(-3.0, 4.0, 3.0);
    camera.set_rotation(0.0, -45.0, 0.0);
    world.add_object(camera);

    //world.load_world("worlds/scene3.cae");
    world.load_world("worlds/test.json");

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
    );
    let mut tablet_object = Object::create(
        ObjectType::TabletMenu,
        renderer::vertex::create_vertices_skinned(&tablet.0),
    );
    tablet_object.set_position(0.0, -10.0, 0.0);
    tablet_object.set_default_texture("textures/tablet.png");
    tablet_object.set_canvas(tablet_canvas);
    let last_object = world.get_objects().len() - 1;
    tablet_object.build_canvas(&mut world);
    world.add_object_at_index(tablet_object, last_object);

    println!(
        "Memory after startup: {} MB",
        ALLOCATOR.allocated() as f32 / 1000000.0
    );

    renderer::eventloop::start_engine(world, no_vr);
}
