use std::collections::HashMap;

use cgmath::Vector3;

use crate::{renderer::{transform::Transform, vertex::Vertex}, world::material::Material};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    Cube,
    Camera,
    Sphere,
    Mesh,
    StaticMesh,
    SkinnedMesh,
    Skybox,
    Grabbable,
    TabletMenu,
    TabletMenuButton
}

fn new_bone_vec(amount: usize) -> Vec<(Transform, Transform, i64)> {
    let mut bones = Vec::new();
    for _ in 0..amount {
        bones.push((Transform::zero(), Transform::zero(), -1));
    }
    bones
}

// this is a game object, and will be used to render the vertices
#[derive(Clone)]
pub struct Object {
    object_type: ObjectType,
    transform: Transform,
    vertices: Vec<(Vec<Vertex>, String)>,
    materials: HashMap<String, Material>,
    bones: Vec<(Transform, Transform, i64)>,
    skeleton: HashMap<String, usize>,
    movable: bool,
    tag: String
}
impl Object {
    pub fn create(object_type: ObjectType, meshes: Vec<(Vec<Vertex>, String)>) -> Self {
        let mut materials = HashMap::new();
        materials.insert("default".to_string(), Material { texture: "textures/missing.png".to_string(), displacement: "".to_string() });
        Self {
            object_type: object_type,
            transform: Transform::zero(),
            vertices: meshes,
            materials,
            bones: Vec::new(),
            skeleton: HashMap::new(),
            movable: false,
            tag: "unnamed".to_string()
        }
    }

    pub fn add_material(&mut self, material: Material, name: String) {
        self.materials.insert(name, material);
    }
    pub fn add_meshes(&mut self, meshes: Vec<(Vec<Vertex>, String)>) {
        self.vertices.extend(meshes);
    }

    pub fn set_position(&mut self, x: f32, y: f32, z: f32) {
        self.transform.position = Vector3::new(x, y, z);
    }
    pub fn set_scale(&mut self, x: f32, y: f32, z: f32) {
        self.transform.scale = Vector3::new(x, y, z);
    }
    pub fn set_rotation(&mut self, x: f32, y: f32, z: f32) {
        self.transform.rotation = Vector3::new(x, y, z);
    }
    pub fn set_rotation_y(&mut self, rotation: f32) {
        self.transform.rotation.y = rotation;
    }
    pub fn set_vertices(&mut self, vertices: Vec<(Vec<Vertex>, String)>) {
        self.vertices = vertices;
    }
    pub fn set_object_type(&mut self, object_type: ObjectType) {
        self.object_type = object_type;
    }

    pub fn clear_vertices(&mut self) {
        self.vertices.clear();
    }
    pub fn set_default_texture(&mut self, texture: &str) {
        self.materials.insert("default".to_string(), Material { texture: texture.to_string(), displacement: "".to_string() });
    }
    pub fn set_displacement(&mut self, texture: &str) {
        for material in self.materials.values_mut() {
            material.displacement = texture.to_string();
        }
    }

    pub fn set_bones(&mut self, bones: HashMap<i64, (usize, Transform, String, i64)>,
        position: Vector3<f32>, rotation: Vector3<f32>, scale: Vector3<f32>
    ) {
        let mut bones_converted: Vec<(Transform, Transform, i64)> = new_bone_vec(bones.len());
        for bone in bones {
            if bone.1.0 > bones_converted.len() { continue; }
            bones_converted[bone.1.0] = (
                Transform { position: position, rotation: rotation, scale: scale },
                Transform { position: bone.1.1.position, rotation: bone.1.1.rotation, scale: bone.1.1.scale },
                bone.1.3
            );
        }
        self.bones = bones_converted;
    }
    pub fn set_skeleton(&mut self, skeleton: HashMap<String, usize>) {
        self.skeleton = skeleton;
    }

    pub fn set_movable(&mut self, value: bool) {
        self.movable = value;
    }
    pub fn set_tag(&mut self, tag: &str) {
        self.tag = tag.to_string();
    }

    pub fn get_vertices(&self) -> &Vec<(Vec<Vertex>, String)> {
        &self.vertices
    }
    pub fn get_materials(&self) -> &HashMap<String, Material> {
        &self.materials
    }
    pub fn get_object_type(&self) -> ObjectType {
        self.object_type
    }

    pub fn get_bones(&self) -> &Vec<(Transform, Transform, i64)> {
        &self.bones
    }
    pub fn get_skeleton(&self) -> &HashMap<String, usize> {
        &self.skeleton
    }

    pub fn get_movable(&self) -> bool {
        self.movable
    }
    pub fn get_position(&self) -> Vector3<f32> {
        self.transform.position
    }
    pub fn get_rotation(&self) -> Vector3<f32> {
        self.transform.rotation
    }
    pub fn get_scale(&self) -> Vector3<f32> {
        self.transform.scale
    }
    pub fn get_tag(&self) -> &str {
        &self.tag
    }
}