use cgmath::Vector3;
use rust_embed::RustEmbed;
use fbx::{File, Node};
use fbx::Property;
use std::collections::{HashMap, HashSet};
use std::io::{BufReader, Cursor};

use crate::renderer::transform;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

#[derive(Debug)]
struct Mesh {
    vertices: Vec<f64>,
    indices: Vec<i32>,
    uv: Vec<f64>,
    uv_indices: Vec<i32>,
    _normals: Vec<f64>,
    id: i64
}

#[derive(Debug)]
struct Material {
    _id: i64,
    name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    Mesh,
    Bone
}

#[derive(Debug)]
struct Transform {
    translation: (f32, f32, f32),
    rotation: (f32, f32, f32),
    scaling: (f32, f32, f32),
    name: String,
    id: i64,
    parent: i64,
    object: ObjectType
}

pub struct Keyframe {
    pub time: f32,
    pub transform: transform::Transform
}

pub struct BoneAnimation {
    pub bone_id: i64,
    pub keyframes: Vec<Keyframe>
}

#[derive(Debug)]
pub struct AnimationCurve {
    pub id: i64,
    pub times: Vec<i64>,
    pub values: Vec<f32>,
}

#[derive(Debug)]
pub struct AnimationCurveNode {
    pub id: i64,
    pub property: String,
}

#[derive(Debug)]
pub struct AnimationLayer {
    pub id: i64,
}

#[derive(Debug)]
pub struct AnimationStack {
    pub id: i64,
    pub name: String,
}

struct Connection {
    from: i64,
    to: i64,
}

#[derive(Debug)]
struct Cluster {
    id: i64,
    indices: Vec<i32>,
    weights: Vec<f64>,
    bone_id: i64,
}

pub struct SkinnedVertex {
    pub position: [f32; 3],
    pub bone_ids: [u32; 4],
    pub weights: [f32; 4],
}

fn rotate_x(v: [f32; 3], angle: f32) -> [f32; 3] {
    let (s, c) = angle.sin_cos();
    [
        v[0],
        v[1] * c - v[2] * s,
        v[1] * s + v[2] * c,
    ]
}

fn rotate_y(v: [f32; 3], angle: f32) -> [f32; 3] {
    let (s, c) = angle.sin_cos();
    [
        v[0] * c + v[2] * s,
        v[1],
        -v[0] * s + v[2] * c,
    ]
}

fn rotate_z(v: [f32; 3], angle: f32) -> [f32; 3] {
    let (s, c) = angle.sin_cos();
    [
        v[0] * c - v[1] * s,
        v[0] * s + v[1] * c,
        v[2],
    ]
}

fn get_id(node: &Node) -> Option<i64> {
    match node.properties.get(0) {
        Some(Property::I64(id)) => Some(*id),
        Some(Property::I32(id)) => Some(*id as i64),
        _ => None,
    }
}

fn parse_connections(node: &Node) -> Vec<Connection> {
    let mut conns = Vec::new();

    if node.name == "Connections" {
        for child in &node.children {
            if child.name == "C" && child.properties.len() >= 3 {
                let from = match &child.properties[1] {
                    Property::I64(v) => *v,
                    Property::I32(v) => *v as i64,
                    _ => continue,
                };
                let to = match &child.properties[2] {
                    Property::I64(v) => *v,
                    Property::I32(v) => *v as i64,
                    _ => continue,
                };

                conns.push(Connection { from, to });
            }
        }
    }

    conns
}

fn parse_clusters(node: &Node) -> HashMap<i64, Cluster> {
    let mut clusters = HashMap::new();

    if node.name == "Objects" {
        for child in &node.children {
            if child.name == "Deformer" {

                let id = get_id(child).unwrap_or(0);

                let deformer_type = match child.properties.get(2) {
                    Some(Property::String(s)) => s.as_str(),
                    _ => "",
                };

                if deformer_type == "Cluster" {
                    let mut indices = vec![];
                    let mut weights = vec![];

                    for sub in &child.children {
                        match sub.name.as_str() {
                            "Indexes" => {
                                for prop in &sub.properties {
                                    if let Property::I32Array(arr) = prop {
                                        indices.extend(arr);
                                    }
                                }
                            }

                            "Weights" => {
                                for prop in &sub.properties {
                                    if let Property::F64Array(arr) = prop {
                                        weights.extend(arr);
                                    }
                                }
                            }

                            _ => {}
                        }
                    }

                    clusters.insert(id, Cluster {
                        id,
                        indices,
                        weights,
                        bone_id: 0,
                    });
                }
            }
        }
    }

    clusters
}

