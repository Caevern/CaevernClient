use crate::renderer::buffer_collection::BufferCollection;
use crate::{game_objects::object::Object, renderer::transforms::create_transforms};
use cgmath::Matrix;
use cgmath::SquareMatrix;

pub fn update_object(
    object: &mut Object,
    queue: &wgpu::Queue,
    buffer_collection: &mut BufferCollection,
) {
    let object_position = object.get_position();
    let object_rotation = object.get_rotation();
    let object_scale = object.get_scale();

    let model_mat = create_transforms(
        [object_position.x, object_position.y, object_position.z],
        [object_rotation.x, object_rotation.y, object_rotation.z],
        [object_scale.x, object_scale.y, object_scale.z],
    );
    let normal_mat = (model_mat.invert().unwrap()).transpose();

    let model_ref: &[f32; 16] = model_mat.as_ref();
    let normal_ref: &[f32; 16] = normal_mat.as_ref();

    queue.write_buffer(
        &buffer_collection.model_uniform_buffers[object.buffer_bindings.model_uniform_buffer],
        0,
        bytemuck::cast_slice(model_ref),
    );
    queue.write_buffer(
        &buffer_collection.model_uniform_buffers[object.buffer_bindings.model_uniform_buffer],
        64,
        bytemuck::cast_slice(normal_ref),
    );
}
