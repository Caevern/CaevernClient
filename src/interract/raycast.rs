use std::collections::HashMap;

use cgmath::Vector3;

use crate::world::object::{Object, ObjectType};

pub fn distance(position_1: Vector3<f32>, position_2: Vector3<f32>) -> f32 {
    let dx = position_1.x - position_2.x;
    let dy = position_1.y - position_2.y;
    let dz = position_1.z - position_2.z;

    (dx * dx + dy * dy + dz * dz).sqrt()
}

// the function to get the nearest grabbable object
pub fn raycast_grab(
    objects: &HashMap<usize, Object>,
    position: Vector3<f32>,
    direction: cgmath::Vector3<f32>,
    max_distance: usize,
) -> usize {
    let mut position_checking: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
    position_checking.x = position.x + direction.x / 10.0;
    position_checking.y = position.y + direction.y / 10.0;
    position_checking.z = position.z + direction.z / 10.0;
    for _ in 0..max_distance * 10 {
        for (index, object) in objects {
            if object.get_object_type() != ObjectType::Grabbable {
                continue;
            }
            let distance_found = distance(position_checking, object.get_position());
            if distance_found < 0.5 {
                return *index;
            }
        }
        position_checking.x += direction.x / 10.0;
        position_checking.y += direction.y / 10.0;
        position_checking.z += direction.z / 10.0;
    }
    0
}