fn parse_skins(node: &Node) -> HashSet<i64> {
    let mut skins = HashSet::new();

    if node.name == "Objects" {
        for child in &node.children {
            if child.name != "Deformer" {
                continue;
            }

            let id = get_id(child).unwrap_or(0);

            let deformer_type = match child.properties.get(2) {
                Some(Property::String(s)) => s.as_str(),
                _ => "",
            };

            if deformer_type == "Skin" {
                skins.insert(id);
            }
        }
    }

    skins
}

fn _parse_animation_curves(node: &Node)
    -> HashMap<i64, AnimationCurve>
{
    let mut curves = HashMap::new();

    if node.name != "Objects" {
        return curves;
    }

    for child in &node.children {
        if child.name != "AnimationCurve" {
            continue;
        }

        let id = get_id(child).unwrap_or(0);

        let mut times = Vec::new();
        let mut values = Vec::new();

        for sub in &child.children {
            match sub.name.as_str() {
                "KeyTime" => {
                    for prop in &sub.properties {
                        if let Property::I64Array(arr) = prop {
                            times.extend(arr);
                        }
                    }
                }

                "KeyValueFloat" => {
                    for prop in &sub.properties {
                        match prop {
                            Property::F32Array(arr) => {
                                values.extend(arr.iter().copied());
                            }

                            Property::F64Array(arr) => {
                                values.extend(
                                    arr.iter()
                                        .map(|v| *v as f32)
                                );
                            }

                            _ => {}
                        }
                    }
                }

                _ => {}
            }
        }

        curves.insert(id, AnimationCurve {
            id,
            times,
            values,
        });
    }

    curves
}

fn _parse_curve_nodes(node: &Node)
    -> HashMap<i64, AnimationCurveNode>
{
    let mut nodes = HashMap::new();

    if node.name != "Objects" {
        return nodes;
    }

    for child in &node.children {
        if child.name != "AnimationCurveNode" {
            continue;
        }

        let id = get_id(child).unwrap_or(0);

        let property =
            match child.properties.get(1) {
                Some(Property::String(s)) => s.clone(),
                _ => String::new(),
            };

        nodes.insert(id, AnimationCurveNode {
            id,
            property,
        });
    }

    nodes
}

fn parse_animation_stacks(node: &Node)
    -> HashMap<i64, AnimationStack>
{
    let mut stacks = HashMap::new();

    if node.name != "Objects" {
        return stacks;
    }

    for child in &node.children {
        if child.name != "AnimationStack" {
            continue;
        }

        let id = get_id(child).unwrap_or(0);

        let name =
            match child.properties.get(1) {
                Some(Property::String(s)) => s.clone(),
                _ => "Unnamed".into(),
            };

        stacks.insert(id, AnimationStack {
            id,
            name,
        });
    }

    stacks
}

fn parse_animation_layers(node: &Node)
    -> HashMap<i64, AnimationLayer>
{
    let mut layers = HashMap::new();

    if node.name != "Objects" {
        return layers;
    }

    for child in &node.children {
        if child.name != "AnimationLayer" {
            continue;
        }

        let id = get_id(child).unwrap_or(0);

        layers.insert(id, AnimationLayer {
            id,
        });
    }

    layers
}

