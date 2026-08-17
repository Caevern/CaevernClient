use crate::renderer::transform::Transform;

pub enum UserUpdate {
    SendUserPosition(Transform),
}
