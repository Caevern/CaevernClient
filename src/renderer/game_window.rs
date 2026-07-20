use std::println;
use std::sync::Arc;
use std::sync::mpsc;

use winit::application::ApplicationHandler;
use winit::dpi::LogicalPosition;
use winit::event::DeviceEvent;
use winit::event::DeviceId;
use winit::event::ElementState;
use winit::event::MouseButton;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::keyboard::PhysicalKey;
use winit::window::WindowAttributes;
use winit::window::{Icon, Window};

use crate::network::users::LocalUserUpdate;
use crate::network::users::start_user_handler;
use crate::renderer::render::Renderer;
use crate::world::world::World;
use crate::xr::xr_manager::XRManager;

pub struct GameWindow<'window> {
    pub window: Option<Arc<Window>>,
    pub renderer: Option<Renderer<'window>>,

    pub window_size: (u32, u32),

    pub render_start_time: std::time::Instant,

    pub title: String,
    pub icon: Option<Icon>,

    pub keys: [bool; 6],
    pub mouse_movement: [f32; 2],

    pub mouse_locked: bool,
    pub use_confined: bool,
    pub xr_enabled: bool,
    pub menu_tablet_state: usize,

    pub home_world: World
}

impl<'window> ApplicationHandler for GameWindow<'window> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let attributes = WindowAttributes::default()
            .with_title(self.title.clone())
            .with_window_icon(self.icon.clone());
        let window = Arc::new(event_loop.create_window(attributes).unwrap());

        let (job_tx, job_rx) = mpsc::channel::<LocalUserUpdate>();

        // TODO: change xr_enabled to an actual option
        self.xr_enabled = true;

        if self.xr_enabled {
            if let Ok(xr) = XRManager::new() {
                println!("STARTED XRManager!!!")
            } else {
                println!("Initializing XRManager has failed :C")
            }
        }

        let mut renderer = pollster::block_on(Renderer::new(
            &window,
            job_tx,
        ));

        self.window_size = (
            renderer.init.size.width,
            renderer.init.size.height,
        );

        start_user_handler(job_rx);

        renderer.set_world(self.home_world.clone());

        self.render_start_time = std::time::Instant::now();
        self.renderer = Some(renderer);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Closing Caevern");
                self.renderer = None;
                self.window = None;
                event_loop.exit()
            }

            WindowEvent::Resized(size) => {
                let renderer = self.renderer.as_mut().unwrap();
                renderer.resize(size);
                self.window_size = (
                    renderer.init.size.width,
                    renderer.init.size.height,
                );
            }

            WindowEvent::MouseInput { device_id: _, state, button } => {
                if state.is_pressed() {
                    match button {
                        MouseButton::Left => {
                            if !self.mouse_locked {
                                if let Err(err) = self.window.as_mut().unwrap().set_cursor_grab(winit::window::CursorGrabMode::Locked) {
                                    eprintln!("Failed to lock the cursor: {:?}", err);
                                    self.use_confined = true;
                                }
                                self.window.as_mut().unwrap().set_cursor_visible(false);
                                self.mouse_locked = true;
                            }
                        }
                        _ => {}
                    }
                }
            }

            WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _ } => {
                match event.state {
                    ElementState::Pressed => {
                        match event.physical_key {
                            PhysicalKey::Code(KeyCode::KeyW) => { self.keys[0] = true; }
                            PhysicalKey::Code(KeyCode::KeyA) => { self.keys[1] = true; }
                            PhysicalKey::Code(KeyCode::KeyS) => { self.keys[2] = true; }
                            PhysicalKey::Code(KeyCode::KeyD) => { self.keys[3] = true; }
                            PhysicalKey::Code(KeyCode::Space) => { self.keys[4] = true; }
                            PhysicalKey::Code(KeyCode::Escape) | PhysicalKey::Code(KeyCode::SuperLeft) | PhysicalKey::Code(KeyCode::SuperRight) => {
                                self.mouse_locked = false;
                                if let Err(err) = self.window.as_ref().unwrap().set_cursor_grab(winit::window::CursorGrabMode::None) {
                                    eprintln!("Failed to unlock the cursor: {:?}", err);
                                }
                                self.window.as_ref().unwrap().set_cursor_visible(true);
                            }
                            _ => {}
                        }
                    }
                    ElementState::Released => {
                        match event.physical_key {
                            PhysicalKey::Code(KeyCode::KeyW) => { self.keys[0] = false; }
                            PhysicalKey::Code(KeyCode::KeyA) => { self.keys[1] = false; }
                            PhysicalKey::Code(KeyCode::KeyS) => { self.keys[2] = false; }
                            PhysicalKey::Code(KeyCode::KeyD) => { self.keys[3] = false; }
                            PhysicalKey::Code(KeyCode::Space) => { self.keys[4] = false; }
                            _ => {}
                        }
                    }
                }
            }

            WindowEvent::RedrawRequested => {
                let now = std::time::Instant::now();
                let dt = now - self.render_start_time;

                let renderer = self.renderer.as_mut().unwrap();

                renderer.update(
                    dt,
                    self.keys,
                    self.mouse_movement,
                    self.menu_tablet_state
                );

                self.mouse_movement = [0.0, 0.0];

                match renderer.render() {
                    Ok(_) => {
                        self.window.as_ref().unwrap().request_redraw();
                    }
                    Err(e) => eprintln!("{:?}", e),
                }
            }

            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = event {
            if self.mouse_locked {
                self.mouse_movement[0] -= delta.0 as f32 * 0.3;
                self.mouse_movement[1] -= delta.1 as f32 * 0.3;
                if !self.use_confined {
                    if let Err(err) = self.window.as_ref().unwrap().set_cursor_position(
                        LogicalPosition{x: self.window_size.0 / 2, y: self.window_size.1 / 2}
                    ) {
                        eprint!("Failed to move back cursor {:?}", err);
                    }
                }
            }
        }
    }
}