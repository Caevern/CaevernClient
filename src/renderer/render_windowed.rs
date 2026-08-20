use cgmath::*;
use rust_embed::RustEmbed;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use std::{f32, println};
use winit::window::Window;

use crate::ALLOCATOR;
use crate::interract::raycast::raycast_grab;
use crate::network::avatar_updates::AvatarUpdate;
use crate::network::user_updates::UserUpdate::{self, UpdateAvatarId};
use crate::physics::gravity::apply_gravity;
use crate::physics::movement::{get_camera_movement, get_camera_rotation};
use crate::renderer::buffers::bind_group_layout::create_bind_group_layout;
use crate::renderer::buffers::displacement_buffer::create_buffer_displacement;
use crate::renderer::buffers::uniform_buffers::{
    create_fragment_uniform_buffer, create_vertex_uniform_buffer,
};
use crate::renderer::default_elements::register_default_textures;
use crate::renderer::pipelines::displacement_default::create_pipeline;
use crate::renderer::texture_object::TextureObject;
use crate::renderer::transform::Transform;
use crate::renderer::transforms::create_transforms;
use crate::renderer::vertex::{Vertex, create_vertices_skinned};
use crate::renderer::{init_wgpu, transform, transforms, vertex};
use crate::setup::fonts::load_font_uvs;
use crate::world::material::Material;
use crate::world::object::{Object, ObjectType};
use crate::world::objects::fbx_parser::parse;
use crate::world::objects::player::Player;
use crate::world::objects::skeleton::create_skeleton;
use crate::world::objects::text;
use crate::world::world::World;

#[derive(PartialEq)]
pub enum ShaderType {
    Displacement,
    DisplacementBones,
}

#[derive(RustEmbed)]
#[folder = "assets/"]
pub struct Assets;

