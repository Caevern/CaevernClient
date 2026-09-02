use std::{collections::HashMap, println};
use std::io::Cursor;
use rust_embed::RustEmbed;
use std::io::{self, Read};

use crate::renderer::vertex::create_vertices_skinned;
use crate::renderer::{skinned_vertex::SkinnedVertex, transform::Transform};
use crate::world::object::{Object, ObjectType};
use crate::world::scene::Scene;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

fn read_u32<R: Read>(reader: &mut R) -> io::Result<u32> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;

    Ok(u32::from_le_bytes(bytes))
}

fn read_f32<R: Read>(reader: &mut R) -> io::Result<f32> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;

    Ok(f32::from_le_bytes(bytes))
}

fn read_vec3<R: Read>(reader: &mut R) -> [f32; 3] {
    [
        read_f32(reader).expect("Failed to read x in vec3"),
        read_f32(reader).expect("Failed to read y in vec3"),
        read_f32(reader).expect("Failed to read z in vec3"),
    ]
}

fn read_string<R: Read>(reader: &mut R) -> String {
    let string_length = read_u32(reader).expect("Failed to read string length") as usize;
    let mut bytes = vec![0u8; string_length];
    reader.read_exact(&mut bytes).expect("Failed to read string");

    String::from_utf8(bytes.into_iter().map(|b| b.to_ascii_lowercase()).collect()).expect("Failed to create string")
}

fn read_mesh<R: Read>(mut reader: &mut R, i: u32) -> (
    Vec<SkinnedVertex>,
    Vec<[i8; 3]>,
    Vec<[f32; 3]>,
    Vec<[f32; 2]>,
    String,
) {
    let vertex_count = read_u32(&mut reader).expect("Failed to read vertex count") as usize;
    let mut vertices = vec![(0f32, 0f32, 0f32); vertex_count as usize];
    for i in 0..vertex_count {
        let x = read_f32(&mut reader).expect("Failed to read vertex x");
        let y = read_f32(&mut reader).expect("Failed to read vertex y");
        let z = read_f32(&mut reader).expect("Failed to read vertex z");
        vertices[i] = (x, y, z);
    }
    println!("vertex_count: {vertex_count}");

    let triangle_index_count = read_u32(&mut reader).expect("Failed to read triangle count") as usize;
    let mut indices = vec![0u32; triangle_index_count as usize];
    for i in 0..triangle_index_count {
        let index = read_u32(&mut reader).expect("Failed to read triangle index");
        indices[i] = index;
    }
    let triangle_count = triangle_index_count / 3;
    println!("triangle_count: {triangle_count}");

    let mut mesh_data = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), String::new());

    for i in 0..triangle_index_count {
        let vertex_index = indices[i];
        let vertex = vertices[vertex_index as usize];

        let skinned_vertex = SkinnedVertex {
            position: [vertex.0, vertex.1, vertex.2],
            bone_ids: [0, 0, 0, 0],
            weights: [0.0, 0.0, 0.0, 0.0],
        };

        mesh_data.0.push(skinned_vertex);
        mesh_data.1.push([0, 1, 0]);
        mesh_data.2.push([1.0, 1.0, 1.0]);
        mesh_data.3.push([0.0, 0.0]);
    }

    mesh_data
}

fn read_object<R: Read>(mut reader: &mut R, i: u32, meshes: &HashMap<String, (
    Vec<SkinnedVertex>,
    Vec<[i8; 3]>,
    Vec<[f32; 3]>,
    Vec<[f32; 2]>,
    String,
)>) -> (
    Vec<SkinnedVertex>,
    Vec<[i8; 3]>,
    Vec<[f32; 3]>,
    Vec<[f32; 2]>,
    String,
) {
    let mesh_name = read_string(&mut reader);
    println!("mesh {i}: {mesh_name}");

    if let Some(mesh_data) = meshes.get(&mesh_name) {
        let mut mesh = mesh_data.clone();

        let mesh_position = read_vec3(&mut reader);
        let mesh_rotation = read_vec3(&mut reader);
        let mesh_scale = read_vec3(&mut reader);

        for vertex in &mut mesh.0 {
            vertex.position[0] = vertex.position[0] * mesh_scale[0] + mesh_position[0];
            vertex.position[1] = vertex.position[1] * mesh_scale[1] + mesh_position[1];
            vertex.position[2] = vertex.position[2] * mesh_scale[2] + mesh_position[2];
        }

        return mesh
    }
    (Vec::new(), Vec::new(), Vec::new(), Vec::new(), String::new())
}

fn read_object_data<R: Read>(mut reader: &mut R, meshes_found: &HashMap<String, (Vec<SkinnedVertex>, Vec<[i8; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, String)>) -> (
    Vec<(
        Vec<SkinnedVertex>,
        Vec<[i8; 3]>,
        Vec<[f32; 3]>,
        Vec<[f32; 2]>,
        String,
    )>,
    HashMap<i64, (usize, Transform, String, i64, usize)>,
) {
    let mesh_count = read_u32(&mut reader).expect("Failed to read mesh count");
    let mut meshes = Vec::new();
    for j in 0..mesh_count {
        let mesh = read_object(&mut reader, j, meshes_found);
        meshes.push(mesh);
    }

    (meshes, HashMap::new())
}

fn create_object<R: Read>(mut reader: &mut R, meshes: &HashMap<String, (Vec<SkinnedVertex>, Vec<[i8; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, String)>) -> Object {
    let object_name = read_string(&mut reader);
    println!("object_name: {object_name}");

    let object_position = read_vec3(&mut reader);
    let object_rotation = read_vec3(&mut reader);
    let object_scale = read_vec3(&mut reader);

    let mesh_data = read_object_data(reader, meshes);

    let mut object = Object::create(ObjectType::Mesh, create_vertices_skinned(&mesh_data.0));
    object.set_position(object_position[0], object_position[1], object_position[2]);
    object.set_rotation(object_rotation[0] * 0.0174532925, object_rotation[1] * 0.0174532925, object_rotation[2] * 0.0174532925);
    object.set_scale(object_scale[0], object_scale[1], object_scale[2]);

    object
}

pub fn parse_cae(
    path: &str
) -> Vec<Object> {
    let data = Assets::get(path).expect("Failed to get asset").data;
    let mut reader = Cursor::new(data);

    let mut header = [0u8; 4];
    reader.read_exact(&mut header).expect("Failed to read header");

    if &header != b"CAEV" {
        panic!("Not a Caevern file!");
    }
    println!("----------------- LOADING CAEVERN FILE -----------------");

    let version = read_u32(&mut reader).expect("Failed to read version");
    println!("version: {version}");

    let mesh_count = read_u32(&mut reader).expect("Failed to read mesh count");
    println!("\nmesh_count: {mesh_count}");
    let mut meshes = HashMap::new();
    for i in 0..mesh_count {
        let original_name = read_string(&mut reader);
        let mesh_name = format!("{}@{}", original_name, i);
        println!("mesh_name: {mesh_name}");

        let mesh = read_mesh(&mut reader, i);
        meshes.insert(mesh_name, mesh);
    }

    let object_count = read_u32(&mut reader).expect("Failed to read object count");
    println!("\nobject_count: {object_count}");

    let mut objects = Vec::new();
    for _ in 0..object_count {
        let object = create_object(&mut reader, &meshes);
        objects.push(object);
    }

    println!("\n----------------- LOADED CAEVERN FILE ------------------");

    objects
}
