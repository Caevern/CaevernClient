#[derive(Clone, Copy)]
pub struct BufferBindings {
    pub vertex_buffer: usize,
    pub uniform_bind_group: usize,
    pub num_vertices: usize,

    pub final_matrices: usize,
    pub bones: usize,
    pub bone_buffer: usize,
    pub shader_type: usize,

    pub model_uniform_buffer: usize,
}
impl BufferBindings {
    pub fn empty() -> Self {
        Self {
            vertex_buffer: 0,
            uniform_bind_group: 0,
            num_vertices: 0,
            final_matrices: 0,
            bones: 0,
            bone_buffer: 0,
            shader_type: 0,
            model_uniform_buffer: 0,
        }
    }
}
