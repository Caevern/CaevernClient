use cgmath::*;
use rust_embed::RustEmbed;
use std::collections::HashMap;
use std::time::Instant;
use std::{f32, println};
use std::ops::Deref;

use crate::ALLOCATOR;
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
use crate::renderer::transforms::{fov_to_projection, get_eye_view_matrix, pose_to_view_matrix};
use crate::renderer::vertex::Vertex;
use crate::renderer::{transform, transforms, vertex};
use crate::setup::fonts::load_font_uvs;
use crate::world::object::ObjectType;
use crate::world::objects::player::Player;
use crate::world::objects::text;
use crate::xr::xr_manager::XRManager;

pub struct RendererOpenXR {
    pub init: XRManager,
    pub buffer_collection: BufferCollection,

    pipeline_displacement: wgpu::RenderPipeline,
    pipeline_displacement_bones: wgpu::RenderPipeline,

    frame: usize,
}
impl RendererOpenXR {
    pub async fn new(init: XRManager) -> Self {
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
            wgpu::TextureFormat::Bgra8UnormSrgb,
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
            wgpu::TextureFormat::Bgra8UnormSrgb,
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
                if object_type == ObjectType::TabletMenu
                    || object_type == ObjectType::TabletMenuButton
                {
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
                        &self.buffer_collection.model_uniform_buffers[object.buffer_bindings.model_uniform_buffer],
                        0,
                        bytemuck::cast_slice(model_ref),
                    );
                    self.init.queue.write_buffer(
                        &self.buffer_collection.model_uniform_buffers[object.buffer_bindings.model_uniform_buffer],
                        64,
                        bytemuck::cast_slice(normal_ref),
                    );
                }
            }
        } else if menu_tablet_state == 3 {
            for object_index in 0..world.get_objects().len() {
                let object = world.get_object(object_index);
                let object_type = object.get_object_type();
                if object_type == ObjectType::TabletMenu
                    || object_type == ObjectType::TabletMenuButton
                {
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
                        &self.buffer_collection.model_uniform_buffers[object.buffer_bindings.model_uniform_buffer],
                        0,
                        bytemuck::cast_slice(model_ref),
                    );
                    self.init.queue.write_buffer(
                        &self.buffer_collection.model_uniform_buffers[object.buffer_bindings.model_uniform_buffer],
                        64,
                        bytemuck::cast_slice(normal_ref),
                    );
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
                    //self.bones[i][skeleton["head"]].0.rotation.x = -player.camera.rotation.x;
                    //self.bones[i][skeleton["arm_right"]].0.rotation.z = -player.camera.rotation.x;
                    /*self.bones[i][skeleton["head"]].0.rotation.y =
                    -player.camera.rotation.y - 1.57079633;*/
                    // TODO: make the local character have this dissabled by default.
                    /*self.bones[i][skeleton["neck"]].0.scale = [0.0, 0.0, 0.0].into();
                    update_bone(
                        &world,
                        object_index,
                        object.buffer_bindings,
                        skeleton["head"],
                        &mut self.buffer_collection,
                        &self.init.queue,
                    );*/
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

                /*let _ = self
                .data_thread_tx
                .send(UserUpdate::SendUserPosition(Transform {
                    position: position.into(),
                    rotation: Vector3::new(
                        -player.camera.rotation.x,
                        rotation[1],
                        -player.camera.rotation.z,
                    ),
                    scale: Vector3::new(1.0, 1.0, 1.0),
                }));*/

                let model_mat =
                    transforms::create_transforms(position, rotation, object.get_scale().into());
                let normal_mat = (model_mat.invert().unwrap()).transpose();

                let model_ref: &[f32; 16] = model_mat.as_ref();
                let normal_ref: &[f32; 16] = normal_mat.as_ref();

                self.init.queue.write_buffer(
                    &self.buffer_collection.model_uniform_buffers[object.buffer_bindings.model_uniform_buffer],
                    0,
                    bytemuck::cast_slice(model_ref),
                );
                self.init.queue.write_buffer(
                    &self.buffer_collection.model_uniform_buffers[object.buffer_bindings.model_uniform_buffer],
                    64,
                    bytemuck::cast_slice(normal_ref),
                );
            }
        }

        // update skybox positions
        /*if self.frame % 10 == 0 {
            let grabbable_object_index =
                raycast_grab(world.get_objects(), player.camera.position, forward, 5);

            if grabbable_object_index > 0 {
                let y_rotation = world.get_objects()[&grabbable_object_index]
                    .get_rotation()
                    .y;
                //world.objects[&grabbable_object_index].set_rotation_y(y_rotation + 0.1);
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
                    &self.fragment_uniform_buffer,
                    16,
                    bytemuck::cast_slice(eye_position),
                );
                self.init.queue.write_buffer(
                    &self.model_uniform_buffers[grabbable_object_index],
                    0,
                    bytemuck::cast_slice(model_ref),
                );
                self.init.queue.write_buffer(
                    &self.model_uniform_buffers[grabbable_object_index],
                    64,
                    bytemuck::cast_slice(normal_ref),
                );
            }
        }*/

        //let up_direction = cgmath::Vector3::unit_y();
        //let camera_position = Point3 {
        //    x: player.camera.position.x,
        //    y: player.camera.position.y,
        //    z: player.camera.position.z,
        //};
        //let (view_mat, project_mat, _) = transforms::create_view_rotation(
        //    camera_position,
        //    player.camera.rotation.y,
        //    player.camera.rotation.x,
        //    up_direction,
        //    1.0,
        //);

        //let view_project_mat = project_mat * view_mat;
        //let view_projection_ref: &[f32; 16] = view_project_mat.as_ref();

        //self.init.queue.write_buffer(
        //    &self.buffer_collection.vertex_uniform_buffer,
        //    64,
        //    bytemuck::cast_slice(view_projection_ref),
        //);

        // update ingame fps label when menu tablet is enabled
        if menu_tablet_state == 1 && self.frame % 60 == 0 {
            for (_, object) in world.get_objects() {
                match object.get_tag() {
                    "fps_label" => {
                        let fps_label = text::create_plane_with_text(
                            (-0.5, -0.3, -0.02),
                            (0.02, 0.02, 1.0),
                            &self.buffer_collection.font_maps["NotoSansJP"],
                            [1.0, 1.0, 1.0],
                            &format!("FPS: {}", (1.0 / frame_time).round()),
                        );
                        let meshes = vertex::create_vertices(&fps_label);
                        for (vertices, _) in meshes {
                            self.buffer_collection.num_vertices
                                [object.buffer_bindings.num_vertices] = vec![vertices.len() as u32];
                            let vertex_buffer =
                                self.init.device.create_buffer(&wgpu::BufferDescriptor {
                                    label: Some("Vertex Buffer"),
                                    size: (size_of::<Vertex>() * vertices.len()) as u64,
                                    usage: wgpu::BufferUsages::VERTEX
                                        | wgpu::BufferUsages::COPY_DST,
                                    mapped_at_creation: false,
                                });
                            self.buffer_collection.vertex_buffers
                                [object.buffer_bindings.vertex_buffer] = vec![vertex_buffer];
                            self.init.queue.write_buffer(
                                &self.buffer_collection.vertex_buffers
                                    [object.buffer_bindings.vertex_buffer][0],
                                0,
                                bytemuck::cast_slice(&vertices),
                            );
                        }
                    }
                    "ram_label" => {
                        let ram_label = text::create_plane_with_text(
                            (-0.5, -0.2, -0.02),
                            (0.02, 0.02, 1.0),
                            &self.buffer_collection.font_maps["NotoSansJP"],
                            [1.0, 1.0, 1.0],
                            &format!("RAM: {:.2} MB", ALLOCATOR.allocated() as f32 / 1000000.0),
                        );
                        let meshes = vertex::create_vertices(&ram_label);
                        for (vertices, _) in meshes {
                            self.buffer_collection.num_vertices
                                [object.buffer_bindings.num_vertices] = vec![vertices.len() as u32];
                            let vertex_buffer =
                                self.init.device.create_buffer(&wgpu::BufferDescriptor {
                                    label: Some("Vertex Buffer"),
                                    size: (size_of::<Vertex>() * vertices.len()) as u64,
                                    usage: wgpu::BufferUsages::VERTEX
                                        | wgpu::BufferUsages::COPY_DST,
                                    mapped_at_creation: false,
                                });
                            self.buffer_collection.vertex_buffers
                                [object.buffer_bindings.vertex_buffer] = vec![vertex_buffer];
                            self.init.queue.write_buffer(
                                &self.buffer_collection.vertex_buffers
                                    [object.buffer_bindings.vertex_buffer][0],
                                0,
                                bytemuck::cast_slice(&vertices),
                            );
                        }
                    }
                    _ => {
                        continue;
                    }
                };
            }
        }

        self.frame += 1;
    }

    // TODO: For parity, move out of this struct
    fn render_scene(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView
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
                render_pass.set_vertex_buffer(
                    0,
                    self.buffer_collection.vertex_buffers[mesh][i].slice(..),
                );

                render_pass.set_bind_group(
                    0,
                    &self.buffer_collection.uniform_bind_groups[mesh][i],
                    &[],
                );

                render_pass.draw(
                    0..self.buffer_collection.num_vertices[mesh][i],
                    0..1,
                );
            }
        }
    }

    fn render_frame(&mut self, player: &Player) -> Result<(), openxr::sys::Result> {
        let frame_state = self.init.frame_waiter.wait()?;

        self.init.frame_stream.begin()?;

        if frame_state.should_render {
            let (_view_flags, views) = self.init.session.locate_views(
                self.init.view_config,
                frame_state.predicted_display_time,
                &self.init.reference_space,
            )?;

            assert_eq!(views.len(), 2);

            let mut encoder = self.init.device.create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("XR Render Encoder"),
                },
            );

            let left_proj = fov_to_projection(views[0].fov, 0.1, 1000.0);
            let left_view = get_eye_view_matrix(
                player.camera.position,
                player.camera.rotation.y,
                views[0].pose,
            );
            let left_view_proj = left_proj * left_view;
            let left_view_proj_ref: &[f32; 16] = left_view_proj.as_ref();
            self.init.queue.write_buffer(
                &self.buffer_collection.vertex_uniform_buffer,
                64,
                bytemuck::cast_slice(left_view_proj_ref),
            );
            let left_index = self.init.swapchains[0].acquire_image()?;
            self.init.swapchains[0].wait_image(openxr::Duration::from_nanos(1_000_000_000))?;
            let left_color_view = &self.init.swapchain_views[0][left_index as usize];

            self.render_scene(
                &mut encoder,
                left_color_view,
                &self.init.depth_views[0]
            );

            let right_proj = fov_to_projection(views[1].fov, 0.1, 1000.0);
            let right_view = get_eye_view_matrix(
                player.camera.position,
                player.camera.rotation.y,
                views[1].pose,
            );
            let right_view_proj = right_proj * right_view;
            let right_vp_ref: &[f32; 16] = right_view_proj.as_ref();
            self.init.queue.write_buffer(
                &self.buffer_collection.vertex_uniform_buffer,
                64,
                bytemuck::cast_slice(right_vp_ref),
            );
            let right_index = self.init.swapchains[1].acquire_image()?;
            self.init.swapchains[1].wait_image(openxr::Duration::from_nanos(1_000_000_000))?;
            let right_color_view = &self.init.swapchain_views[1][right_index as usize];

            self.render_scene(
                &mut encoder,
                right_color_view,
                &self.init.depth_views[1]
            );

            self.init.queue.submit(Some(encoder.finish()));

            self.init.swapchains[0].release_image()?;
            self.init.swapchains[1].release_image()?;

            let left_rect = openxr::Rect2Di {
                offset: openxr::Offset2Di { x: 0, y: 0 },
                extent: openxr::Extent2Di {
                    width: self.init.views[0].recommended_image_rect_width as i32,
                    height: self.init.views[0].recommended_image_rect_height as i32,
                },
            };

            let right_rect = openxr::Rect2Di {
                offset: openxr::Offset2Di { x: 0, y: 0 },
                extent: openxr::Extent2Di {
                    width: self.init.views[1].recommended_image_rect_width as i32,
                    height: self.init.views[1].recommended_image_rect_height as i32,
                },
            };

            let proj_views = [
                openxr::CompositionLayerProjectionView::new()
                    .pose(views[0].pose)
                    .fov(views[0].fov)
                    .sub_image(
                        openxr::SwapchainSubImage::new()
                            .swapchain(&self.init.swapchains[0])
                            .image_rect(left_rect)
                            .image_array_index(0),
                    ),
                openxr::CompositionLayerProjectionView::new()
                    .pose(views[1].pose)
                    .fov(views[1].fov)
                    .sub_image(
                        openxr::SwapchainSubImage::new()
                            .swapchain(&self.init.swapchains[1])
                            .image_rect(right_rect)
                            .image_array_index(0),
                    ),
            ];

            let projection_layer = openxr::CompositionLayerProjection::new()
                .space(&self.init.reference_space)
                .views(&proj_views);

            let layer_base: &openxr::CompositionLayerBase<openxr::Vulkan> = &projection_layer;

            self.init.frame_stream.end(
                frame_state.predicted_display_time,
                openxr::EnvironmentBlendMode::OPAQUE,
                &[layer_base],
            )?;
        } else {
            self.init.frame_stream.end(
                frame_state.predicted_display_time,
                openxr::EnvironmentBlendMode::OPAQUE,
                &[],
            )?;
        }

        Ok(())
    }

    pub fn run_frame_loop(&mut self, engine: Engine) {
        let mut engine = engine;

        let mut last_frame_time = Instant::now();

        loop {
            let now = Instant::now();
            let frame_time = now.duration_since(last_frame_time).as_secs_f32();
            last_frame_time = now;

            self.init.poll_events().ok();

            self.update(frame_time, 0, &mut engine);

            if self.init.get_session_running() {
                if let Err(e) = self.render_frame(&engine.player) {
                    eprintln!("XR Frame Error: {:?}", e);
                }
            } else {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    }
}
