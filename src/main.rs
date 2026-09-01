// Copyright 2026 Charli van Nood

pub mod game;
pub mod interract;
pub mod network;
pub mod physics;
pub mod renderer;
pub mod setup;
pub mod world;
pub mod xr;

use cap::Cap;
use cgmath::Vector3;
use std::alloc;
use world::object::Object;

use crate::renderer::transform::Transform;
use crate::setup::fonts::load_font_uvs;
use crate::world::objects::text;
use crate::world::{object::ObjectType, parsers::fbx_parser::parse};

#[global_allocator]
static ALLOCATOR: Cap<alloc::System> = Cap::new(alloc::System, usize::max_value());

fn main() {
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

    world.load_world("worlds/scene.cae");

    //let tablet = cube::create_cube((0.0, 0.0, 0.0), (0.5, 0.4, 0.01));
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
    world.add_object(tablet_object);

    let font_uvs = load_font_uvs("fonts/NotoSansJP.ttf");
    let sentence = text::create_plane_with_text(
        (-0.5, 0.3, -0.02),
        (0.03, 0.03, 1.0),
        &font_uvs,
        [1.0, 1.0, 1.0],
        "goodbye :C",
    );
    let mut sentence_object = Object::create(
        ObjectType::TabletMenu,
        renderer::vertex::create_vertices(&sentence),
    );
    sentence_object.set_default_texture("fonts/NotoSansJP.ttf");
    world.add_object(sentence_object);

    let chat_button = text::create_plane_with_text(
        (-0.5, 0.0, -0.02),
        (0.03, 0.03, 1.0),
        &font_uvs,
        [1.0, 1.0, 1.0],
        "CHAT",
    );
    let mut chat_button_object = Object::create(
        ObjectType::TabletMenuButton,
        renderer::vertex::create_vertices(&chat_button),
    );
    chat_button_object.set_default_texture("fonts/NotoSansJP.ttf");
    world.add_object(chat_button_object);

    let fps_label = text::create_plane_with_text(
        (-0.5, -0.3, -0.02),
        (0.02, 0.02, 1.0),
        &font_uvs,
        [1.0, 1.0, 1.0],
        "FPS: 0",
    );
    let mut fps_label_object = Object::create(
        ObjectType::TabletMenu,
        renderer::vertex::create_vertices(&fps_label),
    );
    fps_label_object.set_default_texture("fonts/NotoSansJP.ttf");
    fps_label_object.set_tag("fps_label");
    world.add_object(fps_label_object);

    let ram_label = text::create_plane_with_text(
        (-0.5, -0.2, -0.02),
        (0.02, 0.02, 1.0),
        &font_uvs,
        [1.0, 1.0, 1.0],
        "RAM: 0",
    );
    let mut ram_label_object = Object::create(
        ObjectType::TabletMenu,
        renderer::vertex::create_vertices(&ram_label),
    );
    ram_label_object.set_default_texture("fonts/NotoSansJP.ttf");
    ram_label_object.set_tag("ram_label");
    world.add_object(ram_label_object);

    println!(
        "Memory after startup: {} MB",
        ALLOCATOR.allocated() as f32 / 1000000.0
    );

    renderer::eventloop::start_engine(world);
}
