use cgmath::*;

use crate::{interract::input_state::InputState, world::objects::player::Player};

pub fn get_camera_rotation(player: &Player, mouse: &Vector2<f32>, frame_time: f32) -> (f32, f32) {
    let mut rotation_x =
        player.get_camera_transform().rotation.x + mouse.y * (frame_time * player.sensitivity);
    rotation_x = rotation_x.clamp(
        -std::f32::consts::FRAC_PI_2 / 1.01,
        std::f32::consts::FRAC_PI_2 / 1.01,
    );
    let rotation_y =
        player.get_camera_transform().rotation.y - mouse.x * (frame_time * player.sensitivity);
    (rotation_x, rotation_y)
}

pub fn get_camera_movement(
    player: &mut Player,
    input: &InputState,
    forward: Vector3<f32>,
    frame_time: f32,
) -> Vector3<f32> {
    let mut forward_x = forward[0];
    let mut forward_z = forward[2];

    let len = (forward_x * forward_x + forward_z * forward_z).sqrt();
    if len > 0.0 {
        forward_x /= len;
        forward_z /= len;
    }

    let right = Vector3::new(
        (player.camera.rotation.y + player.camera_offset.rotation.y).sin(),
        0.0,
        -(player.camera.rotation.y + player.camera_offset.rotation.y).cos(),
    )
    .normalize();

    let mut walking_force = Vector3::new(0.0, 0.0, 0.0);

    if input.left {
        walking_force.x += frame_time * forward_x * player.walking_speed;
        walking_force.z += frame_time * forward_z * player.walking_speed;
    }
    if input.right {
        walking_force.x -= frame_time * forward_x * player.walking_speed;
        walking_force.z -= frame_time * forward_z * player.walking_speed;
    }
    if input.backwards {
        walking_force.x += frame_time * right[0] * player.walking_speed;
        walking_force.z += frame_time * right[2] * player.walking_speed;
    }
    if input.forewards {
        walking_force.x -= frame_time * right[0] * player.walking_speed;
        walking_force.z -= frame_time * right[2] * player.walking_speed;
    }
    if input.jump && player.is_grounded {
        player.add_force(0.0, player.jump_force, 0.0);
        player.is_grounded = false;
    }
    walking_force
}
