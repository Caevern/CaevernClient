use std::{
    collections::HashMap,
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
    world::objects::{fbx_parser::parse, player::Player, skeleton::create_skeleton},
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
    }
}
