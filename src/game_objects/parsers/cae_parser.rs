use rust_embed::RustEmbed;
use std::io::Cursor;
use std::io::{self, Read};
use std::{collections::HashMap, println};

use crate::game_objects::material::Material;
use crate::game_objects::object::{Object, ObjectType};
use crate::game_objects::parsers::parsed_mesh::ParsedMesh;
use crate::game_objects::parsers::transforms::{rotate_x, rotate_y, rotate_z};
use crate::renderer::skinned_vertex::SkinnedVertex;
use crate::renderer::vertex::create_vertices_skinned;

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

fn read_bool<R: Read>(reader: &mut R) -> io::Result<bool> {
    let mut byte = [0u8; 1];
    reader.read_exact(&mut byte)?;
    Ok(byte[0] != 0)
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
    reader
        .read_exact(&mut bytes)
        .expect("Failed to read string");

    String::from_utf8(bytes.into_iter().map(|b| b.to_ascii_lowercase()).collect())
        .expect("Failed to create string")
}

fn read_mesh<R: Read>(mut reader: &mut R) -> (Vec<SkinnedVertex>, String) {
    let vertex_count = read_u32(&mut reader).expect("Failed to read vertex count") as usize;
    let mut vertices = vec![(0f32, 0f32, 0f32); vertex_count as usize];
    for vertex in vertices.iter_mut().take(vertex_count) {
        let x = read_f32(&mut reader).expect("Failed to read vertex x");
        let y = read_f32(&mut reader).expect("Failed to read vertex y");
        let z = read_f32(&mut reader).expect("Failed to read vertex z");
        *vertex = (x, y, z);
    }

    let triangle_index_count =
        read_u32(&mut reader).expect("Failed to read triangle count") as usize;
    let mut indices = vec![0u32; triangle_index_count as usize];
    for triangle in indices.iter_mut().take(triangle_index_count) {
        let index = read_u32(&mut reader).expect("Failed to read triangle index");
        *triangle = index;
    }

    let uv_count = read_u32(&mut reader).expect("Failed to read uv count") as usize;
    let mut uvs = vec![(0f32, 0f32); uv_count as usize];
    for uv in uvs.iter_mut().take(uv_count) {
        let x = read_f32(&mut reader)
            .expect("Failed to read uv x")
            .rem_euclid(1.0);
        let y = read_f32(&mut reader)
            .expect("Failed to read uv y")
            .rem_euclid(1.0);
        *uv = (x, y);
    }

    let normals_count = read_u32(&mut reader).expect("Failed to read normals count") as usize;
    let mut normals = vec![(0f32, 0f32, 0f32); normals_count as usize];
    for normal in normals.iter_mut().take(normals_count) {
        let x = read_f32(&mut reader).expect("Failed to read normals x");
        let y = read_f32(&mut reader).expect("Failed to read normals y");
        let z = read_f32(&mut reader).expect("Failed to read normals z");
        *normal = (x, y, z);
    }

    let mut mesh_data = (Vec::new(), String::new());

    for index in indices.iter().take(triangle_index_count) {
        let vertex_index = *index;
        let vertex = vertices[vertex_index as usize];
        let uv = uvs[vertex_index as usize];
        let normal = normals[vertex_index as usize];

        let skinned_vertex = SkinnedVertex {
            position: [vertex.0, vertex.1, vertex.2],
            bone_ids: [0, 0, 0, 0],
            weights: [0.0, 0.0, 0.0, 0.0],
            normal: [normal.0, normal.1, normal.2],
            color: [1.0, 1.0, 1.0],
            uv: [uv.0, uv.1],
        };

        mesh_data.0.push(skinned_vertex);
    }

    mesh_data
}

fn read_object<R: Read>(
    mut reader: &mut R,
    i: u32,
    meshes: &HashMap<String, (Vec<SkinnedVertex>, String)>,
    materials: &HashMap<String, Material>,
    _color: (f32, f32, f32),
) -> (Vec<SkinnedVertex>, String) {
    let mesh_name = read_string(&mut reader);
    println!("mesh {i}: {mesh_name}");

    if let Some(mesh_data) = meshes.get(&mesh_name) {
        let mut mesh = mesh_data.clone();

        let mesh_position = read_vec3(&mut reader);
        let mesh_rotation = read_vec3(&mut reader);
        let mesh_scale = read_vec3(&mut reader);

        let material_name = read_string(&mut reader);
        let material = materials
            .get(&material_name)
            .expect("Material doesn't exist");
        mesh.1 = material_name;

        for vertex in &mut mesh.0 {
            vertex.position = rotate_x(vertex.position, mesh_rotation[0].to_radians());
            vertex.position = rotate_y(vertex.position, mesh_rotation[1].to_radians());
            vertex.position = rotate_z(vertex.position, mesh_rotation[2].to_radians());

            vertex.position[0] = vertex.position[0] * mesh_scale[0] + mesh_position[0];
            vertex.position[1] = vertex.position[1] * mesh_scale[1] + mesh_position[1];
            vertex.position[2] = vertex.position[2] * mesh_scale[2] + mesh_position[2];

            vertex.color[0] = material.color.0;
            vertex.color[1] = material.color.1;
            vertex.color[2] = material.color.2;
        }

        return mesh;
    }
    (Vec::new(), "default".to_string())
}

