use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use winit::event_loop::EventLoop;
use winit::window::Icon;

use crate::interract::input_state::InputState;
use crate::renderer::game_window::GameWindow;
use crate::world::world::World;

fn load_icon() -> Option<Icon> {
    let image = image::open("assets/icons/icon-small.png")
        .ok()?
        .into_rgba8();
    let (width, height) = image.dimensions();

    Icon::from_rgba(image.into_raw(), width, height).ok()
}

pub fn start_engine(world: World, no_vr: bool) {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();

    let mut game_window = GameWindow {
        window: None,

        windowed_renderer: None,

        engine: None,

        depth_texture: None,

        window_size: (0, 0),

        title: "Caevern".to_string(),
        icon: load_icon(),

        render_start_time: std::time::Instant::now(),
        keys: [false; 6],
        mouse_movement: [0.0; 2],
        input: InputState::default(),

        muted: Arc::new(AtomicBool::new(true)),
        mouse_locked: false,
        use_confined: false,

        xr_enabled: !no_vr,

        menu_tablet_state: 0,

        home_world: Some(world),
    };

    event_loop.run_app(&mut game_window).unwrap();
}
