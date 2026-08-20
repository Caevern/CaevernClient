use wgpu::BindGroup;

use crate::renderer::vertex::Vertex;

pub fn create_buffer_displacement(
    queue: &wgpu::Queue,
    device: &wgpu::Device,
    uniform_bind_group_layout: &wgpu::BindGroupLayout,
    vertex_uniform_buffer: &wgpu::Buffer,
    fragment_uniform_buffer: &wgpu::Buffer,
    model_uniform_buffer: &wgpu::Buffer,
    bones_buffer: &wgpu::Buffer,
    displacement_texture: &wgpu::Texture,
    displacement_texture_size: wgpu::Extent3d,
    displacement_rgba: &Vec<u8>,
    displacement_width: u32,
    displacement_height: u32,
    texture: &wgpu::Texture,
    texture_size: wgpu::Extent3d,
    rgba: &Vec<u8>,
    width: u32,
    height: u32,
    vertex_num: usize,
) -> (BindGroup, wgpu::Buffer) {
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        texture_size,
    );

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &displacement_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &displacement_rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * displacement_width),
            rows_per_image: Some(displacement_height),
        },
        displacement_texture_size,
    );

    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let displacement_texture_view =
        displacement_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    });

    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &uniform_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: vertex_uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: fragment_uniform_buffer.as_entire_binding(),
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
                resource: bones_buffer.as_entire_binding(),
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

    return (uniform_bind_group, vertex_buffer);
}