fn read_object_data<R: Read>(
    mut reader: &mut R,
    meshes_found: &HashMap<String, (Vec<SkinnedVertex>, String)>,
    materials: &HashMap<String, Material>,
    color: (f32, f32, f32),
) -> ParsedMesh {
    let mesh_count = read_u32(&mut reader).expect("Failed to read mesh count");
    let mut meshes = Vec::new();

    for j in 0..mesh_count {
        let mesh = read_object(&mut reader, j, meshes_found, materials, color);
        meshes.push(mesh);
    }

    ParsedMesh {
        vertices: meshes,
        bones: HashMap::new(),
    }
}

fn create_object<R: Read>(
    mut reader: &mut R,
    meshes: &HashMap<String, (Vec<SkinnedVertex>, String)>,
    materials: &HashMap<String, Material>,
) -> Object {
    let object_name = read_string(&mut reader);
    println!("object_name: {object_name}");

    let object_position = read_vec3(&mut reader);
    let object_rotation = read_vec3(&mut reader);
    let object_scale = read_vec3(&mut reader);

    let material_name = read_string(&mut reader);
    //let material = Material::from_texture(&material_name);
    println!("material_name: {material_name}");

    let mut color = (1.0, 1.0, 1.0);

    if material_name != "missing" {
        let r = read_f32(&mut reader).expect("Failed to read r");
        let g = read_f32(&mut reader).expect("Failed to read g");
        let b = read_f32(&mut reader).expect("Failed to read b");
        let _ = read_f32(&mut reader).expect("Failed to read a");

        color = (r, g, b);
    }

    let mesh_data = read_object_data(reader, meshes, materials, color);

    let mut object = Object::create(
        ObjectType::Mesh,
        create_vertices_skinned(&mesh_data.vertices),
    );
    object.set_position(object_position[0], object_position[1], object_position[2]);
    object.set_rotation(
        object_rotation[0].to_radians(),
        object_rotation[1].to_radians(),
        object_rotation[2].to_radians(),
    );
    object.set_scale(object_scale[0], object_scale[1], object_scale[2]);
    object.set_default_texture("Image");
    //object.add_material(material, "default".to_string());

    object
}

pub fn parse_cae(path: &str) -> (Vec<Object>, HashMap<String, Material>) {
    let data = Assets::get(path).expect("Failed to get asset").data;
    let mut reader = Cursor::new(data);

    let mut header = [0u8; 4];
    reader
        .read_exact(&mut header)
        .expect("Failed to read header");

    if &header != b"CAEV" {
        panic!("Not a Caevern file!");
    }
    println!("----------------- LOADING CAEVERN FILE -----------------\n");

    let version = read_u32(&mut reader).expect("Failed to read version");
    println!("version: {version}");

    let mesh_count = read_u32(&mut reader).expect("Failed to read mesh count");
    let mut meshes = HashMap::new();
    for i in 0..mesh_count {
        let original_name = read_string(&mut reader);
        let mesh_name = format!("{}@{}", original_name, i);
        let mesh = read_mesh(&mut reader);

        println!("mesh_name: {mesh_name}");
        meshes.insert(mesh_name, mesh);
    }

    let material_count = read_u32(&mut reader).expect("Failed to read material count");
    let mut materials = HashMap::new();
    for _ in 0..material_count {
        let material_name = read_string(&mut reader);
        println!("{}", material_name);

        let r = read_f32(&mut reader).expect("Failed to read r");
        let g = read_f32(&mut reader).expect("Failed to read g");
        let b = read_f32(&mut reader).expect("Failed to read b");
        let _ = read_f32(&mut reader).expect("Failed to read a");

        let color = (r, g, b);

        let mut material_object = Material::from_color(color);

        let texture_exists = read_bool(&mut reader).expect("Failed to read texture_exists");
        if texture_exists {
            let texture_length =
                read_u32(&mut reader).expect("Failed to read texture_length") as usize;
            let mut texture_data = vec![0u8; texture_length];
            reader
                .read_exact(&mut texture_data)
                .expect("Failed to read texture data");

            let image = image::load_from_memory(&texture_data).expect("Failed to decode texture");

            material_object.set_image(Some(image));
        }

        materials.insert(material_name, material_object);
    }

    let object_count = read_u32(&mut reader).expect("Failed to read object count");
    println!("\nobject_count: {object_count}");

    let mut objects = Vec::new();
    for _ in 0..object_count {
        let object = create_object(&mut reader, &meshes, &materials);
        objects.push(object);
    }

    println!("\n----------------- LOADED CAEVERN FILE ------------------");

    (objects, materials)
}
