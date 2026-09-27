use cgmath::Matrix4;
use cgmath::SquareMatrix;
use cgmath::Matrix;

use crate::renderer::buffers::displacement_buffer::create_buffer_displacement;
use crate::renderer::shader_type::ShaderType;
use crate::renderer::texture_object::TextureObject;
use crate::renderer::transforms;
use crate::world::{object::Object, world::World};
use crate::renderer::buffer_collection::BufferCollection;

pub fn create_rendered_object(world: &World, object: &Object, buffer_collection: &mut BufferCollection, device: &wgpu::Device, queue: &wgpu::Queue) {
    for texture in world.get_textures() {
        if buffer_collection.textures.contains_key(&texture.to_string()) {
            continue;
        }

        buffer_collection.textures.insert(
            texture.to_string(),
            TextureObject::create(texture, device),
        );
    }

    let meshes = object.get_vertices();
    let materials = object.get_materials();
    let mut bones: Vec<[[f32; 4]; 4]> = Vec::new();
    buffer_collection.vertex_buffers.push(Vec::new());
    buffer_collection.uniform_bind_groups.push(Vec::new());
    buffer_collection.num_vertices.push(Vec::new());

    let object_id = buffer_collection.vertex_buffers.len() - 1;

    let bone_transforms = object.get_bones();
    buffer_collection.final_matrices.push(Vec::new());
    for _ in 0..bone_transforms.len() {
        buffer_collection.final_matrices[object_id].push([
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
        ]);
    }
    buffer_collection.bones.push(bone_transforms.clone());

    for bone in bone_transforms {
        bones.push(
            transforms::create_transforms(
                bone.0.position.into(),
                bone.0.rotation.into(),
                bone.0.scale.into(),
            )
            .into(),
        );
    }

    let bone_buffer;
    if bones.len() > 0 {
        println!("{}", object_id);
        buffer_collection.shader_type.push(ShaderType::DisplacementBones);
        bone_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Bone Buffer"),
            size: (bones.len() * std::mem::size_of::<Matrix4<f32>>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&bone_buffer, 0, bytemuck::cast_slice(&bones));
    } else {
        buffer_collection.shader_type.push(ShaderType::Displacement);
        bone_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Bone Buffer"),
            size: 16,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
    }

    buffer_collection.bone_buffers.push(bone_buffer);

    let model_uniform_buffer: wgpu::Buffer =
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Uniform Buffer"),
            size: 128,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

    let model_mat = transforms::create_transforms(
        [
            object.get_position().x,
            object.get_position().y,
            object.get_position().z,
        ],
        [
            object.get_rotation().x,
            object.get_rotation().y,
            object.get_rotation().z,
        ],
        [
            object.get_scale().x,
            object.get_scale().y,
            object.get_scale().z,
        ],
    );
    let normal_mat = (model_mat.invert().unwrap()).transpose();

    let model_ref: &[f32; 16] = model_mat.as_ref();
    let normal_ref: &[f32; 16] = normal_mat.as_ref();
    queue.write_buffer(&model_uniform_buffer, 0, bytemuck::cast_slice(model_ref));
    queue.write_buffer(&model_uniform_buffer, 64, bytemuck::cast_slice(normal_ref));

    for (vertices, material_name) in meshes {
        let material_found;
        let bytes_filtered: Vec<u8> =
            material_name.bytes().filter(|c| c > &(31 as u8)).collect();
        let material_string = String::from_utf8(bytes_filtered).unwrap();

        if let Some(material) = materials.get(&material_string) {
            material_found = material;
        } else {
            material_found = &materials.get("default").unwrap();
        }

        let material_found_texture = material_found.get_texture();
        let material_found_displacement = material_found.get_displacement();

        println!(
            "loading: {} from: {}",
            material_found_texture, material_string
        );

        let texture_object;
        if let Some(texture) = buffer_collection.textures.get(material_found_texture) {
            texture_object = texture;
        } else {
            continue;
        }

        let texture_object_displacement;
        if let Some(texture_displacement_name) = material_found_displacement {
            if let Some(texture_displacement) = buffer_collection.textures.get(texture_displacement_name) {
                texture_object_displacement = Some(texture_displacement);
            } else {
                texture_object_displacement = None;
            }
        } else {
            texture_object_displacement = None;
        }

        let uniform_bind_group;
        let vertex_buffer;
        if let Some(texture_displacement) = texture_object_displacement {
            (uniform_bind_group, vertex_buffer) = create_buffer_displacement(
                &queue,
                &device,
                &buffer_collection.uniform_bind_group_layout,
                &buffer_collection.vertex_uniform_buffer,
                &buffer_collection.fragment_uniform_buffer,
                &model_uniform_buffer,
                &buffer_collection.bone_buffers[object_id],
                &texture_displacement.texture,
                texture_displacement.texture_size,
                &texture_displacement.texture_rgba,
                texture_displacement.texture_width,
                texture_displacement.texture_height,
                &texture_object.texture,
                texture_object.texture_size,
                &texture_object.texture_rgba,
                texture_object.texture_width,
                texture_object.texture_height,
                vertices.len(),
            );
        } else {
            if let Some(texture_displacement) = buffer_collection.textures.get("textures/displacement.png") {
                (uniform_bind_group, vertex_buffer) = create_buffer_displacement(
                    &queue,
                    &device,
                    &buffer_collection.uniform_bind_group_layout,
                    &buffer_collection.vertex_uniform_buffer,
                    &buffer_collection.fragment_uniform_buffer,
                    &model_uniform_buffer,
                    &buffer_collection.bone_buffers[object_id],
                    &texture_displacement.texture,
                    texture_displacement.texture_size,
                    &texture_displacement.texture_rgba,
                    texture_displacement.texture_width,
                    texture_displacement.texture_height,
                    &texture_object.texture,
                    texture_object.texture_size,
                    &texture_object.texture_rgba,
                    texture_object.texture_width,
                    texture_object.texture_height,
                    vertices.len(),
                );
            } else {
                continue;
            }
        }

        buffer_collection.vertex_buffers[object_id].push(vertex_buffer);
        buffer_collection.uniform_bind_groups[object_id].push(uniform_bind_group);

        buffer_collection.num_vertices[object_id].push(vertices.len() as u32);
        queue.write_buffer(
            &buffer_collection.vertex_buffers[object_id][buffer_collection.vertex_buffers[object_id].len() - 1],
            0,
            bytemuck::cast_slice(vertices),
        );
    }

    buffer_collection.model_uniform_buffers.push(model_uniform_buffer);
}
