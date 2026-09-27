use cgmath::Vector3;
use cgmath::SquareMatrix;
use cgmath::Matrix;

use crate::renderer::buffer_collection::BufferCollection;
use crate::renderer::transforms::create_transforms;
use crate::world::world::World;

pub fn update_bone(world: &World, object_index: usize, affected_bone: usize, buffer_collection: &mut BufferCollection, queue: &wgpu::Queue) {
    for (bone_index, bone) in buffer_collection.bones[object_index].iter().enumerate() {
        let mut bone_position =
            Vector3::new(bone.1.position.x, bone.1.position.y, bone.1.position.z);

        let mut is_descendant = false;

        if bone_index == affected_bone {
            is_descendant = true;
        }

        if bone.2 != -1 {
            let mut current_parent = bone.2;
            for _ in 0..buffer_collection.bones[object_index].len() {
                if current_parent == -1 {
                    break;
                }
                if current_parent as usize == affected_bone {
                    is_descendant = true;
                }

                let current_parent_bone = buffer_collection.bones[object_index][current_parent as usize];

                if current_parent_bone.2 != -1 {
                    bone_position.x += current_parent_bone.1.position.x;
                    bone_position.y += current_parent_bone.1.position.y;
                    bone_position.z += current_parent_bone.1.position.z;
                }

                current_parent = current_parent_bone.2;
            }
        } else {
            bone_position = Vector3::new(0.0, 0.0, 0.0);
        }

        if !is_descendant {
            continue;
        }

        let mut global_matrix = create_transforms(
            [
                bone.0.position.x,
                bone.0.position.y
                    + bone_position.y
                        * world.get_object(object_index).get_scale().y
                        * 175.0,
                bone.0.rotation.z,
            ],
            bone.0.rotation.into(),
            bone.0.scale.into(),
        );

        if bone.2 != -1 {
            let mut current_parent = bone.2;
            for _ in 0..buffer_collection.bones[object_index].len() {
                if current_parent == -1 {
                    break;
                }

                let current_parent_bone = buffer_collection.bones[object_index][current_parent as usize];

                let parent_matrix = create_transforms(
                    current_parent_bone.0.position.into(),
                    current_parent_bone.0.rotation.into(),
                    current_parent_bone.0.scale.into(),
                );
                global_matrix = parent_matrix * global_matrix;

                current_parent = current_parent_bone.2;
            }
        }

        if bone.3 != 0 {
            let model_mat = create_transforms(
                [
                    -2.5 + bone_position.x * 0.05,
                    3.0 + bone_position.y * 0.05,
                    bone_position.z * 0.05,
                ],
                [0.0, 0.0, 0.0],
                [1.0, 1.0, 1.0],
            );
            let normal_mat = (model_mat.invert().unwrap()).transpose();

            let model_ref: &[f32; 16] = model_mat.as_ref();
            let normal_ref: &[f32; 16] = normal_mat.as_ref();

            queue.write_buffer(
                &buffer_collection.model_uniform_buffers[bone.3],
                0,
                bytemuck::cast_slice(model_ref),
            );
            queue.write_buffer(
                &buffer_collection.model_uniform_buffers[bone.3],
                64,
                bytemuck::cast_slice(normal_ref),
            );
        }

        let bind_global = create_transforms(
            [
                0.0,
                bone_position.y * world.get_object(object_index).get_scale().y * 175.0,
                0.0,
            ],
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
        );
        let inverse_bind = bind_global.invert().expect("BIND DOESN'T EXIST");

        let final_matrix = global_matrix * inverse_bind;
        buffer_collection.final_matrices[object_index][bone_index] = final_matrix.into();
    }
    queue.write_buffer(
        &buffer_collection.bone_buffers[object_index],
        0,
        bytemuck::cast_slice(&buffer_collection.final_matrices[object_index]),
    );
}
