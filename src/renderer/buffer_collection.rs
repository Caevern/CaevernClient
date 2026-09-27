use std::collections::HashMap;

use crate::renderer::{shader_type::ShaderType, texture_object::TextureObject, transform::Transform};

pub struct BufferCollection {
    pub textures: HashMap<String, TextureObject>,
    pub font_maps: HashMap<String, HashMap<String, (f32, f32, f32, f32, f32)>>,

    pub final_matrices: Vec<Vec<[[f32; 4]; 4]>>,

    pub bones: Vec<Vec<(Transform, Transform, i64, usize)>>,
    pub bone_buffers: Vec<wgpu::Buffer>,

    pub shader_type: Vec<ShaderType>,

    pub vertex_buffers: Vec<Vec<wgpu::Buffer>>,
    pub uniform_bind_groups: Vec<Vec<wgpu::BindGroup>>,
    pub num_vertices: Vec<Vec<u32>>,

    pub model_uniform_buffers: Vec<wgpu::Buffer>,

    pub uniform_bind_group_layout: wgpu::BindGroupLayout,
    pub vertex_uniform_buffer: wgpu::Buffer,
    pub fragment_uniform_buffer: wgpu::Buffer,
}
