use cgmath::*;
use winit::window::Window;
use std::iter;
use wgpu::BindGroup;
use rust_embed::RustEmbed;
use std::collections::HashMap;
use std::sync::mpsc::Sender;

use crate::interract::raycast::raycast_grab;
use crate::physics::gravity::apply_gravity;
use crate::physics::movement::{get_camera_movement, get_camera_rotation};
use crate::renderer::texture_object::TextureObject;
use crate::renderer::transform::Transform;
use crate::renderer::{transform, transforms, vertex};
use crate::renderer::vertex::Vertex;
use crate::setup::fonts::{load_font_atlas, load_font_uvs};
use crate::world::object::ObjectType;
use crate::world::objects::player::Player;
use crate::world::objects::text;
use crate::world::world::World;
use crate::network::users::{LocalUserUpdate};
use crate::ALLOCATOR;

#[derive(PartialEq)]
pub enum ShaderType {
    Displacement, DisplacementBones
}

#[derive(RustEmbed)]
#[folder = "assets/"]
pub struct Assets;

pub struct Renderer {
    pub init: transforms::InitWgpu,
    project_mat: Matrix4<f32>,

    pipeline_displacement: wgpu::RenderPipeline,
    pipeline_displacement_bones: wgpu::RenderPipeline,

    frame: usize,
    previous_frame_time: std::time::Instant,

    vertex_buffers: Vec<Vec<wgpu::Buffer>>,
    uniform_bind_groups: Vec<Vec<wgpu::BindGroup>>,
    num_vertices: Vec<Vec<u32>>,
    bone_buffers: Vec<wgpu::Buffer>,
    bones: Vec<Vec<(transform::Transform, i64)>>,
    shader_type: Vec<ShaderType>,

    uniform_bind_group_layout: wgpu::BindGroupLayout,
    vertex_uniform_buffer: wgpu::Buffer,
    model_uniform_buffers: Vec<wgpu::Buffer>,
    fragment_uniform_buffer: wgpu::Buffer,

    textures: HashMap<String, TextureObject>,
    font_maps: HashMap<String, HashMap<String, (f32, f32, f32, f32, f32)>>,

    // the client position and rotation
    player: Player,
    //camera_position: (f32, f32, f32),
    //camera_rotation: (f32, f32, f32),
    //camera_acceleration_walking: (f32, f32, f32),

    world: World,
    current_camera: usize,

    // networking
    job_tx: Sender<LocalUserUpdate>
}
impl Renderer {
    fn create_buffer_displacement(
        init: &transforms::InitWgpu, 
        uniform_bind_group_layout: &wgpu::BindGroupLayout, 
        vertex_uniform_buffer: &wgpu::Buffer, fragment_uniform_buffer: &wgpu::Buffer, model_uniform_buffer: &wgpu::Buffer, bones_buffer: &wgpu::Buffer,
        displacement_texture: &wgpu::Texture, displacement_texture_size: wgpu::Extent3d, 
        displacement_rgba: &Vec<u8>, displacement_width: u32, displacement_height: u32,
        texture: &wgpu::Texture, texture_size: wgpu::Extent3d, rgba: &Vec<u8>, width: u32, height: u32, vertex_num: usize,
    ) -> (BindGroup, wgpu::Buffer) {
        init.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            texture_size,
        );

