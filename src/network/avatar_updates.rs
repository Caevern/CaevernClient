use crate::renderer::transform::Transform;

pub enum AvatarUpdate {
    RegisterUser(Transform, u32),
    SetUserPosition(Transform, usize),
}
