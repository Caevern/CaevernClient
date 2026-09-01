use crate::world::objects::player::Player;

pub fn check_grounded(player: &mut Player) {
    player.is_grounded = player.camera.position.y <= 4.0;
    if player.camera.position.y < 4.0 {
        player.camera.position.y = 4.0;
    }
}

pub fn apply_gravity(player: &mut Player, frame_time: f32) {
    check_grounded(player);

    if player.is_grounded {
        if player.forces.y < 0.0 {
            player.forces.y = 0.0;
        } else {
            player.camera.position.y += player.forces.y * frame_time;
        }
        return;
    }

    player.forces.y -= player.gravity * frame_time;
    player.camera.position.y += player.forces.y * frame_time;
}
