use cgmath::*;
use rust_embed::RustEmbed;
use std::collections::HashMap;
use std::{f32, println};

use crate::ALLOCATOR;
use crate::game::update_bone::update_bone;
use crate::game::update_game::Engine;
use crate::interract::raycast::raycast_grab;
use crate::renderer::buffers::bind_group_layout::create_bind_group_layout;
use crate::renderer::buffers::displacement_buffer::create_buffer_displacement;
use crate::renderer::buffers::uniform_buffers::{
    create_fragment_uniform_buffer, create_vertex_uniform_buffer,
};
use crate::renderer::default_elements::register_default_textures;
use crate::renderer::pipelines::displacement_default::create_pipeline;
use crate::renderer::shader_type::ShaderType;
use crate::renderer::texture_object::TextureObject;
use crate::renderer::transforms::create_transforms;
use crate::renderer::vertex::Vertex;
use crate::renderer::{transform, transforms, vertex};
use crate::setup::fonts::load_font_uvs;
use crate::world::object::{Object, ObjectType};
use crate::world::objects::text;
use crate::world::world::World;
use crate::xr::xr_manager::XRManager;

#[derive(RustEmbed)]
#[folder = "assets/"]
pub struct Assets;

pub struct RendererOpenXR {
    pub init: XRManager,

    pipeline_displacement: wgpu::RenderPipeline,
    pipeline_displacement_bones: wgpu::RenderPipeline,

    frame: usize,

    vertex_buffers: Vec<Vec<wgpu::Buffer>>,
    uniform_bind_groups: Vec<Vec<wgpu::BindGroup>>,
    num_vertices: Vec<Vec<u32>>,
    bone_buffers: Vec<wgpu::Buffer>,
    bones: Vec<Vec<(transform::Transform, transform::Transform, i64, usize)>>,
    final_marices: Vec<Vec<[[f32; 4]; 4]>>,
    shader_type: Vec<ShaderType>,

    uniform_bind_group_layout: wgpu::BindGroupLayout,
    vertex_uniform_buffer: wgpu::Buffer,
    model_uniform_buffers: Vec<wgpu::Buffer>,
    fragment_uniform_buffer: wgpu::Buffer,

    textures: HashMap<String, TextureObject>,
    font_maps: HashMap<String, HashMap<String, (f32, f32, f32, f32, f32)>>,
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

