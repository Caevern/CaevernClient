use cgmath::*;

use crate::world::objects::player::Player;

pub fn get_camera_rotation(player: &Player, mouse: [f32; 2], frame_time: f32) -> (f32, f32) {
    let mut rotation_x = player.get_camera_transform().rotation.x + mouse[1] * (frame_time * 0.02);
    rotation_x = rotation_x.clamp(-std::f32::consts::FRAC_PI_2 / 1.01, std::f32::consts::FRAC_PI_2 / 1.01);
    let rotation_y = player.get_camera_transform().rotation.y - mouse[0] * (frame_time * 0.02);
    (rotation_x, rotation_y)
}

pub fn get_camera_movement(
    player: &mut Player, keys: [bool; 6],
    forward: Vector3<f32>, frame_time: f32
) -> Vector3<f32> {
    let mut forward_x = forward[0];
    let mut forward_z = forward[2];

    let len = (forward_x * forward_x + forward_z * forward_z).sqrt();
    if len > 0.0 {
        forward_x /= len;
        forward_z /= len;
    }

    let right = Vector3::new(
        player.camera.rotation.y.sin(),
        0.0,
        -player.camera.rotation.y.cos(),
    ).normalize();

    let mut walking_force = Vector3::new(0.0, 0.0, 0.0);

    if keys[0] {
        walking_force.x += frame_time * forward_x * player.walking_speed;
        walking_force.z += frame_time * forward_z * player.walking_speed;
    }
    if keys[2] {
        walking_force.x -= frame_time * forward_x * player.walking_speed;
        walking_force.z -= frame_time * forward_z * player.walking_speed;
    }
    if keys[1] {
        walking_force.x += frame_time * right[0] * player.walking_speed;
        walking_force.z += frame_time * right[2] * player.walking_speed;
    }
    if keys[3] {
        walking_force.x -= frame_time * right[0] * player.walking_speed;
        walking_force.z -= frame_time * right[2] * player.walking_speed;
    }
    if keys[4] && player.is_grounded {
        player.add_force(0.0, player.jump_force, 0.0);
        player.is_grounded = false;
    }
    walking_force
}