        init.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &displacement_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &displacement_rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * displacement_width),
                rows_per_image: Some(displacement_height),
            },
            displacement_texture_size,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let displacement_texture_view = displacement_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = init.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let uniform_bind_group = init.device.create_bind_group(&wgpu::BindGroupDescriptor{
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: vertex_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: fragment_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&displacement_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: model_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: bones_buffer.as_entire_binding(),
                },
            ],
            label: Some("Uniform Bind Group"),
        });

        let max_buffer_size = size_of::<Vertex>() * vertex_num;
        let vertex_buffer = init.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: max_buffer_size as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        return (uniform_bind_group, vertex_buffer)
    }

    pub async fn new(window: &Window, job_tx: Sender<LocalUserUpdate>) -> Self {
        let init =  transforms::InitWgpu::init_wgpu(window).await;

        let camera_position: (f32, f32, f32) = (-10.0, 4.0, 0.0);
        let camera_rotation: (f32, f32, f32) = (0.0, 0.0, 0.0);

        let (_, project_mat, _) = transforms::create_view_projection(
            camera_position.into(), camera_rotation.into(), cgmath::Vector3::unit_y(), 
            init.config.width as f32 / init.config.height as f32);

        let uniform_bind_group_layout: wgpu::BindGroupLayout = init.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("Uniform Bind Group Layout"),
        });

        let pipeline_layout = init.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader_displacement = init.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/displacement.wgsl").into()),
        });

        let pipeline_displacement = init.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_displacement,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_displacement,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: init.config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState{
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            //depth_stencil: None,
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None
        });

        let shader_displacement_bones = init.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/displacement_bones.wgsl").into()),
        });

        let pipeline_displacement_bones = init.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_displacement_bones,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_displacement_bones,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: init.config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState{
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            //depth_stencil: None,
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None
        });

        let vertex_uniform_buffer: wgpu::Buffer = init.device.create_buffer(&wgpu::BufferDescriptor{
            label: Some("Vertex Uniform Buffer"),
            size: 192,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let fragment_uniform_buffer = init.device.create_buffer(&wgpu::BufferDescriptor{
            label: Some("Fragment Uniform Buffer"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let model_mat = transforms::create_transforms(
            [0.0, 0.0, 0.0], 
            [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]
        );
        let normal_mat = (model_mat.invert().unwrap()).transpose();

        let model_ref:&[f32; 16] = model_mat.as_ref();
        let normal_ref:&[f32; 16] = normal_mat.as_ref();
        init.queue.write_buffer(&vertex_uniform_buffer, 0, bytemuck::cast_slice(model_ref));
        init.queue.write_buffer(&vertex_uniform_buffer, 128, bytemuck::cast_slice(normal_ref));

        let mut textures: HashMap<String, TextureObject> = HashMap::new();

        // create missing texture
        textures.insert("textures/missing.png".to_string(), TextureObject::create("textures/missing.png", &init));
        textures.insert("textures/tablet.png".to_string(), TextureObject::create("textures/tablet.png", &init));
        textures.insert("textures/displacement.png".to_string(), TextureObject::create("textures/displacement.png", &init));

        // create font atlasses
        textures.insert("fonts/NotoSansJP.ttf".to_string(), TextureObject::load_from_dynamic_image(load_font_atlas("fonts/NotoSansJP.ttf"), &init));

        let mut font_maps: HashMap<String, HashMap<String, (f32, f32, f32, f32, f32)>> = HashMap::new();

        font_maps.insert("NotoSansJP".to_string(), load_font_uvs("fonts/NotoSansJP.ttf"));

        let vertex_buffers = Vec::new();
        let uniform_bind_groups = Vec::new();
        let num_vertices = Vec::new();
        let bone_buffers = Vec::new();
        let bones = Vec::new();
        let shader_type = Vec::new();

        let previous_frame_time = std::time::Instant::now();

        Self {
            init,
            project_mat,

            pipeline_displacement,
            pipeline_displacement_bones,

            frame: 0,
            previous_frame_time,

            vertex_buffers,
            uniform_bind_groups,
            num_vertices,
            bone_buffers,
            bones,
            shader_type,

            uniform_bind_group_layout,
            vertex_uniform_buffer,
            model_uniform_buffers: Vec::new(),
            fragment_uniform_buffer,

            textures,
            font_maps,

            player: Player::new(),
            //camera_position,
            //camera_rotation,
            //camera_acceleration_walking: (0.0, 0.0, 0.0),

            world: World::new(),
            current_camera: 0,

            job_tx
        }
    }
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.init.instance.poll_all(true);
            self.init.size = new_size;
            self.init.config.width = new_size.width;
            self.init.config.height = new_size.height;
            self.init.surface.configure(&self.init.device, &self.init.config);
            self.project_mat = transforms::create_projection(new_size.width as f32 / new_size.height as f32);
        }
    }
    pub fn update(&mut self, _dt: std::time::Duration, keys: [bool; 6], mouse: [f32; 2], menu_tablet_state: usize) {
        let current_time = std::time::Instant::now();
        let mut frame_time = current_time.duration_since(self.previous_frame_time).as_secs_f32() * 20.0;
        self.previous_frame_time = current_time;

        if frame_time > 5.0 {
            frame_time = 5.0
        }

        let updated_camera_rotation = get_camera_rotation(&self.player, mouse, frame_time);
        self.player.camera.rotation.x = updated_camera_rotation.0;
        self.player.camera.rotation.y = updated_camera_rotation.1;

        let forward = Vector3::new(
            self.player.camera.rotation.y.cos() * self.player.camera.rotation.x.cos(),
            self.player.camera.rotation.x.sin(),
            self.player.camera.rotation.y.sin() * self.player.camera.rotation.x.cos(),
        ).normalize();

        let updated_camera_position = get_camera_movement(
            &mut self.player, keys, forward, frame_time
        );
        self.player.camera.position += updated_camera_position;

        apply_gravity(&mut self.player, frame_time);

        if menu_tablet_state == 2 {
            for i in 0..self.world.get_objects().len() {
                let object_type = self.world.get_objects()[i].get_object_type();
                if object_type == ObjectType::TabletMenu || object_type == ObjectType::TabletMenuButton {
                    let model_mat = transforms::create_transforms(
                        [
                            self.player.camera.position.x + forward.x,
                            self.player.camera.position.y + forward.y,
                            self.player.camera.position.z + forward.z
                            ],
                        [
                            -self.player.camera.rotation.x,
                            -self.player.camera.rotation.y + std::f32::consts::FRAC_PI_2,
                            -self.player.camera.rotation.z
                            ], [1.0, 1.0, 1.0]
                    );
                    let normal_mat = (model_mat.invert().unwrap()).transpose();

                    let model_ref:&[f32; 16] = model_mat.as_ref();
                    let normal_ref:&[f32; 16] = normal_mat.as_ref();

                    self.init.queue.write_buffer(&self.model_uniform_buffers[i], 0, bytemuck::cast_slice(model_ref));
                    self.init.queue.write_buffer(&self.model_uniform_buffers[i], 64, bytemuck::cast_slice(normal_ref));
                }
            }
        } else if menu_tablet_state == 3 {
            for i in 0..self.world.get_objects().len() {
                let object_type = self.world.get_objects()[i].get_object_type();
                if object_type == ObjectType::TabletMenu || object_type == ObjectType::TabletMenuButton {
                    let model_mat = transforms::create_transforms(
                        [0.0, -10.0, 0.0],
                        [
                            -self.player.camera.rotation.x,
                            -self.player.camera.rotation.y + std::f32::consts::FRAC_PI_2,
                            -self.player.camera.rotation.z
                            ], [1.0, 1.0, 1.0]
                    );
                    let normal_mat = (model_mat.invert().unwrap()).transpose();

                    let model_ref:&[f32; 16] = model_mat.as_ref();
                    let normal_ref:&[f32; 16] = normal_mat.as_ref();

                    self.init.queue.write_buffer(&self.model_uniform_buffers[i], 0, bytemuck::cast_slice(model_ref));
                    self.init.queue.write_buffer(&self.model_uniform_buffers[i], 64, bytemuck::cast_slice(normal_ref));
                }
            }
        }

        for i in 0..self.world.get_objects().len() {
            if self.world.get_object(i).get_object_type() == ObjectType::Skybox {
                let model_mat = transforms::create_transforms(
                    [
                        self.player.camera.position.x,
                        self.player.camera.position.y,
                        self.player.camera.position.z
                        ],
                    [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]
                );
                let normal_mat = (model_mat.invert().unwrap()).transpose();

                let model_ref:&[f32; 16] = model_mat.as_ref();
                let normal_ref:&[f32; 16] = normal_mat.as_ref();
                let eye_position:&[f32; 3] = &self.player.camera.position.into();
                self.init.queue.write_buffer(&self.fragment_uniform_buffer, 16, bytemuck::cast_slice(eye_position));
                self.init.queue.write_buffer(&self.model_uniform_buffers[i], 0, bytemuck::cast_slice(model_ref));
                self.init.queue.write_buffer(&self.model_uniform_buffers[i], 64, bytemuck::cast_slice(normal_ref));
            } else if self.world.get_object(i).get_object_type() == ObjectType::SkinnedMesh {
                let skeleton = self.world.get_object(i).get_skeleton();
                self.bones[i][skeleton["neck"]].0.rotation.y = self.player.camera.rotation.y + 1.57079633;
                //self.bones[i][skeleton["head"]].0.scale.x = 0.0;
                //self.bones[i][skeleton["head"]].0.scale.y = 0.0;
                //self.bones[i][skeleton["head"]].0.scale.z = 0.0;
                self.update_bones(i);
            }
        }

        // update skybox positions
        if self.frame % 10 == 0 {
            let grabbable_object_index = raycast_grab(
                self.world.get_objects(), self.player.camera.position, forward, 5
            );

            if grabbable_object_index > 0 {
                let y_rotation = self.world.get_objects()[grabbable_object_index].get_rotation().y;
                self.world.objects[grabbable_object_index].set_rotation_y(y_rotation + 0.1);
                let model_mat = transforms::create_transforms(
                    [0.0, 0.0, 0.0], 
                    [0.0, y_rotation + 0.1, 0.0], [1.0, 1.0, 1.0]
                );
                let normal_mat = (model_mat.invert().unwrap()).transpose();

                let model_ref:&[f32; 16] = model_mat.as_ref();
                let normal_ref:&[f32; 16] = normal_mat.as_ref();
                let eye_position:&[f32; 3] = &self.player.camera.position.into();
                self.init.queue.write_buffer(&self.fragment_uniform_buffer, 16, bytemuck::cast_slice(eye_position));
                self.init.queue.write_buffer(&self.model_uniform_buffers[grabbable_object_index], 0, bytemuck::cast_slice(model_ref));
                self.init.queue.write_buffer(&self.model_uniform_buffers[grabbable_object_index], 64, bytemuck::cast_slice(normal_ref));
            }

            let _ = self.job_tx.send(LocalUserUpdate::SendUserPosition(
                Transform {
                    position: self.player.camera.position,
                    rotation: self.player.camera.rotation,
                    scale: Vector3::new(0.0, 0.0, 0.0)
                }
            ));
        }

        let up_direction = cgmath::Vector3::unit_y();
        let camera_position = Point3 {
            x: self.player.camera.position.x,
            y: self.player.camera.position.y,
            z: self.player.camera.position.z
        };
        let (view_mat, project_mat, _) = transforms::create_view_rotation(
            camera_position, self.player.camera.rotation.y, self.player.camera.rotation.x,
            up_direction, self.init.config.width as f32 / self.init.config.height as f32
        );

        let view_project_mat = project_mat * view_mat;
        let view_projection_ref:&[f32; 16] = view_project_mat.as_ref();

        self.init.queue.write_buffer(&self.vertex_uniform_buffer, 64, bytemuck::cast_slice(view_projection_ref));

        let current_time_updated = std::time::Instant::now();
        let update_time = current_time_updated.duration_since(current_time).as_secs_f32();

        // update ingame fps label when menu tablet is enabled
        if menu_tablet_state == 1 && self.frame % 60 == 0 {
            for (index, object) in self.world.get_objects().iter().enumerate() {
                match object.get_tag() {
                    "fps_label" => {
                        let fps_label = text::create_plane_with_text(
                        (-0.5, -0.3, -0.02), (0.02, 0.02, 1.0),
                        &self.font_maps["NotoSansJP"], [1.0, 1.0, 1.0], &format!("FPS: {}", (1.0 / update_time).round())
                        );
                        let meshes = vertex::create_vertices(&fps_label);
                        for (vertices, _) in meshes {
                            self.num_vertices[index] = vec![vertices.len() as u32];
                            let vertex_buffer = self.init.device.create_buffer(&wgpu::BufferDescriptor {
                                label: Some("Vertex Buffer"),
                                size: (size_of::<Vertex>() * vertices.len()) as u64,
                                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                                mapped_at_creation: false
                            });
                            self.vertex_buffers[index] = vec![vertex_buffer];
                            self.init.queue.write_buffer(&self.vertex_buffers[index][0], 0, bytemuck::cast_slice(&vertices));
                        }
                    }
                    "ram_label" => {
                        let ram_label = text::create_plane_with_text(
                        (-0.5, -0.2, -0.02), (0.02, 0.02, 1.0),
                        &self.font_maps["NotoSansJP"], [1.0, 1.0, 1.0], &format!("RAM: {:.2} MB", ALLOCATOR.allocated() as f32 / 1000000.0)
                        );
                        let meshes = vertex::create_vertices(&ram_label);
                        for (vertices, _) in meshes {
                            self.num_vertices[index] = vec![vertices.len() as u32];
                            let vertex_buffer = self.init.device.create_buffer(&wgpu::BufferDescriptor {
                                label: Some("Vertex Buffer"),
                                size: (size_of::<Vertex>() * vertices.len()) as u64,
                                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                                mapped_at_creation: false
                            });
                            self.vertex_buffers[index] = vec![vertex_buffer];
                            self.init.queue.write_buffer(&self.vertex_buffers[index][0], 0, bytemuck::cast_slice(&vertices));
                        }
                    }
                    _ => { continue; }
                };
            }
        }

        self.frame += 1;
    }

    pub fn update_bones(&mut self, object_index: usize) {
        let mut bones: Vec<[[f32; 4]; 4]> = Vec::new();
        for bone in self.bones[object_index].iter() {
            let mut position_added = bone.0.position;
            let mut rotation_added = bone.0.rotation;
            if bone.1 != -1 {
                let mut current_parent = bone.1;
                for _ in 0..self.bones[object_index].len() {
                    if current_parent == -1 {
                        break;
                    }
                    let current_parent_bone = self.bones[object_index][current_parent as usize];
                    position_added += current_parent_bone.0.position;
                    rotation_added += current_parent_bone.0.rotation;
                    current_parent = current_parent_bone.1;
                }
            }
            bones.push(transforms::create_transforms(
                position_added.into(),
                rotation_added.into(),
                bone.0.scale.into()
            ).into());
        }
        self.init.queue.write_buffer(&self.bone_buffers[object_index], 0, bytemuck::cast_slice(&bones));
    }

    pub fn set_world(&mut self, world: World) {
        self.world = world;

        self.vertex_buffers.clear();
        self.uniform_bind_groups.clear();
        self.num_vertices.clear();

        for texture in self.world.get_textures() {
            self.textures.insert(texture.to_string(), TextureObject::create(texture, &self.init));
        }

        for object in self.world.get_objects().iter().enumerate() {
            let meshes = object.1.get_vertices();
            let materials = object.1.get_materials();
            let mut bones: Vec<[[f32; 4]; 4]> = Vec::new();
            self.vertex_buffers.push(Vec::new());
            self.uniform_bind_groups.push(Vec::new());
            self.num_vertices.push(Vec::new());

            let bone_transforms = object.1.get_bones();
            self.bones.push(bone_transforms.clone());

            for bone in bone_transforms {
                bones.push(transforms::create_transforms(
                    bone.0.position.into(),
                    bone.0.rotation.into(),
                    bone.0.scale.into()
                ).into());
            }

            let bone_buffer;
            if bones.len() > 0 {
                println!("{}", object.0);
                self.shader_type.push(ShaderType::DisplacementBones);
                bone_buffer = self.init.device.create_buffer(&wgpu::BufferDescriptor{
                    label: Some("Bone Buffer"),
                    size: (bones.len() * std::mem::size_of::<Matrix4<f32>>()) as u64,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                self.init.queue.write_buffer(&bone_buffer, 0, bytemuck::cast_slice(&bones));
            } else {
                self.shader_type.push(ShaderType::Displacement);
                bone_buffer = self.init.device.create_buffer(&wgpu::BufferDescriptor{
                    label: Some("Bone Buffer"),
                    size: 16,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
            }

            self.bone_buffers.push(bone_buffer);

            let model_uniform_buffer: wgpu::Buffer = self.init.device.create_buffer(&wgpu::BufferDescriptor{
                label: Some("Vertex Uniform Buffer"),
                size: 128,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let model_mat = transforms::create_transforms(
                [
                        object.1.get_position().x,
                        object.1.get_position().y,
                        object.1.get_position().z
                    ], [
                        object.1.get_rotation().x,
                        object.1.get_rotation().y,
                        object.1.get_rotation().z
                    ], [
                        object.1.get_scale().x,
                        object.1.get_scale().y,
                        object.1.get_scale().z
                    ]
            );
            let normal_mat = (model_mat.invert().unwrap()).transpose();

            let model_ref:&[f32; 16] = model_mat.as_ref();
            let normal_ref:&[f32; 16] = normal_mat.as_ref();
            self.init.queue.write_buffer(&model_uniform_buffer, 0, bytemuck::cast_slice(model_ref));
            self.init.queue.write_buffer(&model_uniform_buffer, 64, bytemuck::cast_slice(normal_ref));

            for (vertices, material_name) in meshes {
                let material_found;
                let bytes_filtered: Vec<u8> = material_name.bytes().filter(|c| c > &(31 as u8)).collect();
                let material_string = String::from_utf8(bytes_filtered).unwrap();

                if let Some(material) = materials.get(&material_string) {
                    material_found = material;
                } else {
                    material_found = &materials.get("default").unwrap();
                }

                let material_found_texture = material_found.get_texture();
                let material_found_displacement = material_found.get_displacement();

                println!("loading: {} from: {}", material_found_texture, material_string);

                let texture_object;
                if let Some(texture) = self.textures.get(material_found_texture) {
                    texture_object = texture;
                } else { continue; }

                let texture_object_displacement;
                if let Some(texture_displacement_name) = material_found_displacement {
                    if let Some(texture_displacement) = self.textures.get(texture_displacement_name) {
                        texture_object_displacement = Some(texture_displacement);
                    } else { texture_object_displacement = None; }
                } else { texture_object_displacement = None; }

                let uniform_bind_group;
                let vertex_buffer;
                if let Some(texture_displacement) = texture_object_displacement {
                    (uniform_bind_group, vertex_buffer) = Self::create_buffer_displacement(
                        &self.init, &self.uniform_bind_group_layout,
                        &self.vertex_uniform_buffer, &self.fragment_uniform_buffer, &model_uniform_buffer,
                        &self.bone_buffers[object.0], &texture_displacement.texture, texture_displacement.texture_size,
                        &texture_displacement.texture_rgba,
                        texture_displacement.texture_width, texture_displacement.texture_height,
                        &texture_object.texture, texture_object.texture_size, &texture_object.texture_rgba,
                        texture_object.texture_width, texture_object.texture_height, vertices.len(),
                    );
                } else {
                    if let Some(texture_displacement) = self.textures.get("textures/displacement.png") {
                        (uniform_bind_group, vertex_buffer) = Self::create_buffer_displacement(
                            &self.init, &self.uniform_bind_group_layout,
                            &self.vertex_uniform_buffer, &self.fragment_uniform_buffer, &model_uniform_buffer,
                            &self.bone_buffers[object.0], &texture_displacement.texture, texture_displacement.texture_size,
                            &texture_displacement.texture_rgba,
                            texture_displacement.texture_width, texture_displacement.texture_height,
                            &texture_object.texture, texture_object.texture_size, &texture_object.texture_rgba,
                            texture_object.texture_width, texture_object.texture_height, vertices.len(),
                        );
                    } else { continue; }
                }

                self.vertex_buffers[object.0].push(vertex_buffer);
                self.uniform_bind_groups[object.0].push(uniform_bind_group);

                self.num_vertices[object.0].push(vertices.len() as u32);
                self.init.queue.write_buffer(
                    &self.vertex_buffers[object.0][self.vertex_buffers[object.0].len() - 1], 0, 
                    bytemuck::cast_slice(vertices)
                );
            }

            self.model_uniform_buffers.push(model_uniform_buffer);
        }

        if self.world.get_cameras().len() > self.current_camera {
            let object_position = self.world.get_camera(self.current_camera).get_position();
            self.player.camera.position.x = object_position.x as f32;
            self.player.camera.position.y = object_position.y as f32;
            self.player.camera.position.z = object_position.z as f32;
            self.player.camera.rotation = self.world.get_camera(self.current_camera).get_rotation();
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        //let output = self.init.surface.get_current_frame()?.output;
        let output = self.init.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let depth_texture = self.init.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.init.config.width,
                height: self.init.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format:wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .init.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.2,
                            g: 0.247,
                            b: 0.314,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })], 
                //depth_stencil_attachment: None,
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: false,
                    }),
                    stencil_ops: None,
                }),
            });

            let mut current_shader = ShaderType::Displacement;
            render_pass.set_pipeline(&self.pipeline_displacement);

            for mesh in 0..self.vertex_buffers.len() {
                let shader_type = &self.shader_type[mesh];
                if shader_type != &current_shader {
                    match shader_type {
                        ShaderType::Displacement => {
                            render_pass.set_pipeline(&self.pipeline_displacement);
                            current_shader = ShaderType::Displacement;
                        }
                        ShaderType::DisplacementBones => {
                            render_pass.set_pipeline(&self.pipeline_displacement_bones);
                            current_shader = ShaderType::DisplacementBones;
                        }
                    }
                }

                for i in 0..self.vertex_buffers[mesh].len() {
                    render_pass.set_vertex_buffer(0, self.vertex_buffers[mesh][i].slice(..));           
                    render_pass.set_bind_group(0, &self.uniform_bind_groups[mesh][i], &[]);
                    render_pass.draw(0..self.num_vertices[mesh][i], 0..1);
                }
            }
        }

        self.init.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}