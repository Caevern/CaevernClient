use std::println;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;

use tungstenite::connect;
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

use crate::network::avatar_updates::AvatarUpdate;
use crate::network::user_authenticate::authenticate_user;
use crate::network::user_handler::start_user_handler;
use crate::network::user_updates::UserUpdate;
use crate::network::voice::start_voice_handler;
use crate::renderer::render::Renderer;
use crate::world::world::World;
use crate::xr::xr_manager::XRManager;

pub struct GameWindow<'window> {
    pub window: Option<Arc<Window>>,
    pub renderer: Option<Renderer<'window>>,

    pub depth_texture: Option<wgpu::Texture>,

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

    pub home_world: World,
}

impl<'window> ApplicationHandler for GameWindow<'window> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let attributes = WindowAttributes::default()
            .with_title(self.title.clone())
            .with_window_icon(self.icon.clone());
        let window = Arc::new(event_loop.create_window(attributes).unwrap());

        let (data_thread_tx, data_thread_rx) = mpsc::channel::<UserUpdate>();
        let (avatar_thread_tx, avatar_thread_rx) = mpsc::channel::<AvatarUpdate>();

        // TODO: change xr_enabled to an actual option
        self.xr_enabled = false;

        if self.xr_enabled {
            if let Ok(_xr) = XRManager::new() {
                println!("STARTED XRManager!!!")
            } else {
                println!("Initializing XRManager has failed :C")
            }
        }

        let mut renderer =
            pollster::block_on(Renderer::new(&window, data_thread_tx, avatar_thread_rx));

        self.depth_texture = Some(
            renderer
                .init
                .device
                .create_texture(&wgpu::TextureDescriptor {
                    size: wgpu::Extent3d {
                        width: renderer.init.config.width,
                        height: renderer.init.config.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Depth24Plus,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    label: None,
                    view_formats: &[],
                }),
        );

        self.window_size = (renderer.init.size.width, renderer.init.size.height);

        if let Ok((socket, _)) = connect("ws://localhost:42142/ws/user") {
            let (socket, user_id) = authenticate_user(socket);
            println!("User ID: {}", user_id);

            start_user_handler(socket, data_thread_rx, avatar_thread_tx, user_id);

            thread::spawn(move || {
                let runtime =
                    tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

                runtime.block_on(async {
                    start_voice_handler(user_id).await;
                });
            });
        } else {
            println!("Failed to connect to websocket /ws/user, not connected to any server");
        }

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
                self.depth_texture = Some(renderer.init.device.create_texture(
                    &wgpu::TextureDescriptor {
                        size: wgpu::Extent3d {
                            width: renderer.init.config.width,
                            height: renderer.init.config.height,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Depth24Plus,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        label: None,
                        view_formats: &[],
                    },
                ));
                self.window_size = (renderer.init.size.width, renderer.init.size.height);
            }

            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                if state.is_pressed() {
                    match button {
                        MouseButton::Left => {
                            if !self.mouse_locked {
                                if let Err(err) = self
                                    .window
                                    .as_mut()
                                    .unwrap()
                                    .set_cursor_grab(winit::window::CursorGrabMode::Locked)
                                {
                                    eprintln!("Failed to lock the cursor: {:?}", err);
                                    self.use_confined = true;

                                    if let Err(err) =
                                        self.window.as_ref().unwrap().set_cursor_grab(
                                            winit::window::CursorGrabMode::Confined,
                                        )
                                    {
                                        eprintln!("Failed to confine the cursor: {:?}", err);
                                    }
                                    let window_size = self.window.as_ref().unwrap().inner_size();
                                    let center_x = window_size.width as f64 / 2.0;
                                    let center_y = window_size.height as f64 / 2.0;
                                    self.window
                                        .as_ref()
                                        .unwrap()
                                        .set_cursor_position(winit::dpi::PhysicalPosition::new(
                                            center_x, center_y,
                                        ))
                                        .expect("Failed to set cursor position");
                                }
                                self.window.as_mut().unwrap().set_cursor_visible(false);
                                self.mouse_locked = true;
                            }
                        }
                        _ => {}
                    }
                }
            }

            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => match event.state {
                ElementState::Pressed => match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => {
                        self.keys[0] = true;
                    }
                    PhysicalKey::Code(KeyCode::KeyA) => {
                        self.keys[1] = true;
                    }
                    PhysicalKey::Code(KeyCode::KeyS) => {
                        self.keys[2] = true;
                    }
                    PhysicalKey::Code(KeyCode::KeyD) => {
                        self.keys[3] = true;
                    }
                    PhysicalKey::Code(KeyCode::Space) => {
                        self.keys[4] = true;
                    }
                    PhysicalKey::Code(KeyCode::SuperLeft)
                    | PhysicalKey::Code(KeyCode::SuperRight) => {
                        if let Err(err) = self
                            .window
                            .as_ref()
                            .unwrap()
                            .set_cursor_grab(winit::window::CursorGrabMode::None)
                        {
                            eprintln!("Failed to unlock the cursor: {:?}", err);
                        }
                        self.window.as_ref().unwrap().set_cursor_visible(true);
                    }
                    PhysicalKey::Code(KeyCode::Escape) => {
                        if self.menu_tablet_state == 0 {
                            self.menu_tablet_state = 2;
                            self.mouse_locked = false;
                            if let Err(err) = self
                                .window
                                .as_ref()
                                .unwrap()
                                .set_cursor_grab(winit::window::CursorGrabMode::None)
                            {
                                eprintln!("Failed to unlock the cursor: {:?}", err);
                            }
                            self.window.as_ref().unwrap().set_cursor_visible(true);
                        } else if self.menu_tablet_state == 1 {
                            self.menu_tablet_state = 3;
                            self.mouse_locked = true;
                            if !self.use_confined {
                                if let Err(err) = self
                                    .window
                                    .as_ref()
                                    .unwrap()
                                    .set_cursor_grab(winit::window::CursorGrabMode::Locked)
                                {
                                    eprintln!(
                                        "Failed to lock the cursor, switching to confined: {:?}",
                                        err
                                    );
                                    self.use_confined = true;

                                    if let Err(err) =
                                        self.window.as_ref().unwrap().set_cursor_grab(
                                            winit::window::CursorGrabMode::Confined,
                                        )
                                    {
                                        eprintln!("Failed to confine the cursor: {:?}", err);
                                    }
                                    let window_size = self.window.as_ref().unwrap().inner_size();
                                    let center_x = window_size.width as f64 / 2.0;
                                    let center_y = window_size.height as f64 / 2.0;
                                    self.window
                                        .as_ref()
                                        .unwrap()
                                        .set_cursor_position(winit::dpi::PhysicalPosition::new(
                                            center_x, center_y,
                                        ))
                                        .expect("Failed to set cursor position");
                                }
                            } else {
                                if let Err(err) = self
                                    .window
                                    .as_ref()
                                    .unwrap()
                                    .set_cursor_grab(winit::window::CursorGrabMode::Confined)
                                {
                                    eprintln!("Failed to confine the cursor: {:?}", err);
                                }
                                let window_size = self.window.as_ref().unwrap().inner_size();
                                let center_x = window_size.width as f64 / 2.0;
                                let center_y = window_size.height as f64 / 2.0;
                                self.window
                                    .as_ref()
                                    .unwrap()
                                    .set_cursor_position(winit::dpi::PhysicalPosition::new(
                                        center_x, center_y,
                                    ))
                                    .expect("Failed to set cursor position");
                            }
                            self.window.as_ref().unwrap().set_cursor_visible(false);
                        }
                    }
                    _ => {}
                },
                ElementState::Released => match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => {
                        self.keys[0] = false;
                    }
                    PhysicalKey::Code(KeyCode::KeyA) => {
                        self.keys[1] = false;
                    }
                    PhysicalKey::Code(KeyCode::KeyS) => {
                        self.keys[2] = false;
                    }
                    PhysicalKey::Code(KeyCode::KeyD) => {
                        self.keys[3] = false;
                    }
                    PhysicalKey::Code(KeyCode::Space) => {
                        self.keys[4] = false;
                    }
                    _ => {}
                },
            },

            WindowEvent::RedrawRequested => {
                let now = std::time::Instant::now();
                let dt = now - self.render_start_time;

                let renderer = self.renderer.as_mut().unwrap();

                renderer.update(dt, self.keys, self.mouse_movement, self.menu_tablet_state);

                if self.menu_tablet_state == 2 {
                    self.menu_tablet_state = 1;
                } else if self.menu_tablet_state == 3 {
                    self.menu_tablet_state = 0;
                }

                self.mouse_movement = [0.0, 0.0];

                match renderer.render(&self.depth_texture.as_mut().unwrap()) {
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
                    if let Err(err) =
                        self.window
                            .as_ref()
                            .unwrap()
                            .set_cursor_position(LogicalPosition {
                                x: self.window_size.0 / 2,
                                y: self.window_size.1 / 2,
                            })
                    {
                        eprint!("Failed to move back cursor {:?}", err);
                    }
                }
            }
        }
    }
}