fn traverse_nodes(node: &Node) -> Vec<Mesh> {
    let mut meshes = Vec::new();

    if node.name == "Geometry" {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut uv = Vec::new();
        let mut uv_indices = Vec::new();
        let mut normals: Vec<f64> = Vec::new();

        for child in &node.children {
            match child.name.as_str() {
                "Vertices" => {
                    for prop in &child.properties {
                        if let Property::F64Array(arr) = prop {
                            vertices.extend(arr);
                        }
                    }
                }
                "PolygonVertexIndex" => {
                    for prop in &child.properties {
                        if let Property::I32Array(arr) = prop {
                            indices.extend(arr);
                        }
                    }
                }
                "LayerElementUV" => {
                    for sub in &child.children {
                        match sub.name.as_str() {
                            "UV" => {
                                for prop in &sub.properties {
                                    if let Property::F64Array(arr) = prop {
                                        uv.extend(arr);
                                    }
                                }
                            }
                            "UVIndex" => {
                                for prop in &sub.properties {
                                    if let Property::I32Array(arr) = prop {
                                        uv_indices.extend(arr);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                "LayerElementNormal" => {
                    for sub in &child.children {
                        match sub.name.as_str() {
                            "Normals" => {
                                for prop in &sub.properties {
                                    if let Property::F64Array(arr) = prop {
                                        normals.extend(arr);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        //println!("{:?}", normals);

        let id: i64;
        if let Some(id_found) = get_id(node) {
            id = id_found;
        } else {
            id = 0;
        }

        meshes.push(Mesh {
            vertices,
            indices,
            uv,
            uv_indices,
            _normals: normals,
            id
        });
    }

    // Recurse into children
    for child in &node.children {
        meshes.extend(traverse_nodes(child));
    }

    meshes
}

fn get_transform(node: &Node) -> Option<Transform> {
    if node.name == "Model" {
        let mut name = String::new();
        if let Some(Property::String(s)) = node.properties.get(1) {
            name = s.clone();
        }

        let mut translation = (0.0, 0.0, 0.0);
        let mut rotation = (0.0, 0.0, 0.0);
        let mut scaling = (1.0, 1.0, 1.0);

        for child in &node.children {
            if child.name == "Properties70" {
                for p in &child.children {
                    if p.name == "P" {
                        // Property format: ["Lcl Translation", "Lcl Translation", "", "A", x, y, z]
                        if p.properties.len() >= 7 {
                            // First element is a String
                            let prop_name = match &p.properties[0] {
                                Property::String(s) => s.as_str(),
                                _ => continue,
                            };

                            // Extract x, y, z as f64
                            let mut nums = [0.0, 0.0, 0.0];
                            for (i, n) in nums.iter_mut().enumerate() {
                                let idx = 4 + i;
                                if let Some(prop) = p.properties.get(idx) {
                                    *n = match prop {
                                        Property::F64(v) => *v as f32,
                                        Property::I32(v) => *v as f32,
                                        Property::I64(v) => *v as f32,
                                        _ => 0.0,
                                    };
                                }
                            }

                            match prop_name {
                                "Lcl Translation" => translation = (nums[0], nums[1], nums[2]),
                                "Lcl Rotation" => rotation = (nums[0], nums[1], nums[2]),
                                "Lcl Scaling" => scaling = (nums[0], nums[1], nums[2]),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        let node_type = match node.properties.get(2) {
        Some(Property::String(s)) => s.as_str(),
            _ => "",
        };


        let id: i64;
        if let Some(id_found) = get_id(node) {
            id = id_found;
        } else {
            id = 0;
        }

        let mut object = ObjectType::Mesh;
        if node_type == "LimbNode" {
            object = ObjectType::Bone;
        }

        return Some(Transform {
            translation,
            rotation,
            scaling,
            name,
            id,
            parent: -1,
            object
        });
    }

    None
}

fn parse_materials(node: &Node) -> HashMap<i64, Material> {
    let mut materials = HashMap::new();

    if node.name == "Objects" {
        for child in &node.children {
            if child.name == "Material" {
                let id = get_id(child).unwrap_or(0);

                let name = match child.properties.get(1) {
                    Some(Property::String(s)) => s.clone(),
                    _ => "Unnamed".to_string(),
                };

                println!("MATERIAL: {} {}", name, id);

                materials.insert(id, Material { _id: id, name });
            }
        }
    }

    materials
}

fn pack_weights(
    influences: &[(i64, f32)],
    bone_map: &HashMap<i64, (usize, transform::Transform, String, i64)>,
) -> ([u32; 4], [f32; 4]) {

    let mut ids = [0u32; 4];
    let mut weights = [0.0f32; 4];

    let mut sorted = influences.to_vec();

    sorted.sort_by(|a, b|
        b.1.partial_cmp(&a.1).unwrap());

    for (i, (bone_id, weight))
        in sorted.iter().take(4).enumerate()
    {
        ids[i] = bone_map[bone_id].0 as u32;
        weights[i] = *weight;
    }

    let total: f32 = weights.iter().sum();

    if total > 0.0 {
        for w in &mut weights {
            *w /= total;
        }
    }

    (ids, weights)
}

pub fn parse(path: &str, global_transform: transform::Transform) -> (Vec<(Vec<SkinnedVertex>, Vec<[i8; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, String)>, HashMap<i64, (usize, transform::Transform, String, i64)>) {
    let data = Assets::get(path).expect("Failed to get asset").data;
    let cursor = Cursor::new(data);
    let mut reader = BufReader::new(cursor);
    let file = File::read_from(&mut reader).expect("Failed to parse FBX file");

    let mut mesh_data: Vec<(Vec<SkinnedVertex>, Vec<[i8; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, String)> = Vec::new();

    let mut transforms = HashMap::new();
    let mut materials = HashMap::new();
    let mut connections = Vec::new();
    let mut clusters = HashMap::new();
    let mut skins = HashSet::new();
    let mut geometry_clusters: HashMap<i64, Vec<i64>> = HashMap::new();
    let mut bone_map = HashMap::new();

    for node in &file.children {
        materials.extend(parse_materials(node));
    }
    for node in &file.children {
        skins.extend(parse_skins(node));
    }
    for node in &file.children {
        let animation_stack = parse_animation_stacks(node);
        for animation in animation_stack {
            println!("{} {}", animation.0, animation.1.name);
        }
    }
    for node in &file.children {
        let animation_layer = parse_animation_layers(node);
        for animation in animation_layer {
            println!("{}", animation.0);
        }
    }
    /*for node in &file.children {
        let animation_curve_node = parse_curve_nodes(node);
        for animation in animation_curve_node {
            println!("{} {}", animation.0, animation.1.property);
        }
    }
    for node in &file.children {
        let animation_curve = parse_animation_curves(node);
        for animation in animation_curve {
            println!("{} {} {}", animation.0, animation.1.times.len(), animation.1.values.len());
        }
    }*/

    for node in &file.children {
        if node.name == "Objects" {
            for child in &node.children {
                if let Some(transform) = get_transform(child) {
                    transforms.insert(transform.id, transform);
                }
            }
        }
        connections.extend(parse_connections(node));
    }

    for node in &file.children {
        clusters.extend(parse_clusters(node));
    }
    for conn in &connections {
        if clusters.contains_key(&conn.to) {
            if let Some(cluster) = clusters.get_mut(&conn.to) {
                cluster.bone_id = conn.from;
            }
        }
    }

    for cluster in clusters.values() {
        let mut skin_id = None;

        for conn in &connections {
            if conn.from == cluster.id {
                if skins.contains(&conn.to) {
                    skin_id = Some(conn.to);
                    break;
                }
            }
        }

        let Some(skin_id) = skin_id else {
            continue;
        };

        for conn in &connections {
            if conn.from == skin_id {
                geometry_clusters
                    .entry(conn.to)
                    .or_default()
                    .push(cluster.id);
            }
        }
    }

    for (geom, cluster_ids) in &geometry_clusters {
        println!(
            "Geometry {} has {} clusters",
            geom,
            cluster_ids.len()
        );
    }

    let existing_keys: HashSet<_> = transforms.keys().cloned().collect();
    for transform in transforms.iter_mut() {
        for connection in &connections {
            if &connection.from == transform.0 && transform.1.object == ObjectType::Bone {
                if existing_keys.contains(&connection.to) {
                    transform.1.parent = connection.to;
                }
            }
        }
    }

    let bones: Vec<_> = transforms.values().filter(|t| t.object == ObjectType::Bone).collect();
    for (index, bone) in bones.iter().enumerate() {
        let mut parent_bone_index = -1;
        for (parent_index, parent_bone) in bones.iter().enumerate() {
            if parent_bone.id == bone.parent {
                parent_bone_index = parent_index as i64;
            }
        }
        bone_map.insert(bone.id, (index,
            transform::Transform{
                position: Vector3::new( bone.translation.0, bone.translation.1, bone.translation.2 ),
                rotation: Vector3::new( bone.rotation.0, bone.rotation.1, bone.rotation.2 ),
                scale: Vector3::new( bone.scaling.0, bone.scaling.1, bone.scaling.2 ),
            }, bone.name.clone(), parent_bone_index
        ));
    }

    let mut selected_material = "default";

    for node in &file.children {
        let meshes = traverse_nodes(node);
        for (index, mesh) in meshes.iter().enumerate() {
            let mut transform = &Transform{
                translation: (3.0 * index as f32, 0.0, 0.0),
                rotation: (0.0, 0.0, 0.0),
                scaling: (0.0, 0.0, 0.0),
                name: "Unknown".to_string(),
                id: 0,
                parent: -1,
                object: ObjectType::Bone
            };
            let mut model_id = None;
            for connection in &connections {
                if connection.from == mesh.id {
                    model_id = Some(connection.to);
                    if let Some(transform_found) = transforms.get(&connection.to) {
                        if transform_found.object == ObjectType::Bone { continue; }
                        transform = transform_found;
                    }
                }
            }
            let mut material = None;

            if let Some(model_id) = model_id {
                for connection in &connections {
                    if connection.to == model_id {
                        if let Some(mat) = materials.get(&connection.from) {
                            material = Some(mat);
                        }
                    }
                }
            }

            let vertex_count = mesh.vertices.len() / 3;
            let mut vertex_weights = vec![Vec::<(i64, f32)>::new(); vertex_count];

            if let Some(cluster_ids) = geometry_clusters.get(&mesh.id) {
                for cluster_id in cluster_ids {
                    let cluster = &clusters[cluster_id];

                    for (&vertex_idx, &weight) in cluster.indices.iter().zip(cluster.weights.iter()) {
                        vertex_weights[vertex_idx as usize].push((cluster.bone_id, weight as f32));
                    }
                }
            }

            if let Some(mat) = material {
                selected_material = &mat.name;
                println!("{} uses material: {}", transform.name, mat.name);
            }

            let mut triangles = vec![];
            let mut current_polygon = vec![];
            let mut current_uvs = vec![];

            mesh_data.push((Vec::new(), Vec::new(), Vec::new(), Vec::new(), selected_material.to_string()));
            let mesh_data_index = mesh_data.len() - 1;

            for (index_uv, i) in mesh.indices.iter().enumerate() {
                let index = if *i < 0 {
                    ((-i) - 1) as usize
                } else {
                    *i as usize
                };
                current_polygon.push(index);
                current_uvs.push(mesh.uv_indices[index_uv as usize]);

                if *i < 0 {
                    // polygon ended, triangulate
                    for j in 1..current_polygon.len() - 1 {
                        triangles.push([
                            current_polygon[0],
                            current_polygon[j],
                            current_polygon[j + 1],
                            current_uvs[0] as usize,
                            current_uvs[j] as usize,
                            current_uvs[j + 1] as usize,
                        ]);
                    }
                    current_polygon.clear();
                    current_uvs.clear();
                }
            }

            for tri in triangles {
                let mut v = [
                    mesh.vertices[tri[0]*3] as f32 * global_transform.scale.x * transform.scaling.0,
                    mesh.vertices[tri[0]*3+1] as f32 * global_transform.scale.z * transform.scaling.2,
                    mesh.vertices[tri[0]*3+2] as f32 * global_transform.scale.y * transform.scaling.1,
                ];

                v = rotate_x(v, (global_transform.rotation.x + transform.rotation.0) * 0.0174532925);
                v = rotate_y(v, (global_transform.rotation.y + transform.rotation.1) * 0.0174532925);
                v = rotate_z(v, (global_transform.rotation.z + transform.rotation.2) * 0.0174532925);

                // translate
                v[0] += global_transform.position.x + transform.translation.0;
                v[1] += global_transform.position.y + transform.translation.1;
                v[2] += global_transform.position.z + transform.translation.2;

                let (bone_ids, weights) = pack_weights(&vertex_weights[tri[0]], &bone_map);

                mesh_data[mesh_data_index].0.push(SkinnedVertex{ position: v, bone_ids, weights});
                mesh_data[mesh_data_index].3.push([
                    mesh.uv[tri[3] * 2] as f32,
                    1.0 - mesh.uv[tri[3] * 2 + 1] as f32
                ]);
                mesh_data[mesh_data_index].1.push([0, 1, 0]);
                mesh_data[mesh_data_index].2.push([1.0, 1.0, 1.0]);

                let mut v = [
                    mesh.vertices[tri[1]*3] as f32 * global_transform.scale.x * transform.scaling.0,
                    mesh.vertices[tri[1]*3+1] as f32 * global_transform.scale.z * transform.scaling.2,
                    mesh.vertices[tri[1]*3+2] as f32 * global_transform.scale.y * transform.scaling.1,
                ];

                v = rotate_x(v, (global_transform.rotation.x + transform.rotation.0) * 0.0174532925);
                v = rotate_y(v, (global_transform.rotation.y + transform.rotation.1) * 0.0174532925);
                v = rotate_z(v, (global_transform.rotation.z + transform.rotation.2) * 0.0174532925);

                // translate
                v[0] += global_transform.position.x + transform.translation.0;
                v[1] += global_transform.position.y + transform.translation.1;
                v[2] += global_transform.position.z + transform.translation.2;

                let (bone_ids, weights) = pack_weights(&vertex_weights[tri[1]], &bone_map);

                mesh_data[mesh_data_index].0.push(SkinnedVertex { position: v, bone_ids, weights });
                mesh_data[mesh_data_index].3.push([
                    mesh.uv[tri[4] * 2] as f32, 
                    1.0 - mesh.uv[tri[4] * 2 + 1] as f32
                ]);
                mesh_data[mesh_data_index].1.push([0, 1, 0]);
                mesh_data[mesh_data_index].2.push([1.0, 1.0, 1.0]);

                let mut v = [
                    mesh.vertices[tri[2]*3] as f32 * global_transform.scale.x * transform.scaling.0,
                    mesh.vertices[tri[2]*3+1] as f32 * global_transform.scale.z * transform.scaling.2,
                    mesh.vertices[tri[2]*3+2] as f32 * global_transform.scale.y * transform.scaling.1,
                ];

                v = rotate_x(v, (global_transform.rotation.x + transform.rotation.0) * 0.0174532925);
                v = rotate_y(v, (global_transform.rotation.y + transform.rotation.1) * 0.0174532925);
                v = rotate_z(v, (global_transform.rotation.z + transform.rotation.2) * 0.0174532925);

                // translate
                v[0] += global_transform.position.x + transform.translation.0;
                v[1] += global_transform.position.y + transform.translation.1;
                v[2] += global_transform.position.z + transform.translation.2;

                let (bone_ids, weights) = pack_weights(&vertex_weights[tri[2]], &bone_map);

                mesh_data[mesh_data_index].0.push(SkinnedVertex { position: v, bone_ids, weights });
                mesh_data[mesh_data_index].3.push([
                    mesh.uv[tri[5] * 2] as f32,
                    1.0 - mesh.uv[tri[5] * 2 + 1] as f32
                ]);
                mesh_data[mesh_data_index].1.push([0, 1, 0]);
                mesh_data[mesh_data_index].2.push([1.0, 1.0, 1.0]);
            }

            if mesh_data[mesh_data_index].0.len() == 0 {
                mesh_data.remove(mesh_data_index);
            }
        }
    }

    (mesh_data, bone_map)
}