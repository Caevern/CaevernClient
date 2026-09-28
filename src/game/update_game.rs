use std::{
    collections::HashMap,
    f32,
    sync::mpsc::{Receiver, Sender},
};

use cgmath::{InnerSpace, Matrix, SquareMatrix, Vector3};

use crate::{
    game::{update_bone::update_bone, update_bones::update_bones},
    network::{
        avatar_updates::AvatarUpdate,
        user_updates::UserUpdate::{self, UpdateAvatarId},
    },
    physics::{
        gravity::apply_gravity,
        movement::{get_camera_movement, get_camera_rotation},
    },
    renderer::{
        buffer_collection::BufferCollection,
        create_rendered_object::create_rendered_object,
        transform::Transform,
        transforms::create_transforms,
        vertex::{Vertex, create_vertices_skinned},
    },
    world::{
        material::Material,
        object::{Object, ObjectType},
        objects::{player::Player, skeleton::create_skeleton},
        parsers::fbx_parser::parse,
        world::World,
    },
};

pub struct Engine {
    // player
    pub player: Player,

    // world
    pub world: World,

    // fallback model
    fallback_vertices: Vec<(Vec<Vertex>, String)>,
    fallback_bones: HashMap<i64, (usize, Transform, String, i64, usize)>,
    fallback_skeleton: HashMap<String, usize>,

    // networking
    data_thread_tx: Sender<UserUpdate>,
    avatar_thread_rx: Receiver<AvatarUpdate>,
}
impl Engine {
    pub fn new(
        data_thread_tx: Sender<UserUpdate>,
        avatar_thread_rx: Receiver<AvatarUpdate>,
    ) -> Self {
        let model_parsed = parse("models/fallback.fbx", Transform::zero());
        let fallback_vertices = create_vertices_skinned(&model_parsed.0);
        let fallback_bones = model_parsed.1;

        let bone_bindings = vec![("head".to_string(), "head.xModel")];
        let fallback_skeleton = create_skeleton(bone_bindings, &fallback_bones);

        Self {
            player: Player::new(),
            fallback_vertices,
            fallback_bones,
            fallback_skeleton,
            data_thread_tx,
            avatar_thread_rx,
            world: World::new(),
        }
    }

    pub fn update(
        &mut self,
        mouse: [f32; 2],
        keys: [bool; 6],
        frame_time: f32,
        buffer_collection: &mut BufferCollection,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
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

        let player_position = [
            self.player.camera.position.x - self.player.camera.rotation.y.cos() * 0.1,
            self.player.camera.position.y - self.player.height,
            self.player.camera.position.z - self.player.camera.rotation.y.sin() * 0.1,
        ];

        apply_gravity(&mut self.player, frame_time);

        let _ = self
            .data_thread_tx
            .send(UserUpdate::SendUserPosition(Transform {
                position: player_position.into(),
                rotation: Vector3::new(
                    -self.player.camera.rotation.x,
                    -self.player.camera.rotation.y + 1.57079633,
                    -self.player.camera.rotation.z,
                ),
                scale: Vector3::new(1.0, 1.0, 1.0),
            }));

        self.check_avatar_thread(buffer_collection, device, queue);
    }

    pub fn set_world(
        &mut self,
        world: World,
        buffer_collection: &mut BufferCollection,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        self.world = world;

        let textures = self.world.get_textures().clone();
        for object_index in 0..self.world.objects.len() {
            let mut object = self.world.objects.get_mut(&object_index).unwrap();
            create_rendered_object(&textures, &mut object, buffer_collection, device, queue);
        }
    }

    fn check_avatar_thread(
        &mut self,
        buffer_collection: &mut BufferCollection,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
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
                        transform.position.y - 0.2,
                        transform.position.z,
                    );
                    object.set_rotation(0.0, transform.rotation.y + f32::consts::PI, 0.0);
                    object.set_scale(0.0145, 0.0145, 0.0145);

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
                    self.world
                        .textures
                        .insert("textures/CG_Body_Base_color.png".to_string());
                    self.world
                        .textures
                        .insert("textures/CG_Hairs_Base_color.png".to_string());
                    self.world
                        .textures
                        .insert("textures/CG_Dress_Base_color.png".to_string());

                    let object_id = self.world.get_objects().len();

                    create_rendered_object(
                        &self.world.get_textures(),
                        &mut object,
                        buffer_collection,
                        device,
                        queue,
                    );

                    buffer_collection.bones[object.buffer_bindings.bones]
                        [self.fallback_skeleton["head"]]
                        .0
                        .rotation = [transform.rotation.x, 0.0, transform.rotation.z].into();

                    let buffer_bindings = object.buffer_bindings;
                    self.world.add_object(object);
                    update_bones(
                        &self.world,
                        object_id,
                        buffer_bindings,
                        buffer_collection,
                        queue,
                    );

                    self.data_thread_tx
                        .send(UpdateAvatarId(id, object_id))
                        .expect(
                            "Updating the avatar lookup table with the network stack has failed.",
                        );
                }
                AvatarUpdate::SetUserPosition(transform, object_id) => {
                    let object = self.world.get_object(object_id);

                    buffer_collection.bones[object.buffer_bindings.bones]
                        [self.fallback_skeleton["head"]]
                        .0
                        .rotation = [transform.rotation.x, 0.0, transform.rotation.z].into();
                    update_bone(
                        &self.world,
                        object_id,
                        object.buffer_bindings,
                        self.fallback_skeleton["head"],
                        buffer_collection,
                        queue,
                    );

                    let position = [
                        transform.position.x,
                        transform.position.y - 0.2,
                        transform.position.z,
                    ];
                    let rotation = [0.0, transform.rotation.y + f32::consts::PI, 0.0];

                    let model_mat =
                        create_transforms(position, rotation, object.get_scale().into());
                    let normal_mat = (model_mat.invert().unwrap()).transpose();

                    let model_ref: &[f32; 16] = model_mat.as_ref();
                    let normal_ref: &[f32; 16] = normal_mat.as_ref();

                    queue.write_buffer(
                        &buffer_collection.model_uniform_buffers
                            [object.buffer_bindings.model_uniform_buffer],
                        0,
                        bytemuck::cast_slice(model_ref),
                    );
                    queue.write_buffer(
                        &buffer_collection.model_uniform_buffers
                            [object.buffer_bindings.model_uniform_buffer],
                        64,
                        bytemuck::cast_slice(normal_ref),
                    );
                }
            }
        }
    }
}
