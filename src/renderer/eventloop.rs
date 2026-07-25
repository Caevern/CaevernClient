use winit::event_loop::EventLoop;

use crate::renderer::game_window::GameWindow;
use crate::world::world::World;

pub fn start_engine(world: World) {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();

    let mut game_window = GameWindow {
        window: None,
        renderer: None,

        depth_texture: None,

        window_size: (0, 0),

        title: "Caevern".to_string(),
        icon: None,

        render_start_time: std::time::Instant::now(),
        keys: [false; 6],
        mouse_movement: [0.0; 2],

        mouse_locked: false,
        use_confined: false,
        xr_enabled: false,
        menu_tablet_state: 0,

        home_world: world,
    };

    event_loop.run_app(&mut game_window).unwrap();
}
