use bytemuck::{Pod, Zeroable};
use std::mem;

use crate::world::objects::fbx_parser::SkinnedVertex;

// The vertex struct is one singular vertex
#[repr(C)]
#[derive(Clone, Copy, Debug, Zeroable, Pod)]
pub struct Vertex {
    pub position: [f32; 4],
    pub normal: [f32; 4],
    pub color: [f32; 4],
    pub uv: [f32; 4],
    pub bone_ids: [f32; 4],
    pub bone_weights: [f32; 4],
}
impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 6] = wgpu::vertex_attr_array![
    0=>Float32x4, 1=>Float32x4, 2=>Float32x4, 3=>Float32x4, 4=>Float32x4, 5=>Float32x4
    ];
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

// Convert vertices to set vertex size
fn vertex(p: [f32; 3], n: [i8; 3], c: [f32; 3], u: [f32; 2], bi: [u32; 4], bw: [f32; 4]) -> Vertex {
    return Vertex {
        position: [p[0], p[1], p[2], 1.0],
        normal: [n[0] as f32, n[1] as f32, n[2] as f32, 1.0],
        color: [c[0], c[1], c[2], 1.0],
        uv: [u[0], u[1], 0.0, 0.0],
        bone_ids: [bi[0] as f32, bi[1] as f32, bi[2] as f32, bi[3] as f32],
        bone_weights: bw,
    };
}

// Convert object to list of vertices
pub fn create_vertices(
    meshes: &Vec<(
        Vec<[f32; 3]>,
        Vec<[i8; 3]>,
        Vec<[f32; 3]>,
        Vec<[f32; 2]>,
        String,
    )>,
) -> Vec<(Vec<Vertex>, String)> {
    let mut vertex_list: Vec<(Vec<Vertex>, String)> = Vec::with_capacity(meshes.len());
    for mesh in 0..meshes.len() {
        let vertices = &meshes[mesh].0;
        let normals = &meshes[mesh].1;
        let colors = &meshes[mesh].2;
        let uvs = &meshes[mesh].3;
        vertex_list.push((Vec::with_capacity(vertices.len()), meshes[mesh].4.clone()));
        for i in 0..vertices.len() {
            vertex_list[mesh].0.push(vertex(
                vertices[i],
                normals[i],
                colors[i],
                uvs[i],
                [0; 4],
                [0.0; 4],
            ));
        }
    }
    return vertex_list.to_vec();
}

pub fn create_vertices_skinned(
    meshes: &Vec<(
        Vec<SkinnedVertex>,
        Vec<[i8; 3]>,
        Vec<[f32; 3]>,
        Vec<[f32; 2]>,
        String,
    )>,
) -> Vec<(Vec<Vertex>, String)> {
    let mut vertex_list: Vec<(Vec<Vertex>, String)> = Vec::with_capacity(meshes.len());
    for mesh in 0..meshes.len() {
        let vertices = &meshes[mesh].0;
        let normals = &meshes[mesh].1;
        let colors = &meshes[mesh].2;
        let uvs = &meshes[mesh].3;
        vertex_list.push((Vec::with_capacity(vertices.len()), meshes[mesh].4.clone()));
        for i in 0..vertices.len() {
            vertex_list[mesh].0.push(vertex(
                vertices[i].position,
                normals[i],
                colors[i],
                uvs[i],
                vertices[i].bone_ids,
                vertices[i].weights,
            ));
        }
    }
    return vertex_list.to_vec();
}
