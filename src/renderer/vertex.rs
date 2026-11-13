use bytemuck::{Pod, Zeroable};
use std::mem;

use crate::renderer::{simple_vertex::SimpleVertex, skinned_vertex::SkinnedVertex};

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
fn vertex(
    p: [f32; 3],
    n: [f32; 3],
    c: [f32; 3],
    u: [f32; 2],
    bi: [u32; 4],
    bw: [f32; 4],
) -> Vertex {
    Vertex {
        position: [p[0], p[1], p[2], 1.0],
        normal: [n[0], n[1], n[2], 1.0],
        color: [c[0], c[1], c[2], 1.0],
        uv: [u[0], u[1], 0.0, 0.0],
        bone_ids: [bi[0] as f32, bi[1] as f32, bi[2] as f32, bi[3] as f32],
        bone_weights: bw,
    }
}

pub fn create_vertices(meshes: &[(Vec<SimpleVertex>, String)]) -> Vec<(Vec<Vertex>, String)> {
    let mut vertex_list: Vec<(Vec<Vertex>, String)> = Vec::with_capacity(meshes.len());
    for mesh in 0..meshes.len() {
        let vertices = &meshes[mesh].0;
        let material = &meshes[mesh].1;
        vertex_list.push((Vec::with_capacity(vertices.len()), material.clone()));
        for v in vertices {
            vertex_list[mesh].0.push(vertex(
                v.position, v.normal, v.color, v.uv, [0; 4], [0.0; 4],
            ));
        }
    }
    vertex_list.to_vec()
}

pub fn create_vertices_skinned(
    meshes: &[(Vec<SkinnedVertex>, String)],
) -> Vec<(Vec<Vertex>, String)> {
    let mut vertex_list: Vec<(Vec<Vertex>, String)> = Vec::with_capacity(meshes.len());
    for mesh in 0..meshes.len() {
        let vertices = &meshes[mesh].0;
        let material = &meshes[mesh].1;
        vertex_list.push((Vec::with_capacity(vertices.len()), material.clone()));
        for v in vertices {
            vertex_list[mesh].0.push(vertex(
                v.position, v.normal, v.color, v.uv, v.bone_ids, v.weights,
            ));
        }
    }
    vertex_list.to_vec()
}