pub struct RendererWindowed<'window> {
    pub init: init_wgpu::InitWgpu<'window>,

    pipeline_displacement: wgpu::RenderPipeline,
    pipeline_displacement_bones: wgpu::RenderPipeline,

    frame: usize,
    previous_frame_time: std::time::Instant,

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

    // the client position and rotation
    player: Player,

    world: World,
    current_camera: usize,

    fallback_vertices: Vec<(Vec<Vertex>, String)>,
    fallback_bones: HashMap<i64, (usize, Transform, String, i64, usize)>,
    fallback_skeleton: HashMap<String, usize>,

    // networking
    data_thread_tx: Sender<UserUpdate>,
    avatar_thread_rx: Receiver<AvatarUpdate>,
}
impl<'window> RendererWindowed<'window> {
    pub async fn new(
        window: &Arc<Window>,
        data_thread_tx: Sender<UserUpdate>,
        avatar_thread_rx: Receiver<AvatarUpdate>,
    ) -> Self {
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

        let model_parsed = parse("models/fallback.fbx", Transform::zero());
        let fallback_vertices = create_vertices_skinned(&model_parsed.0);
        let fallback_bones = model_parsed.1;

        let bone_bindings = vec![("head".to_string(), "head.xModel")];
        let fallback_skeleton = create_skeleton(bone_bindings, &fallback_bones);

        let vertex_buffers = Vec::new();
        let uniform_bind_groups = Vec::new();
        let num_vertices = Vec::new();
        let bone_buffers = Vec::new();
        let shader_type = Vec::new();

        let previous_frame_time = std::time::Instant::now();

        Self {
            init,

            pipeline_displacement,
            pipeline_displacement_bones,

            frame: 0,
            previous_frame_time,

            vertex_buffers,
            uniform_bind_groups,
            num_vertices,
            bone_buffers,
            bones: Vec::new(),
            final_marices: Vec::new(),
            shader_type,

            uniform_bind_group_layout,
            vertex_uniform_buffer,
            model_uniform_buffers: Vec::new(),
            fragment_uniform_buffer,

            textures,
            font_maps,

            player: Player::new(),

            world: World::new(),
            current_camera: 0,

            fallback_vertices,
            fallback_bones,
            fallback_skeleton,

            data_thread_tx,
            avatar_thread_rx,
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

    pub fn update(
        &mut self,
        _dt: std::time::Duration,
        keys: [bool; 6],
        mouse: [f32; 2],
        menu_tablet_state: usize,
    ) {
        let current_time = std::time::Instant::now();
        let mut frame_time = current_time
            .duration_since(self.previous_frame_time)
            .as_secs_f32()
            * 20.0;
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
        )
        .normalize();

        let updated_camera_position =
            get_camera_movement(&mut self.player, keys, forward, frame_time);
        self.player.camera.position += updated_camera_position;

        apply_gravity(&mut self.player, frame_time);

        if menu_tablet_state == 2 {
            for i in 0..self.world.get_objects().len() {
                let object_type = self.world.get_objects()[i].get_object_type();
                if object_type == ObjectType::TabletMenu
                    || object_type == ObjectType::TabletMenuButton
                {
                    let model_mat = transforms::create_transforms(
                        [
                            self.player.camera.position.x + forward.x,
                            self.player.camera.position.y + forward.y,
                            self.player.camera.position.z + forward.z,
                        ],
                        [
                            -self.player.camera.rotation.x,
                            -self.player.camera.rotation.y + std::f32::consts::FRAC_PI_2,
                            -self.player.camera.rotation.z,
                        ],
                        [1.0, 1.0, 1.0],
                    );
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
        } else if menu_tablet_state == 3 {
            for i in 0..self.world.get_objects().len() {
                let object_type = self.world.get_objects()[i].get_object_type();
                if object_type == ObjectType::TabletMenu
                    || object_type == ObjectType::TabletMenuButton
                {
                    let model_mat = transforms::create_transforms(
                        [0.0, -10.0, 0.0],
                        [
                            -self.player.camera.rotation.x,
                            -self.player.camera.rotation.y + std::f32::consts::FRAC_PI_2,
                            -self.player.camera.rotation.z,
                        ],
                        [1.0, 1.0, 1.0],
                    );
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
        }

        for i in 0..self.world.get_objects().len() {
            if self.world.get_object(i).get_object_type() == ObjectType::Skybox {
                let model_mat = transforms::create_transforms(
                    [
                        self.player.camera.position.x,
                        self.player.camera.position.y,
                        self.player.camera.position.z,
                    ],
                    [0.0, 0.0, 0.0],
                    [1.0, 1.0, 1.0],
                );
                let normal_mat = (model_mat.invert().unwrap()).transpose();

                let model_ref: &[f32; 16] = model_mat.as_ref();
                let normal_ref: &[f32; 16] = normal_mat.as_ref();
                let eye_position: &[f32; 3] = &self.player.camera.position.into();
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
            } else if self.world.get_object(i).get_object_type() == ObjectType::SkinnedMesh {
                if self.frame < 60 {
                    let skeleton = self.world.get_object(i).get_skeleton();
                    //self.bones[i][skeleton["head"]].0.rotation.x = -self.player.camera.rotation.x;
                    //self.bones[i][skeleton["arm_right"]].0.rotation.z = -self.player.camera.rotation.x;
                    /*self.bones[i][skeleton["head"]].0.rotation.y =
                    -self.player.camera.rotation.y - 1.57079633;*/
                    // TODO: make the local character have this dissabled by default.
                    self.bones[i][skeleton["neck"]].0.scale = [0.0, 0.0, 0.0].into();
                    self.update_bone(i, skeleton["head"]);
                }

                let object = self.world.get_object(i);
                let position = [
                    object.get_position().x + self.player.camera.position.x
                        - self.player.camera.rotation.y.cos() * 0.1,
                    object.get_position().y + self.player.camera.position.y,
                    object.get_position().z + self.player.camera.position.z
                        - self.player.camera.rotation.y.sin() * 0.1,
                ];
                let rotation = [
                    object.get_rotation().x,
                    object.get_rotation().y - self.player.camera.rotation.y + 1.57079633,
                    object.get_rotation().z,
                ];

                let _ = self
                    .data_thread_tx
                    .send(UserUpdate::SendUserPosition(Transform {
                        position: position.into(),
                        rotation: Vector3::new(
                            -self.player.camera.rotation.x,
                            rotation[1],
                            -self.player.camera.rotation.z,
                        ),
                        scale: Vector3::new(1.0, 1.0, 1.0),
                    }));

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

        // TODO: Refactor into an avatar struct
        if let Ok(avatar_update) = self.avatar_thread_rx.try_recv() {
            match avatar_update {
                AvatarUpdate::RegisterUser(transform, id) => {
                    println!("Registered User Avatar");
                    let mut object =
                        Object::create(ObjectType::Mesh, self.fallback_vertices.clone());

                    object.set_bones(
                        self.fallback_bones.clone(),
                        Vector3::new(0.0, 0.0, 0.0),
                        Vector3::new(0.0, 0.0, 0.0),
                        Vector3::new(1.0, 1.0, 1.0),
                    );
                    object.set_skeleton(self.fallback_skeleton.clone());

                    object.set_position(
                        transform.position.x,
                        transform.position.y,
                        transform.position.z,
                    );
                    object.set_rotation(0.0, transform.rotation.y + f32::consts::PI, 0.0);
                    object.set_scale(0.017, 0.017, 0.017);

                    object.add_material(
                        Material::from_texture("textures/CG_Body_Base_color.png"),
                        "BodyMaterial".to_string(),
                    );
                    object.add_material(
                        Material::from_texture("textures/CG_Hairs_Base_color.png"),
                        "HairsMaterial".to_string(),
                    );
                    object.add_material(
                        Material::from_texture("textures/CG_Dress_Base_color.png"),
                        "DressMaterial".to_string(),
                    );

                    let object_id = self.world.get_objects().len();

                    self.create_rendered_object(&object);
                    self.world.add_object(object);

                    self.bones[object_id][self.fallback_skeleton["head"]]
                        .0
                        .rotation = [transform.rotation.x, 0.0, transform.rotation.z].into();
                    self.update_bones(object_id);

                    self.data_thread_tx
                        .send(UpdateAvatarId(id, object_id))
                        .expect(
                            "Updating the avatar lookup table with the network stack has failed.",
                        );
                }
                AvatarUpdate::SetUserPosition(transform, object_id) => {
                    self.bones[object_id][self.fallback_skeleton["head"]]
                        .0
                        .rotation = [transform.rotation.x, 0.0, transform.rotation.z].into();
                    self.update_bone(object_id, self.fallback_skeleton["head"]);

                    let object = self.world.get_object(object_id);
                    let position = [
                        transform.position.x,
                        transform.position.y,
                        transform.position.z,
                    ];
                    let rotation = [0.0, transform.rotation.y + f32::consts::PI, 0.0];

                    let model_mat = transforms::create_transforms(
                        position,
                        rotation,
                        object.get_scale().into(),
                    );
                    let normal_mat = (model_mat.invert().unwrap()).transpose();

                    let model_ref: &[f32; 16] = model_mat.as_ref();
                    let normal_ref: &[f32; 16] = normal_mat.as_ref();

                    self.init.queue.write_buffer(
                        &self.model_uniform_buffers[object_id],
                        0,
                        bytemuck::cast_slice(model_ref),
                    );
                    self.init.queue.write_buffer(
                        &self.model_uniform_buffers[object_id],
                        64,
                        bytemuck::cast_slice(normal_ref),
                    );
                }
            }
        }

        if self.frame % 20 == 1 {
            for i in 0..self.world.get_objects().len() {
                if self.world.get_object(i).get_object_type() == ObjectType::SkinnedMesh {
                    self.update_bones(i);
                }
            }
        }

        // update skybox positions
        if self.frame % 10 == 0 {
            let grabbable_object_index = raycast_grab(
                self.world.get_objects(),
                self.player.camera.position,
                forward,
                5,
            );

            if grabbable_object_index > 0 {
                let y_rotation = self.world.get_objects()[grabbable_object_index]
                    .get_rotation()
                    .y;
                self.world.objects[grabbable_object_index].set_rotation_y(y_rotation + 0.1);
                let model_mat = transforms::create_transforms(
                    [0.0, 0.0, 0.0],
                    [0.0, y_rotation + 0.1, 0.0],
                    [1.0, 1.0, 1.0],
                );
                let normal_mat = (model_mat.invert().unwrap()).transpose();

                let model_ref: &[f32; 16] = model_mat.as_ref();
                let normal_ref: &[f32; 16] = normal_mat.as_ref();
                let eye_position: &[f32; 3] = &self.player.camera.position.into();
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
            x: self.player.camera.position.x,
            y: self.player.camera.position.y,
            z: self.player.camera.position.z,
        };
        let (view_mat, project_mat, _) = transforms::create_view_rotation(
            camera_position,
            self.player.camera.rotation.y,
            self.player.camera.rotation.x,
            up_direction,
            self.init.config.width as f32 / self.init.config.height as f32,
        );

        let view_project_mat = project_mat * view_mat;
        let view_projection_ref: &[f32; 16] = view_project_mat.as_ref();

        self.init.queue.write_buffer(
            &self.vertex_uniform_buffer,
            64,
            bytemuck::cast_slice(view_projection_ref),
        );

        let current_time_updated = std::time::Instant::now();
        let update_time = current_time_updated
            .duration_since(current_time)
            .as_secs_f32();

        // update ingame fps label when menu tablet is enabled
        if menu_tablet_state == 1 && self.frame % 60 == 0 {
            for (index, object) in self.world.get_objects().iter().enumerate() {
                match object.get_tag() {
                    "fps_label" => {
                        let fps_label = text::create_plane_with_text(
                            (-0.5, -0.3, -0.02),
                            (0.02, 0.02, 1.0),
                            &self.font_maps["NotoSansJP"],
                            [1.0, 1.0, 1.0],
                            &format!("FPS: {}", (1.0 / update_time).round()),
                        );
                        let meshes = vertex::create_vertices(&fps_label);
                        for (vertices, _) in meshes {
                            self.num_vertices[index] = vec![vertices.len() as u32];
                            let vertex_buffer =
                                self.init.device.create_buffer(&wgpu::BufferDescriptor {
                                    label: Some("Vertex Buffer"),
                                    size: (size_of::<Vertex>() * vertices.len()) as u64,
                                    usage: wgpu::BufferUsages::VERTEX
                                        | wgpu::BufferUsages::COPY_DST,
                                    mapped_at_creation: false,
                                });
                            self.vertex_buffers[index] = vec![vertex_buffer];
                            self.init.queue.write_buffer(
                                &self.vertex_buffers[index][0],
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
                            self.num_vertices[index] = vec![vertices.len() as u32];
                            let vertex_buffer =
                                self.init.device.create_buffer(&wgpu::BufferDescriptor {
                                    label: Some("Vertex Buffer"),
                                    size: (size_of::<Vertex>() * vertices.len()) as u64,
                                    usage: wgpu::BufferUsages::VERTEX
                                        | wgpu::BufferUsages::COPY_DST,
                                    mapped_at_creation: false,
                                });
                            self.vertex_buffers[index] = vec![vertex_buffer];
                            self.init.queue.write_buffer(
                                &self.vertex_buffers[index][0],
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

    // TODO: Update the bones in hiarchy, this keeps updating O(n)
    pub fn update_bone(&mut self, object_index: usize, affected_bone: usize) {
        for (bone_index, bone) in self.bones[object_index].iter().enumerate() {
            let mut bone_position =
                Vector3::new(bone.1.position.x, bone.1.position.y, bone.1.position.z);

            let mut is_descendant = false;

            if bone_index == affected_bone {
                is_descendant = true;
            }

            if bone.2 != -1 {
                let mut current_parent = bone.2;
                for _ in 0..self.bones[object_index].len() {
                    if current_parent == -1 {
                        break;
                    }
                    if current_parent as usize == affected_bone {
                        is_descendant = true;
                    }

                    let current_parent_bone = self.bones[object_index][current_parent as usize];

                    if current_parent_bone.2 != -1 {
                        bone_position.x += current_parent_bone.1.position.x;
                        bone_position.y += current_parent_bone.1.position.y;
                        bone_position.z += current_parent_bone.1.position.z;
                    }

                    current_parent = current_parent_bone.2;
                }
            } else {
                bone_position = Vector3::new(0.0, 0.0, 0.0);
            }

            if !is_descendant {
                continue;
            }

            let mut global_matrix = create_transforms(
                [
                    bone.0.position.x,
                    bone.0.position.y
                        + bone_position.y
                            * self.world.get_object(object_index).get_scale().y
                            * 150.0,
                    bone.0.rotation.z,
                ],
                bone.0.rotation.into(),
                bone.0.scale.into(),
            );

            if bone.2 != -1 {
                let mut current_parent = bone.2;
                for _ in 0..self.bones[object_index].len() {
                    if current_parent == -1 {
                        break;
                    }

                    let current_parent_bone = self.bones[object_index][current_parent as usize];

                    let parent_matrix = create_transforms(
                        current_parent_bone.0.position.into(),
                        current_parent_bone.0.rotation.into(),
                        current_parent_bone.0.scale.into(),
                    );
                    global_matrix = parent_matrix * global_matrix;

                    current_parent = current_parent_bone.2;
                }
            }

            if bone.3 != 0 {
                let model_mat = transforms::create_transforms(
                    [
                        -2.5 + bone_position.x * 0.05,
                        3.0 + bone_position.y * 0.05,
                        bone_position.z * 0.05,
                    ],
                    [0.0, 0.0, 0.0],
                    [1.0, 1.0, 1.0],
                );
                let normal_mat = (model_mat.invert().unwrap()).transpose();

                let model_ref: &[f32; 16] = model_mat.as_ref();
                let normal_ref: &[f32; 16] = normal_mat.as_ref();

                self.init.queue.write_buffer(
                    &self.model_uniform_buffers[bone.3],
                    0,
                    bytemuck::cast_slice(model_ref),
                );
                self.init.queue.write_buffer(
                    &self.model_uniform_buffers[bone.3],
                    64,
                    bytemuck::cast_slice(normal_ref),
                );
            }

            let bind_global = create_transforms(
                [
                    0.0,
                    bone_position.y * self.world.get_object(object_index).get_scale().y * 150.0,
                    0.0,
                ],
                [0.0, 0.0, 0.0],
                [1.0, 1.0, 1.0],
            );
            let inverse_bind = bind_global.invert().expect("BIND DOESN'T EXIST");

            let final_matrix = global_matrix * inverse_bind;
            self.final_marices[object_index][bone_index] = final_matrix.into();
        }
        self.init.queue.write_buffer(
            &self.bone_buffers[object_index],
            0,
            bytemuck::cast_slice(&self.final_marices[object_index]),
        );
    }

    // TODO: Update the bones in hiarchy, this keeps updating O(n)
    pub fn update_bones(&mut self, object_index: usize) {
        for (bone_index, bone) in self.bones[object_index].iter().enumerate() {
            let mut bone_position =
                Vector3::new(bone.1.position.x, bone.1.position.y, bone.1.position.z);

            if bone.2 != -1 {
                let mut current_parent = bone.2;
                for _ in 0..self.bones[object_index].len() {
                    if current_parent == -1 {
                        break;
                    }

                    let current_parent_bone = self.bones[object_index][current_parent as usize];

                    if current_parent_bone.2 != -1 {
                        bone_position.x += current_parent_bone.1.position.x;
                        bone_position.y += current_parent_bone.1.position.y;
                        bone_position.z += current_parent_bone.1.position.z;
                    }

                    current_parent = current_parent_bone.2;
                }
            } else {
                bone_position = Vector3::new(0.0, 0.0, 0.0);
            }

            let mut global_matrix = create_transforms(
                [
                    bone.0.position.x,
                    bone.0.position.y
                        + bone_position.y
                            * self.world.get_object(object_index).get_scale().y
                            * 150.0,
                    bone.0.rotation.z,
                ],
                bone.0.rotation.into(),
                bone.0.scale.into(),
            );

            if bone.2 != -1 {
                let mut current_parent = bone.2;
                for _ in 0..self.bones[object_index].len() {
                    if current_parent == -1 {
                        break;
                    }

                    let current_parent_bone = self.bones[object_index][current_parent as usize];

                    let parent_matrix = create_transforms(
                        current_parent_bone.0.position.into(),
                        current_parent_bone.0.rotation.into(),
                        current_parent_bone.0.scale.into(),
                    );
                    global_matrix = parent_matrix * global_matrix;

                    current_parent = current_parent_bone.2;
                }
            }

            if bone.3 != 0 {
                let model_mat = transforms::create_transforms(
                    [
                        -2.5 + bone_position.x * 0.05,
                        3.0 + bone_position.y * 0.05,
                        bone_position.z * 0.05,
                    ],
                    [0.0, 0.0, 0.0],
                    [1.0, 1.0, 1.0],
                );
                let normal_mat = (model_mat.invert().unwrap()).transpose();

                let model_ref: &[f32; 16] = model_mat.as_ref();
                let normal_ref: &[f32; 16] = normal_mat.as_ref();

                self.init.queue.write_buffer(
                    &self.model_uniform_buffers[bone.3],
                    0,
                    bytemuck::cast_slice(model_ref),
                );
                self.init.queue.write_buffer(
                    &self.model_uniform_buffers[bone.3],
                    64,
                    bytemuck::cast_slice(normal_ref),
                );
            }

            let bind_global = create_transforms(
                [
                    0.0,
                    bone_position.y * self.world.get_object(object_index).get_scale().y * 150.0,
                    0.0,
                ],
                [0.0, 0.0, 0.0],
                [1.0, 1.0, 1.0],
            );
            let inverse_bind = bind_global.invert().expect("BIND DOESN'T EXIST");

            let final_matrix = global_matrix * inverse_bind;
            self.final_marices[object_index][bone_index] = final_matrix.into();
        }
        self.init.queue.write_buffer(
            &self.bone_buffers[object_index],
            0,
            bytemuck::cast_slice(&self.final_marices[object_index]),
        );
    }

    // TODO: Use this in the set world to reduce duplicate code
    pub fn create_rendered_object(&mut self, object: &Object) {
        for texture in self.world.get_textures() {
            if self.textures.contains_key(&texture.to_string()) {
                continue;
            }

            self.textures.insert(
                texture.to_string(),
                TextureObject::create(texture, &self.init.device),
            );
        }

        let meshes = object.get_vertices();
        let materials = object.get_materials();
        let mut bones: Vec<[[f32; 4]; 4]> = Vec::new();
        self.vertex_buffers.push(Vec::new());
        self.uniform_bind_groups.push(Vec::new());
        self.num_vertices.push(Vec::new());

        let object_id = self.vertex_buffers.len() - 1;

        let bone_transforms = object.get_bones();
        self.final_marices.push(Vec::new());
        for _ in 0..bone_transforms.len() {
            self.final_marices[object_id].push([
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
            ]);
        }
        self.bones.push(bone_transforms.clone());

        for bone in bone_transforms {
            bones.push(
                transforms::create_transforms(
                    bone.0.position.into(),
                    bone.0.rotation.into(),
                    bone.0.scale.into(),
                )
                .into(),
            );
        }

        let bone_buffer;
        if bones.len() > 0 {
            println!("{}", object_id);
            self.shader_type.push(ShaderType::DisplacementBones);
            bone_buffer = self.init.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Bone Buffer"),
                size: (bones.len() * std::mem::size_of::<Matrix4<f32>>()) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.init
                .queue
                .write_buffer(&bone_buffer, 0, bytemuck::cast_slice(&bones));
        } else {
            self.shader_type.push(ShaderType::Displacement);
            bone_buffer = self.init.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Bone Buffer"),
                size: 16,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        self.bone_buffers.push(bone_buffer);

        let model_uniform_buffer: wgpu::Buffer =
            self.init.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Vertex Uniform Buffer"),
                size: 128,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

        let model_mat = transforms::create_transforms(
            [
                object.get_position().x,
                object.get_position().y,
                object.get_position().z,
            ],
            [
                object.get_rotation().x,
                object.get_rotation().y,
                object.get_rotation().z,
            ],
            [
                object.get_scale().x,
                object.get_scale().y,
                object.get_scale().z,
            ],
        );
        let normal_mat = (model_mat.invert().unwrap()).transpose();

        let model_ref: &[f32; 16] = model_mat.as_ref();
        let normal_ref: &[f32; 16] = normal_mat.as_ref();
        self.init
            .queue
            .write_buffer(&model_uniform_buffer, 0, bytemuck::cast_slice(model_ref));
        self.init
            .queue
            .write_buffer(&model_uniform_buffer, 64, bytemuck::cast_slice(normal_ref));

        for (vertices, material_name) in meshes {
            let material_found;
            let bytes_filtered: Vec<u8> =
                material_name.bytes().filter(|c| c > &(31 as u8)).collect();
            let material_string = String::from_utf8(bytes_filtered).unwrap();

            if let Some(material) = materials.get(&material_string) {
                material_found = material;
            } else {
                material_found = &materials.get("default").unwrap();
            }

            let material_found_texture = material_found.get_texture();
            let material_found_displacement = material_found.get_displacement();

            println!(
                "loading: {} from: {}",
                material_found_texture, material_string
            );

            let texture_object;
            if let Some(texture) = self.textures.get(material_found_texture) {
                texture_object = texture;
            } else {
                continue;
            }

            let texture_object_displacement;
            if let Some(texture_displacement_name) = material_found_displacement {
                if let Some(texture_displacement) = self.textures.get(texture_displacement_name) {
                    texture_object_displacement = Some(texture_displacement);
                } else {
                    texture_object_displacement = None;
                }
            } else {
                texture_object_displacement = None;
            }

            let uniform_bind_group;
            let vertex_buffer;
            if let Some(texture_displacement) = texture_object_displacement {
                (uniform_bind_group, vertex_buffer) = create_buffer_displacement(
                    &self.init.queue,
                    &self.init.device,
                    &self.uniform_bind_group_layout,
                    &self.vertex_uniform_buffer,
                    &self.fragment_uniform_buffer,
                    &model_uniform_buffer,
                    &self.bone_buffers[object_id],
                    &texture_displacement.texture,
                    texture_displacement.texture_size,
                    &texture_displacement.texture_rgba,
                    texture_displacement.texture_width,
                    texture_displacement.texture_height,
                    &texture_object.texture,
                    texture_object.texture_size,
                    &texture_object.texture_rgba,
                    texture_object.texture_width,
                    texture_object.texture_height,
                    vertices.len(),
                );
            } else {
                if let Some(texture_displacement) = self.textures.get("textures/displacement.png") {
                    (uniform_bind_group, vertex_buffer) = create_buffer_displacement(
                        &self.init.queue,
                        &self.init.device,
                        &self.uniform_bind_group_layout,
                        &self.vertex_uniform_buffer,
                        &self.fragment_uniform_buffer,
                        &model_uniform_buffer,
                        &self.bone_buffers[object_id],
                        &texture_displacement.texture,
                        texture_displacement.texture_size,
                        &texture_displacement.texture_rgba,
                        texture_displacement.texture_width,
                        texture_displacement.texture_height,
                        &texture_object.texture,
                        texture_object.texture_size,
                        &texture_object.texture_rgba,
                        texture_object.texture_width,
                        texture_object.texture_height,
                        vertices.len(),
                    );
                } else {
                    continue;
                }
            }

            self.vertex_buffers[object_id].push(vertex_buffer);
            self.uniform_bind_groups[object_id].push(uniform_bind_group);

            self.num_vertices[object_id].push(vertices.len() as u32);
            self.init.queue.write_buffer(
                &self.vertex_buffers[object_id][self.vertex_buffers[object_id].len() - 1],
                0,
                bytemuck::cast_slice(vertices),
            );
        }

        self.model_uniform_buffers.push(model_uniform_buffer);
    }

    pub fn set_world(&mut self, world: World) {
        self.world = world;

        self.vertex_buffers.clear();
        self.uniform_bind_groups.clear();
        self.num_vertices.clear();

        for texture in self.world.get_textures() {
            self.textures.insert(
                texture.to_string(),
                TextureObject::create(texture, &self.init.device),
            );
        }

        for object in self.world.get_objects().iter().enumerate() {
            let meshes = object.1.get_vertices();
            let materials = object.1.get_materials();
            let mut bones: Vec<[[f32; 4]; 4]> = Vec::new();
            self.vertex_buffers.push(Vec::new());
            self.uniform_bind_groups.push(Vec::new());
            self.num_vertices.push(Vec::new());

            let bone_transforms = object.1.get_bones();
            self.final_marices.push(Vec::new());
            for _ in 0..bone_transforms.len() {
                self.final_marices[object.0].push([
                    [0.0, 0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0, 0.0],
                ]);
            }
            self.bones.push(bone_transforms.clone());

            for bone in bone_transforms {
                bones.push(
                    transforms::create_transforms(
                        bone.0.position.into(),
                        bone.0.rotation.into(),
                        bone.0.scale.into(),
                    )
                    .into(),
                );
            }

            let bone_buffer;
            if bones.len() > 0 {
                println!("{}", object.0);
                self.shader_type.push(ShaderType::DisplacementBones);
                bone_buffer = self.init.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Bone Buffer"),
                    size: (bones.len() * std::mem::size_of::<Matrix4<f32>>()) as u64,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                self.init
                    .queue
                    .write_buffer(&bone_buffer, 0, bytemuck::cast_slice(&bones));
            } else {
                self.shader_type.push(ShaderType::Displacement);
                bone_buffer = self.init.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Bone Buffer"),
                    size: 16,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
            }

            self.bone_buffers.push(bone_buffer);

            let model_uniform_buffer: wgpu::Buffer =
                self.init.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Vertex Uniform Buffer"),
                    size: 128,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });

            let model_mat = transforms::create_transforms(
                [
                    object.1.get_position().x,
                    object.1.get_position().y,
                    object.1.get_position().z,
                ],
                [
                    object.1.get_rotation().x,
                    object.1.get_rotation().y,
                    object.1.get_rotation().z,
                ],
                [
                    object.1.get_scale().x,
                    object.1.get_scale().y,
                    object.1.get_scale().z,
                ],
            );
            let normal_mat = (model_mat.invert().unwrap()).transpose();

            let model_ref: &[f32; 16] = model_mat.as_ref();
            let normal_ref: &[f32; 16] = normal_mat.as_ref();
            self.init
                .queue
                .write_buffer(&model_uniform_buffer, 0, bytemuck::cast_slice(model_ref));
            self.init.queue.write_buffer(
                &model_uniform_buffer,
                64,
                bytemuck::cast_slice(normal_ref),
            );

            for (vertices, material_name) in meshes {
                let material_found;
                let bytes_filtered: Vec<u8> =
                    material_name.bytes().filter(|c| c > &(31 as u8)).collect();
                let material_string = String::from_utf8(bytes_filtered).unwrap();

                if let Some(material) = materials.get(&material_string) {
                    material_found = material;
                } else {
                    material_found = &materials.get("default").unwrap();
                }

                let material_found_texture = material_found.get_texture();
                let material_found_displacement = material_found.get_displacement();

                println!(
                    "loading: {} from: {}",
                    material_found_texture, material_string
                );

                let texture_object;
                if let Some(texture) = self.textures.get(material_found_texture) {
                    texture_object = texture;
                } else {
                    continue;
                }

                let texture_object_displacement;
                if let Some(texture_displacement_name) = material_found_displacement {
                    if let Some(texture_displacement) = self.textures.get(texture_displacement_name)
                    {
                        texture_object_displacement = Some(texture_displacement);
                    } else {
                        texture_object_displacement = None;
                    }
                } else {
                    texture_object_displacement = None;
                }

                let uniform_bind_group;
                let vertex_buffer;
                if let Some(texture_displacement) = texture_object_displacement {
                    (uniform_bind_group, vertex_buffer) = create_buffer_displacement(
                        &self.init.queue,
                        &self.init.device,
                        &self.uniform_bind_group_layout,
                        &self.vertex_uniform_buffer,
                        &self.fragment_uniform_buffer,
                        &model_uniform_buffer,
                        &self.bone_buffers[object.0],
                        &texture_displacement.texture,
                        texture_displacement.texture_size,
                        &texture_displacement.texture_rgba,
                        texture_displacement.texture_width,
                        texture_displacement.texture_height,
                        &texture_object.texture,
                        texture_object.texture_size,
                        &texture_object.texture_rgba,
                        texture_object.texture_width,
                        texture_object.texture_height,
                        vertices.len(),
                    );
                } else {
                    if let Some(texture_displacement) =
                        self.textures.get("textures/displacement.png")
                    {
                        (uniform_bind_group, vertex_buffer) = create_buffer_displacement(
                            &self.init.queue,
                            &self.init.device,
                            &self.uniform_bind_group_layout,
                            &self.vertex_uniform_buffer,
                            &self.fragment_uniform_buffer,
                            &model_uniform_buffer,
                            &self.bone_buffers[object.0],
                            &texture_displacement.texture,
                            texture_displacement.texture_size,
                            &texture_displacement.texture_rgba,
                            texture_displacement.texture_width,
                            texture_displacement.texture_height,
                            &texture_object.texture,
                            texture_object.texture_size,
                            &texture_object.texture_rgba,
                            texture_object.texture_width,
                            texture_object.texture_height,
                            vertices.len(),
                        );
                    } else {
                        continue;
                    }
                }

                self.vertex_buffers[object.0].push(vertex_buffer);
                self.uniform_bind_groups[object.0].push(uniform_bind_group);

                self.num_vertices[object.0].push(vertices.len() as u32);
                self.init.queue.write_buffer(
                    &self.vertex_buffers[object.0][self.vertex_buffers[object.0].len() - 1],
                    0,
                    bytemuck::cast_slice(vertices),
                );
            }

            self.model_uniform_buffers.push(model_uniform_buffer);
        }

        for object in self.world.objects.iter_mut() {
            object.clear_vertices();
        }

        if self.world.get_cameras().len() > self.current_camera {
            let object_position = self.world.get_camera(self.current_camera).get_position();
            self.player.camera.position.x = object_position.x as f32;
            self.player.camera.position.y = object_position.y as f32;
            self.player.camera.position.z = object_position.z as f32;
            self.player.camera.rotation = self.world.get_camera(self.current_camera).get_rotation();
        }

        self.data_thread_tx
            .send(UserUpdate::SendReadySignal)
            .expect("Sending user ready signal failed :C");
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
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                //depth_stencil_attachment: None,
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
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

        self.init.queue.submit(Some(encoder.finish()));
        self.init.queue.present(output);

        Ok(())
    }
}
