// Copyright 2026 Charli van Nood

pub mod game;
pub mod game_objects;
pub mod interract;
pub mod modules;
pub mod network;
pub mod physics;
pub mod renderer;
pub mod setup;
pub mod ui;
pub mod xr;

use cap::Cap;
use std::alloc;

use crate::game_objects::world::World;
use crate::setup::engine_arguments::EngineArguments;

#[global_allocator]
static ALLOCATOR: Cap<alloc::System> = Cap::new(alloc::System, usize::MAX);

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let engine_arguments = EngineArguments::from(&args);

    let mut world = World::new();
    world.load_world("worlds/home.json");

    println!(
        "Memory after startup: {} MB",
        ALLOCATOR.allocated() as f32 / 1000000.0
    );

    renderer::eventloop::start_engine(world, engine_arguments);
}
