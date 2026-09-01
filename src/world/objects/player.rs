use cgmath::Vector3;

use crate::renderer::transform::Transform;

pub struct Player {
    pub transform: Transform,
    pub camera: Transform,
    pub forces: Vector3<f32>,

    pub is_grounded: bool,
    pub walking_speed: f32,
    pub jump_force: f32,
    pub gravity: f32,

    pub sensitivity: f32,
}
impl Player {
    pub fn new() -> Self {
        Self {
            transform: Transform::zero(),
            camera: Transform::zero(),
            forces: Vector3::new(0.0, 0.0, 0.0),

            is_grounded: true,
            walking_speed: 7.5,
            jump_force: 5.0,
            gravity: 30.0,

            sensitivity: 0.5,
        }
    }

    pub fn get_camera_transform(&self) -> &Transform {
        &self.camera
    }

    pub fn add_force(&mut self, x: f32, y: f32, z: f32) {
        self.forces.x += x;
        self.forces.y += y;
        self.forces.z += z;
    }
}
