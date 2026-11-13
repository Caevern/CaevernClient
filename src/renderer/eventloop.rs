use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use winit::event_loop::EventLoop;
use winit::window::Icon;

use crate::game_objects::world::World;
use crate::interract::input_state::InputState;
use crate::renderer::game_window::GameWindow;
use crate::setup::engine_arguments::EngineArguments;

fn load_icon() -> Option<Icon> {
    let image = image::open("assets/icons/icon-small.png")
        .ok()?
        .into_rgba8();
    let (width, height) = image.dimensions();

    Icon::from_rgba(image.into_raw(), width, height).ok()
}

pub fn start_engine(world: World, engine_arguments: EngineArguments) {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();

    println!("AMOUNT OF OBJECTS: {}", world.objects.len());

    let mut game_window = GameWindow {
        window: None,

        windowed_renderer: None,

        engine: None,

        depth_texture: None,

        window_size: (0, 0),

        title: "Caevern".to_string(),
        icon: load_icon(),

        render_start_time: std::time::Instant::now(),
        input: InputState::default(),

        muted: Arc::new(AtomicBool::new(true)),
        mouse_locked: false,
        use_confined: false,

        xr_enabled: !engine_arguments.no_vr,
        server_url: engine_arguments.server_url,

        menu_tablet_state: 0,

        home_world: Some(world),
    };

    event_loop.run_app(&mut game_window).unwrap();
}
