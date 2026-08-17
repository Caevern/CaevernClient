use crate::renderer::transform::Transform;

pub enum UserUpdate {
    SendUserPosition(Transform),
    SendReadySignal,
    UpdateAvatarId(u32, usize), // in the network thread, loop back the avatar id in world
}
