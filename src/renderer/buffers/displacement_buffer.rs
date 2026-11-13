use wgpu::BindGroup;

use crate::renderer::{
    buffer_collection::BufferCollection, textures::texture_object::TextureObject, vertex::Vertex,
};

// TODO: Fix every object having a displacement map
pub fn create_buffer_displacement(
    queue: &wgpu::Queue,
    device: &wgpu::Device,
    buffer_collection: &BufferCollection,
    model_uniform_buffer: &wgpu::Buffer,
    displacement_texture_object: &TextureObject,
    default_texture_object: &TextureObject,
    vertex_num: usize,
) -> (BindGroup, wgpu::Buffer) {
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &default_texture_object.texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &default_texture_object.texture_rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * default_texture_object.texture_width),
            rows_per_image: Some(default_texture_object.texture_height),
        },
        default_texture_object.texture_size,
    );

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &displacement_texture_object.texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &displacement_texture_object.texture_rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * displacement_texture_object.texture_width),
            rows_per_image: Some(displacement_texture_object.texture_height),
        },
        displacement_texture_object.texture_size,
    );

    let texture_view = default_texture_object
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let displacement_texture_view = displacement_texture_object
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    });

    let bone_buffer_index = buffer_collection.bone_buffers.len() - 1;
    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &buffer_collection.uniform_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer_collection.vertex_uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: buffer_collection
                    .fragment_uniform_buffer
                    .as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(&displacement_texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: model_uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 7,
                resource: buffer_collection.bone_buffers[bone_buffer_index].as_entire_binding(),
            },
        ],
        label: Some("Uniform Bind Group"),
    });

    let max_buffer_size = size_of::<Vertex>() * vertex_num;
    let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Vertex Buffer"),
        size: max_buffer_size as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    (uniform_bind_group, vertex_buffer)
}
