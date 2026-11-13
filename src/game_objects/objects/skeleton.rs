use std::collections::HashMap;

use crate::renderer::transform::Transform;

pub fn create_skeleton(
    bone_bindings: Vec<(String, &str)>,
    fallback_bones: &HashMap<i64, (usize, Transform, String, i64, usize)>,
) -> HashMap<String, usize> {
    let mut fallback_skeleton = HashMap::new();
    for bone_binding in bone_bindings {
        for bone in fallback_bones {
            let filtered: String = bone.1.2.chars().filter(|c| (*c as u32) >= 32).collect();
            if filtered == bone_binding.1 {
                fallback_skeleton.insert(bone_binding.0, bone.1.0);
                break;
            }
        }
    }
    fallback_skeleton
}
