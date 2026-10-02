use std::{cell::RefCell, rc::Rc};

use wasmtime::Caller;

use crate::{renderer::vertex::Vertex, world::{object::{Object, ObjectType}, world::World}};

pub fn game_create_mesh(
    mut caller: Caller<'_, Rc<RefCell<World>>>,
    vertices_ptr: u32,
    vertices_len: u32,
    uvs_ptr: u32,
    uvs_len: u32,
    indices_ptr: u32,
    indices_len: u32,
) -> u32 {
    let memory = match caller
        .get_export("memory")
        .and_then(|export| export.into_memory())
    {
        Some(memory) => memory,
        None => {
            eprintln!("[MOD] No memory export");
            return 0;
        }
    };

    let data = memory.data(&caller);

    let vertices_start = vertices_ptr as usize;
    let vertices_bytes_len = vertices_len as usize * size_of::<f32>();
    let vertices_end = vertices_start + vertices_bytes_len;

    let uvs_start = uvs_ptr as usize;
    let uvs_bytes_len = uvs_len as usize * size_of::<f32>();
    let uvs_end = uvs_start + uvs_bytes_len;

    let indices_start = indices_ptr as usize;
    let indices_bytes_len = indices_len as usize * size_of::<u32>();
    let indices_end = indices_start + indices_bytes_len;

    let vertices_bytes = &data[vertices_start..vertices_end];
    let uvs_bytes = &data[uvs_start..uvs_end];
    let indices_bytes = &data[indices_start..indices_end];

    let vertices: Vec<f32> = vertices_bytes
        .chunks_exact(4)
        .map(|bytes| f32::from_le_bytes(bytes.try_into().unwrap()))
        .collect();

    let uvs: Vec<f32> = uvs_bytes
        .chunks_exact(4)
        .map(|bytes| f32::from_le_bytes(bytes.try_into().unwrap()))
        .collect();

    let indices: Vec<u32> = indices_bytes
        .chunks_exact(4)
        .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
        .collect();

    let mut skinned_vertices: Vec<Vertex> = Vec::new();

    for index in &indices {
        skinned_vertices.push(Vertex {
            position: [vertices[(*index as usize) * 3], vertices[(*index as usize) * 3 + 1], vertices[(*index as usize) * 3 + 2], 1.0],
            normal: [0.0, 1.0, 0.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
            uv: [uvs[(*index as usize) * 2], uvs[(*index as usize) * 2 + 1], 0.0, 0.0],
            bone_ids: [0.0, 0.0, 0.0, 0.0],
            bone_weights: [0.0, 0.0, 0.0, 0.0],
        });
    }

    println!("[MOD] vertices: {vertices:?}");
    println!("[MOD] indices: {indices:?}");
    for vertex in &skinned_vertices {
        let position = vertex.position;
        println!("[MOD] vertex: {position:?}");
    }

    let meshes: Vec<(Vec<Vertex>, String)> = vec![(skinned_vertices, "default".to_string())];

    let mut object = Object::create(ObjectType::Mesh, meshes);
    object.set_default_texture("textures/white.png");
    object.set_movable(true);

    caller.data().borrow_mut().add_object(object);

    let object_id = caller.data().borrow_mut().get_objects().len() - 1;
    return object_id as u32;
}
