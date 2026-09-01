use std::{collections::HashMap, println};
use std::io::Cursor;
use rust_embed::RustEmbed;
use std::io::{self, Read};

use crate::renderer::{skinned_vertex::SkinnedVertex, transform::Transform};

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
    let mesh_name = read_string(&mut reader);
    println!("mesh {i}: {mesh_name}");

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

    (Vec::new(), Vec::new(), Vec::new(), Vec::new(), String::new())
}

pub fn parse_cae(
    path: &str
) {
    let data = Assets::get(path).expect("Failed to get asset").data;
    let mut reader = Cursor::new(data);

    let mut header = [0u8; 4];
    reader.read_exact(&mut header).expect("Failed to read header");

    if &header != b"CAEV" {
        panic!("Not a Caevern file!");
    }
    println!("----------------- LOADING CAEVERN FILE -----------------");

    let version = read_u32(&mut reader).expect("Failed to read version");
    println!("version: {version}\n");
    let mesh_count = read_u32(&mut reader).expect("Failed to read mesh count");
    println!("mesh_count: {mesh_count}");

    for i in 0..mesh_count {
        let mesh = read_mesh(&mut reader, i);
    }

    println!("\n----------------- LOADED CAEVERN FILE ------------------");
}
