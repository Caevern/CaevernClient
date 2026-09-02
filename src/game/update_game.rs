use std::{
    collections::HashMap,
    f32,
    sync::mpsc::{Receiver, Sender},
};

use cgmath::{InnerSpace, Vector3};

use crate::{
    network::{avatar_updates::AvatarUpdate, user_updates::UserUpdate},
    physics::{
        gravity::apply_gravity,
        movement::{get_camera_movement, get_camera_rotation},
    },
    renderer::{
        transform::Transform,
        vertex::{Vertex, create_vertices_skinned},
    },
    world::{
        material::Material,
        object::{Object, ObjectType},
        objects::{player::Player, skeleton::create_skeleton},
        parsers::fbx_parser::parse,
    },
};

pub struct Engine {
    // player
    pub player: Player,

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
        }
    }

    pub fn update(&mut self, mouse: [f32; 2], keys: [bool; 6], frame_time: f32) {
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
            self.player.camera.position.y,
            self.player.camera.position.z - self.player.camera.rotation.y.sin() * 0.1,
        ];

        apply_gravity(&mut self.player, frame_time);

        let _ = self
            .data_thread_tx
            .send(UserUpdate::SendUserPosition(Transform {
                position: player_position.into(),
                rotation: Vector3::new(
                    -self.player.camera.rotation.x,
                    self.player.camera.rotation.y + 1.57079633,
                    -self.player.camera.rotation.z,
                ),
                scale: Vector3::new(1.0, 1.0, 1.0),
            }));

        self.check_avatar_thread();
    }

    fn check_avatar_thread(&mut self) {
        if let Ok(avatar_update) = self.avatar_thread_rx.try_recv() {
            match avatar_update {
                AvatarUpdate::RegisterUser(transform, _id) => {
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

                    /*let object_id = self.world.get_objects().len();

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
                        );*/
                }
                AvatarUpdate::SetUserPosition(_transform, _object_id) => {
                    /*self.bones[object_id][self.fallback_skeleton["head"]]
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
                    );*/
                }
            }
        }
    }
}