        Self {
            init,

            pipeline_displacement,
            pipeline_displacement_bones,

            frame: 0,

            vertex_buffers: Vec::new(),
            uniform_bind_groups: Vec::new(),
            num_vertices: Vec::new(),
            bone_buffers: Vec::new(),
            bones: Vec::new(),
            final_marices: Vec::new(),
            shader_type: Vec::new(),

            uniform_bind_group_layout,
            vertex_uniform_buffer,
            model_uniform_buffers: Vec::new(),
            fragment_uniform_buffer,

            textures,
            font_maps,
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
                        &self.model_uniform_buffers[object_index],
                        0,
                        bytemuck::cast_slice(model_ref),
                    );
                    self.init.queue.write_buffer(
                        &self.model_uniform_buffers[object_index],
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
                        &self.model_uniform_buffers[object_index],
                        0,
                        bytemuck::cast_slice(model_ref),
                    );
                    self.init.queue.write_buffer(
                        &self.model_uniform_buffers[object_index],
                        64,
                        bytemuck::cast_slice(normal_ref),
                    );
                }
            }
        }

        for i in 0..world.get_objects().len() {
            if world.get_object(i).get_object_type() == ObjectType::Skybox {
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
                    &self.fragment_uniform_buffer,
                    16,
                    bytemuck::cast_slice(eye_position),
                );
                self.init.queue.write_buffer(
                    &self.model_uniform_buffers[i],
                    0,
                    bytemuck::cast_slice(model_ref),
                );
                self.init.queue.write_buffer(
                    &self.model_uniform_buffers[i],
                    64,
                    bytemuck::cast_slice(normal_ref),
                );
            } else if world.get_object(i).get_object_type() == ObjectType::SkinnedMesh {
                if self.frame < 60 {
                    let skeleton = world.get_object(i).get_skeleton();
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

                let object = world.get_object(i);
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
                    &self.model_uniform_buffers[i],
                    0,
                    bytemuck::cast_slice(model_ref),
                );
                self.init.queue.write_buffer(
                    &self.model_uniform_buffers[i],
                    64,
                    bytemuck::cast_slice(normal_ref),
                );
            }
        }

        // update skybox positions
        if self.frame % 10 == 0 {
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
            1.0,
        );

        let view_project_mat = project_mat * view_mat;
        let view_projection_ref: &[f32; 16] = view_project_mat.as_ref();

        self.init.queue.write_buffer(
            &self.vertex_uniform_buffer,
            64,
            bytemuck::cast_slice(view_projection_ref),
        );

        // update ingame fps label when menu tablet is enabled
        if menu_tablet_state == 1 && self.frame % 60 == 0 {
            for (index, object) in world.get_objects() {
                match object.get_tag() {
                    "fps_label" => {
                        let fps_label = text::create_plane_with_text(
                            (-0.5, -0.3, -0.02),
                            (0.02, 0.02, 1.0),
                            &self.font_maps["NotoSansJP"],
                            [1.0, 1.0, 1.0],
                            &format!("FPS: {}", (1.0 / frame_time).round()),
                        );
                        let meshes = vertex::create_vertices(&fps_label);
                        for (vertices, _) in meshes {
                            self.num_vertices[*index] = vec![vertices.len() as u32];
                            let vertex_buffer =
                                self.init.device.create_buffer(&wgpu::BufferDescriptor {
                                    label: Some("Vertex Buffer"),
                                    size: (size_of::<Vertex>() * vertices.len()) as u64,
                                    usage: wgpu::BufferUsages::VERTEX
                                        | wgpu::BufferUsages::COPY_DST,
                                    mapped_at_creation: false,
                                });
                            self.vertex_buffers[*index] = vec![vertex_buffer];
                            self.init.queue.write_buffer(
                                &self.vertex_buffers[*index][0],
                                0,
                                bytemuck::cast_slice(&vertices),
                            );
                        }
                    }
                    "ram_label" => {
                        let ram_label = text::create_plane_with_text(
                            (-0.5, -0.2, -0.02),
                            (0.02, 0.02, 1.0),
                            &self.font_maps["NotoSansJP"],
                            [1.0, 1.0, 1.0],
                            &format!("RAM: {:.2} MB", ALLOCATOR.allocated() as f32 / 1000000.0),
                        );
                        let meshes = vertex::create_vertices(&ram_label);
                        for (vertices, _) in meshes {
                            self.num_vertices[*index] = vec![vertices.len() as u32];
                            let vertex_buffer =
                                self.init.device.create_buffer(&wgpu::BufferDescriptor {
                                    label: Some("Vertex Buffer"),
                                    size: (size_of::<Vertex>() * vertices.len()) as u64,
                                    usage: wgpu::BufferUsages::VERTEX
                                        | wgpu::BufferUsages::COPY_DST,
                                    mapped_at_creation: false,
                                });
                            self.vertex_buffers[*index] = vec![vertex_buffer];
                            self.init.queue.write_buffer(
                                &self.vertex_buffers[*index][0],
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

    fn render_frame(&mut self) -> Result<(), openxr::sys::Result> {
        let frame_state = self.init.frame_waiter.wait()?;

        self.init.frame_stream.begin()?;

        if frame_state.should_render {
            println!("Rendering frame {}", self.frame);
        }

        self.init.frame_stream.end(
            frame_state.predicted_display_time,
            openxr::EnvironmentBlendMode::OPAQUE,
            &[],
        )?;

        Ok(())
    }

    pub fn run_frame_loop(&mut self) {
        loop {
            self.init.poll_events().ok();

            if self.init.get_session_running() {
                self.render_frame().ok();
            }
        }
    }
}
