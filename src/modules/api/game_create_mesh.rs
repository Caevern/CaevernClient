use wasmtime::Caller;

use crate::renderer::vertex::Vertex;

pub fn game_create_mesh(
    mut caller: Caller<'_, ()>,
    vertices_ptr: u32,
    vertices_len: u32,
    indices_ptr: u32,
    indices_len: u32,
) {
    let memory = match caller
        .get_export("memory")
        .and_then(|export| export.into_memory())
    {
        Some(memory) => memory,
        None => {
            eprintln!("[MOD] No memory export");
            return;
        }
    };

    let data = memory.data(&caller);

    let vertices_start = vertices_ptr as usize;
    let vertices_bytes_len = vertices_len as usize * size_of::<f32>();
    let vertices_end = vertices_start + vertices_bytes_len;

    let indices_start = indices_ptr as usize;
    let indices_bytes_len = indices_len as usize * size_of::<u32>();
    let indices_end = indices_start + indices_bytes_len;

    let vertices_bytes = &data[vertices_start..vertices_end];
    let indices_bytes = &data[indices_start..indices_end];

    let vertices: Vec<f32> = vertices_bytes
        .chunks_exact(4)
        .map(|bytes| f32::from_le_bytes(bytes.try_into().unwrap()))
        .collect();

    let indices: Vec<u32> = indices_bytes
        .chunks_exact(4)
        .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
        .collect();

    println!("[MOD] vertices: {vertices:?}");
    println!("[MOD] indices: {indices:?}");

    let mut mesh = Vec::new();
    for index in &indices {
        if *index as usize >= vertices.len() {
            continue;
        }

        let vertex_index = *index as usize * 3;
        let vertex = [
            vertices[vertex_index],
            vertices[vertex_index + 1],
            vertices[vertex_index + 2],
            0.0,
        ];

        let skinned_vertex = Vertex {
            position: vertex,
            normal: [0.0, 1.0, 0.0, 1.0],
            color: [1.0, 0.0, 1.0, 1.0],
            uv: [0.0, 0.0, 0.0, 0.0],
            bone_ids: [0.0, 0.0, 0.0, 0.0],
            bone_weights: [0.0, 0.0, 0.0, 0.0],
        };
        mesh.push(skinned_vertex);
    }

    println!("Created mesh, firing callback");
}
