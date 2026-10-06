use cgmath::*;
use std::collections::HashMap;
use std::f32;
use std::sync::Arc;
use winit::window::Window;

use crate::ALLOCATOR;
use crate::game::update_bone::update_bone;
use crate::game::update_bones::update_bones;
use crate::game::update_game::Engine;
use crate::interract::raycast::raycast_grab;
use crate::renderer::buffer_collection::BufferCollection;
use crate::renderer::buffers::bind_group_layout::create_bind_group_layout;
use crate::renderer::buffers::uniform_buffers::{
    create_fragment_uniform_buffer, create_vertex_uniform_buffer,
};
use crate::renderer::default_elements::register_default_textures;
use crate::renderer::pipelines::displacement_default::create_pipeline;
use crate::renderer::shader_type::ShaderType;
use crate::renderer::texture_object::TextureObject;
use crate::renderer::vertex::Vertex;
use crate::renderer::{init_wgpu, transforms, vertex};
use crate::setup::fonts::load_font_uvs;
use crate::world::object::ObjectType;

pub struct RendererWindowed<'window> {
    pub init: init_wgpu::InitWgpu<'window>,
    pub buffer_collection: BufferCollection,

    pipeline_displacement: wgpu::RenderPipeline,
    pipeline_displacement_bones: wgpu::RenderPipeline,

    frame: usize,
}
impl<'window> RendererWindowed<'window> {
    pub async fn new(window: &Arc<Window>) -> Self {
        let init = init_wgpu::InitWgpu::init_wgpu(window).await;

        let uniform_bind_group_layout: wgpu::BindGroupLayout =
            create_bind_group_layout(&init.device);

        let pipeline_layout = init
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[Some(&uniform_bind_group_layout)],
                immediate_size: 0,
            });

        let shader_displacement = init
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../shaders/displacement.wgsl").into(),
                ),
            });

        let pipeline_displacement = create_pipeline(
            &init.device,
            &pipeline_layout,
            &shader_displacement,
            init.config.format,
        );

        let shader_displacement_bones =
            init.device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Shader"),
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!("../shaders/displacement_bones.wgsl").into(),
                    ),
                });

        let pipeline_displacement_bones = create_pipeline(
            &init.device,
            &pipeline_layout,
            &shader_displacement_bones,
            init.config.format,
        );

        let vertex_uniform_buffer = create_vertex_uniform_buffer(&init.device);
        let fragment_uniform_buffer = create_fragment_uniform_buffer(&init.device);

        let model_mat =
            transforms::create_transforms([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let normal_mat = (model_mat.invert().unwrap()).transpose();

        let model_ref: &[f32; 16] = model_mat.as_ref();
        let normal_ref: &[f32; 16] = normal_mat.as_ref();
        init.queue
            .write_buffer(&vertex_uniform_buffer, 0, bytemuck::cast_slice(model_ref));
        init.queue.write_buffer(
            &vertex_uniform_buffer,
            128,
            bytemuck::cast_slice(normal_ref),
        );

        let mut textures: HashMap<String, TextureObject> = HashMap::new();
        register_default_textures(&mut textures, &init.device);

        let mut font_maps: HashMap<String, HashMap<String, (f32, f32, f32, f32, f32)>> =
            HashMap::new();

        font_maps.insert(
            "NotoSansJP".to_string(),
            load_font_uvs("fonts/NotoSansJP.ttf"),
        );

        let buffer_collection = BufferCollection {
            textures,
            font_maps,

            final_matrices: Vec::new(),

            bones: Vec::new(),
            bone_buffers: Vec::new(),

            shader_type: Vec::new(),

            vertex_buffers: Vec::new(),
            uniform_bind_groups: Vec::new(),
            num_vertices: Vec::new(),

            model_uniform_buffers: Vec::new(),

            uniform_bind_group_layout,
            vertex_uniform_buffer,
            fragment_uniform_buffer,
        };

        Self {
            init,
            buffer_collection,

            pipeline_displacement,
            pipeline_displacement_bones,

            frame: 0,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.init.instance.poll_all(true);
            self.init.size = new_size;
            self.init.config.width = new_size.width;
            self.init.config.height = new_size.height;
            self.init
                .surface
                .configure(&self.init.device, &self.init.config);
        }
    }

    pub fn update(&mut self, frame_time: f32, menu_tablet_state: usize, engine: &mut Engine) {
        let mut world = engine.world_rc.borrow_mut();

        let player = &engine.player;
        let forward = Vector3::new(
            player.camera.rotation.y.cos() * player.camera.rotation.x.cos(),
            player.camera.rotation.x.sin(),
            player.camera.rotation.y.sin() * player.camera.rotation.x.cos(),
        )
        .normalize();

        if menu_tablet_state == 2 {
            for object_index in 0..world.get_objects().len() {
                let object = world.get_object(object_index);
                let object_type = object.get_object_type();
                if object_type == ObjectType::TabletMenu {
                    let model_mat = transforms::create_transforms(
                        [
                            player.camera.position.x + forward.x,
                            player.camera.position.y + forward.y,
                            player.camera.position.z + forward.z,
                        ],
                        [
                            -player.camera.rotation.x,
                            -player.camera.rotation.y + std::f32::consts::FRAC_PI_2,
                            -player.camera.rotation.z,
                        ],
                        [1.0, 1.0, 1.0],
                    );
                    let normal_mat = (model_mat.invert().unwrap()).transpose();

                    let model_ref: &[f32; 16] = model_mat.as_ref();
                    let normal_ref: &[f32; 16] = normal_mat.as_ref();

                    self.init.queue.write_buffer(
                        &self.buffer_collection.model_uniform_buffers
                            [object.buffer_bindings.model_uniform_buffer],
                        0,
                        bytemuck::cast_slice(model_ref),
                    );
                    self.init.queue.write_buffer(
                        &self.buffer_collection.model_uniform_buffers
                            [object.buffer_bindings.model_uniform_buffer],
                        64,
                        bytemuck::cast_slice(normal_ref),
                    );

                    if let Some(canvas) = &object.canvas {
                        for child in canvas.children.iter() {
                            let child_id = child.get_id();
                            let child_object = world.get_object(child_id);

                            self.init.queue.write_buffer(
                                &self.buffer_collection.model_uniform_buffers
                                    [child_object.buffer_bindings.model_uniform_buffer],
                                0,
                                bytemuck::cast_slice(model_ref),
                            );
                            self.init.queue.write_buffer(
                                &self.buffer_collection.model_uniform_buffers
                                    [child_object.buffer_bindings.model_uniform_buffer],
                                64,
                                bytemuck::cast_slice(normal_ref),
                            );
                        }
                    }
                }
            }
        } else if menu_tablet_state == 3 {
            for object_index in 0..world.get_objects().len() {
                let object = world.get_object(object_index);
                let object_type = object.get_object_type();
                if object_type == ObjectType::TabletMenu {
                    let model_mat = transforms::create_transforms(
                        [0.0, -10.0, 0.0],
                        [
                            -player.camera.rotation.x,
                            -player.camera.rotation.y + std::f32::consts::FRAC_PI_2,
                            -player.camera.rotation.z,
                        ],
                        [1.0, 1.0, 1.0],
                    );
                    let normal_mat = (model_mat.invert().unwrap()).transpose();

                    let model_ref: &[f32; 16] = model_mat.as_ref();
                    let normal_ref: &[f32; 16] = normal_mat.as_ref();

                    self.init.queue.write_buffer(
                        &self.buffer_collection.model_uniform_buffers
                            [object.buffer_bindings.model_uniform_buffer],
                        0,
                        bytemuck::cast_slice(model_ref),
                    );
                    self.init.queue.write_buffer(
                        &self.buffer_collection.model_uniform_buffers
                            [object.buffer_bindings.model_uniform_buffer],
                        64,
                        bytemuck::cast_slice(normal_ref),
                    );

                    if let Some(canvas) = &object.canvas {
                        for child in canvas.children.iter() {
                            let child_id = child.get_id();
                            let child_object = world.get_object(child_id);

                            self.init.queue.write_buffer(
                                &self.buffer_collection.model_uniform_buffers
                                    [child_object.buffer_bindings.model_uniform_buffer],
                                0,
                                bytemuck::cast_slice(model_ref),
                            );
                            self.init.queue.write_buffer(
                                &self.buffer_collection.model_uniform_buffers
                                    [child_object.buffer_bindings.model_uniform_buffer],
                                64,
                                bytemuck::cast_slice(normal_ref),
                            );
                        }
                    }
                }
            }
        }

        for object_index in 0..world.get_objects().len() {
            let object = world.get_object(object_index);
            let object_type = object.get_object_type();
            if object_type == ObjectType::Skybox {
                let model_mat = transforms::create_transforms(
                    [
                        player.camera.position.x,
                        player.camera.position.y,
                        player.camera.position.z,
                    ],
                    [0.0, 0.0, 0.0],
                    [1.0, 1.0, 1.0],
                );
                let normal_mat = (model_mat.invert().unwrap()).transpose();

                let model_ref: &[f32; 16] = model_mat.as_ref();
                let normal_ref: &[f32; 16] = normal_mat.as_ref();
                let eye_position: &[f32; 3] = &player.camera.position.into();
                self.init.queue.write_buffer(
                    &self.buffer_collection.fragment_uniform_buffer,
                    16,
                    bytemuck::cast_slice(eye_position),
                );
                self.init.queue.write_buffer(
                    &self.buffer_collection.model_uniform_buffers
                        [object.buffer_bindings.model_uniform_buffer],
                    0,
                    bytemuck::cast_slice(model_ref),
                );
                self.init.queue.write_buffer(
                    &self.buffer_collection.model_uniform_buffers
                        [object.buffer_bindings.model_uniform_buffer],
                    64,
                    bytemuck::cast_slice(normal_ref),
                );
            } else if object_type == ObjectType::SkinnedMesh {
                if self.frame < 60 {
                    let skeleton = object.get_skeleton();
                    self.buffer_collection.bones[object.buffer_bindings.bones][skeleton["neck"]]
                        .0
                        .scale = [0.0, 0.0, 0.0].into();
                    update_bone(
                        &world,
                        object_index,
                        object.buffer_bindings,
                        skeleton["head"],
                        &mut self.buffer_collection,
                        &self.init.queue,
                    );
                }

                let position = [
                    object.get_position().x + player.camera.position.x
                        - player.camera.rotation.y.cos() * 0.1,
                    object.get_position().y + player.camera.position.y,
                    object.get_position().z + player.camera.position.z
                        - player.camera.rotation.y.sin() * 0.1,
                ];
                let rotation = [
                    object.get_rotation().x,
                    object.get_rotation().y - player.camera.rotation.y + 1.57079633,
                    object.get_rotation().z,
                ];

                let model_mat =
                    transforms::create_transforms(position, rotation, object.get_scale().into());
                let normal_mat = (model_mat.invert().unwrap()).transpose();

                let model_ref: &[f32; 16] = model_mat.as_ref();
                let normal_ref: &[f32; 16] = normal_mat.as_ref();

                self.init.queue.write_buffer(
                    &self.buffer_collection.model_uniform_buffers
                        [object.buffer_bindings.model_uniform_buffer],
                    0,
                    bytemuck::cast_slice(model_ref),
                );
                self.init.queue.write_buffer(
                    &self.buffer_collection.model_uniform_buffers
                        [object.buffer_bindings.model_uniform_buffer],
                    64,
                    bytemuck::cast_slice(normal_ref),
                );
            }
        }

        if self.frame % 20 == 1 {
            for object_index in 0..world.get_objects().len() {
                let object = world.get_object(object_index);
                if object.get_object_type() == ObjectType::SkinnedMesh {
                    update_bones(
                        &world,
                        object_index,
                        object.buffer_bindings,
                        &mut self.buffer_collection,
                        &self.init.queue,
                    );
                }
            }
        }

        // update skybox positions
        if self.frame % 10 == 0 {
            let grabbable_object_index =
                raycast_grab(world.get_objects(), player.camera.position, forward, 5);

            if grabbable_object_index > 0 {
                let grabbable_object = world.get_object_mut(grabbable_object_index);
                let y_rotation = grabbable_object.get_rotation().y;
                grabbable_object.set_rotation_y(y_rotation + 0.1);
                let model_mat = transforms::create_transforms(
                    [0.0, 0.0, 0.0],
                    [0.0, y_rotation + 0.1, 0.0],
                    [1.0, 1.0, 1.0],
                );
                let normal_mat = (model_mat.invert().unwrap()).transpose();

                let model_ref: &[f32; 16] = model_mat.as_ref();
                let normal_ref: &[f32; 16] = normal_mat.as_ref();
                let eye_position: &[f32; 3] = &player.camera.position.into();
                self.init.queue.write_buffer(
                    &self.buffer_collection.fragment_uniform_buffer,
                    16,
                    bytemuck::cast_slice(eye_position),
                );
                self.init.queue.write_buffer(
                    &self.buffer_collection.model_uniform_buffers
                        [grabbable_object.buffer_bindings.model_uniform_buffer],
                    0,
                    bytemuck::cast_slice(model_ref),
                );
                self.init.queue.write_buffer(
                    &self.buffer_collection.model_uniform_buffers
                        [grabbable_object.buffer_bindings.model_uniform_buffer],
                    64,
                    bytemuck::cast_slice(normal_ref),
                );
            }
        }

        let up_direction = cgmath::Vector3::unit_y();
        let camera_position = Point3 {
            x: player.camera.position.x,
            y: player.camera.position.y,
            z: player.camera.position.z,
        };
        let (view_mat, project_mat, _) = transforms::create_view_rotation(
            camera_position,
            player.camera.rotation.y,
            player.camera.rotation.x,
            up_direction,
            self.init.config.width as f32 / self.init.config.height as f32,
        );

        let view_project_mat = project_mat * view_mat;
        let view_projection_ref: &[f32; 16] = view_project_mat.as_ref();

        self.init.queue.write_buffer(
            &self.buffer_collection.vertex_uniform_buffer,
            64,
            bytemuck::cast_slice(view_projection_ref),
        );

        // update ingame fps label when menu tablet is enabled
        if menu_tablet_state == 1 && self.frame % 60 == 0 {
            for (_, object) in world.get_objects() {
                if let Some(canvas) = &object.canvas {
                    for child in &canvas.children {
                        if !child.needs_update() { continue; }
                        let child_id = child.get_id();
                        let child_object = world.get_object(child_id);

                        let text_placeholder = child.get_text_placeholder();

                        let text_formatted = &text_placeholder.replace(
                            "[RAM]",
                            &format!("{:.2} MB", ALLOCATOR.allocated() as f32 / 1000000.0)
                        );
                        let text_formatted = &text_formatted.replace(
                            "[FPS]",
                            &format!("{}", (1.0 / frame_time).round())
                        );
                        let text_formatted = &text_formatted.replace(
                            "[CLOCK]",
                            &chrono::Local::now().format("%H:%M:%S").to_string()
                        );

                        let meshes = child.get_meshes_from_text(text_formatted);

                        let meshes_created = vertex::create_vertices(&meshes);
                        for (vertices, _) in meshes_created {
                            self.buffer_collection.num_vertices
                                [child_object.buffer_bindings.num_vertices] = vec![vertices.len() as u32];
                            let vertex_buffer =
                                self.init.device.create_buffer(&wgpu::BufferDescriptor {
                                    label: Some("Vertex Buffer"),
                                    size: (size_of::<Vertex>() * vertices.len()) as u64,
                                    usage: wgpu::BufferUsages::VERTEX
                                        | wgpu::BufferUsages::COPY_DST,
                                    mapped_at_creation: false,
                                });
                            self.buffer_collection.vertex_buffers
                                [child_object.buffer_bindings.vertex_buffer] = vec![vertex_buffer];
                            self.init.queue.write_buffer(
                                &self.buffer_collection.vertex_buffers
                                    [child_object.buffer_bindings.vertex_buffer][0],
                                0,
                                bytemuck::cast_slice(&vertices),
                            );
                        }
                    }
                }
            }
        }

        self.frame += 1;
    }

    // TODO: For parity, move out of this struct
    fn render_scene(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.2,
                        g: 0.247,
                        b: 0.314,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Discard,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });

        let mut current_shader = ShaderType::Displacement;
        render_pass.set_pipeline(&self.pipeline_displacement);

        for mesh in 0..self.buffer_collection.vertex_buffers.len() {
            let shader_type = &self.buffer_collection.shader_type[mesh];

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

            for i in 0..self.buffer_collection.vertex_buffers[mesh].len() {
                render_pass
                    .set_vertex_buffer(0, self.buffer_collection.vertex_buffers[mesh][i].slice(..));

                render_pass.set_bind_group(
                    0,
                    &self.buffer_collection.uniform_bind_groups[mesh][i],
                    &[],
                );

                render_pass.draw(0..self.buffer_collection.num_vertices[mesh][i], 0..1);
            }
        }
    }

    pub fn render(&mut self, depth_texture: &wgpu::Texture) -> Result<(), ()> {
        let output = match self.init.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,

            wgpu::CurrentSurfaceTexture::Outdated => {
                self.init
                    .surface
                    .configure(&self.init.device, &self.init.config);
                return Ok(());
            }

            wgpu::CurrentSurfaceTexture::Lost => {
                return Ok(());
            }

            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => {
                return Ok(());
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.init
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Window Render Encoder"),
                });

        self.render_scene(&mut encoder, &view, &depth_view);

        self.init.queue.submit(Some(encoder.finish()));
        self.init.queue.present(output);

        Ok(())
    }
}
