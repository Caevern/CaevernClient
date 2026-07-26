use std::collections::{HashMap, HashSet};

use cgmath::Vector3;

use crate::{
    renderer::{transform::Transform, vertex::create_vertices_skinned},
    world::{
        material::Material,
        object::{Object, ObjectType},
        objects::fbx_parser::parse,
        scene::load_scene,
    },
};

#[derive(Clone)]
pub struct World {
    pub objects: Vec<Object>,
    pub textures: HashSet<String>,
    cameras: Vec<Object>,
}
impl World {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            textures: HashSet::new(),
            cameras: Vec::new(),
        }
    }

    pub fn add_object(&mut self, object: Object) {
        if object.get_object_type() == ObjectType::Camera {
            self.cameras.push(object);
        } else {
            self.objects.push(object);
        }
    }

    pub fn get_object(&self, index: usize) -> &Object {
        &self.objects[index]
    }
    pub fn get_objects(&self) -> &Vec<Object> {
        &self.objects
    }

    pub fn get_camera(&self, index: usize) -> &Object {
        &self.cameras[index]
    }
    pub fn get_cameras(&self) -> &Vec<Object> {
        &self.cameras
    }

    pub fn get_textures(&self) -> &HashSet<String> {
        &self.textures
    }

    pub fn load_world(&mut self, path: &str) {
        let scene = load_scene(path);
        if let Ok(scene) = scene {
            let mut static_world_object = Object::create(ObjectType::StaticMesh, Vec::new());

            for scene_object in scene.objects {
                if scene_object.is_static {
                    let model_parsed = parse(
                        &scene_object.reference,
                        Transform::new(
                            Vector3::new(
                                scene_object.transform.position[0],
                                scene_object.transform.position[1],
                                scene_object.transform.position[2],
                            ),
                            Vector3::new(
                                scene_object.transform.rotation[0],
                                scene_object.transform.rotation[1],
                                scene_object.transform.rotation[2],
                            ),
                            Vector3::new(
                                scene_object.transform.scale[0],
                                scene_object.transform.scale[1],
                                scene_object.transform.scale[2],
                            ),
                        ),
                    );

                    let mut vertices = create_vertices_skinned(&model_parsed.0);
                    for mesh in &mut vertices {
                        if mesh.1 == "default" {
                            mesh.1 = format!(
                                "caevern.material{}",
                                static_world_object.get_materials().len()
                            );
                        }
                    }
                    let material_name = format!(
                        "caevern.material{}",
                        static_world_object.get_materials().len()
                    );
                    let mut object_material = Material::from_texture(&scene_object.texture);
                    self.textures.insert(scene_object.texture);
                    if scene_object.displace != "".to_string() {
                        object_material.set_displacement(&scene_object.displace);
                        self.textures.insert(scene_object.displace);
                    }
                    for material in scene_object.materials {
                        static_world_object
                            .add_material(Material::from_texture(&material.texture), material.name);
                        self.textures.insert(material.texture);
                    }
                    static_world_object.add_material(object_material, material_name);
                    static_world_object.add_meshes(vertices);
                } else {
                    let model_parsed = parse(&scene_object.reference, Transform::zero());

                    let mut object =
                        Object::create(ObjectType::Mesh, create_vertices_skinned(&model_parsed.0));

                    object.set_position(
                        scene_object.transform.position[0],
                        scene_object.transform.position[1],
                        scene_object.transform.position[2],
                    );
                    object.set_rotation(
                        scene_object.transform.rotation[0],
                        scene_object.transform.rotation[1],
                        scene_object.transform.rotation[2],
                    );
                    object.set_scale(
                        scene_object.transform.scale[0],
                        scene_object.transform.scale[1],
                        scene_object.transform.scale[2],
                    );

                    if scene_object.skeleton.len() > 0 {
                        let mut skeleton = HashMap::new();
                        for bone_identifier in scene_object.skeleton {
                            for bone in &model_parsed.1 {
                                let filtered: String =
                                    bone.1.2.chars().filter(|c| (*c as u32) >= 32).collect();
                                if filtered == bone_identifier.bone_name {
                                    skeleton.insert(bone_identifier.body_part, bone.1.0);
                                    break;
                                }
                            }
                        }
                        object.set_skeleton(skeleton);
                    }
                    if model_parsed.1.len() > 0 {
                        let mut bone_values = model_parsed.1;

                        if scene_object.render_bones {
                            for bone in bone_values.iter_mut() {
                                let bone_model_parsed = parse(
                                    "models/sphere.fbx",
                                    Transform::new(
                                        Vector3::new(0.0, 0.0, 0.0),
                                        Vector3::new(0.0, 0.0, 0.0),
                                        Vector3::new(0.001, 0.001, 0.001),
                                    ),
                                );
                                let mut bone_object = Object::create(
                                    ObjectType::Mesh,
                                    create_vertices_skinned(&bone_model_parsed.0),
                                );

                                bone_object.set_position(
                                    -2.5 + bone.1.1.position.x * 0.01,
                                    3.0 + bone.1.1.position.z * 0.01,
                                    bone.1.1.position.y * 0.01,
                                );
                                bone_object.set_default_texture("textures/missing.png");
                                bone_object.set_movable(true);

                                let bone_object_id = self.objects.len();
                                bone.1.4 = bone_object_id;

                                self.add_object(bone_object);
                            }
                        }
                        object.set_bones(
                            bone_values,
                            Vector3::new(0.0, 0.0, 0.0),
                            Vector3::new(0.0, 0.0, 0.0),
                            Vector3::new(1.0, 1.0, 1.0),
                        );
                        object.set_object_type(ObjectType::SkinnedMesh);
                    }

                    for material in scene_object.materials {
                        object
                            .add_material(Material::from_texture(&material.texture), material.name);
                        self.textures.insert(material.texture);
                    }
                    self.add_object(object);
                }
            }

            self.add_object(static_world_object);
        } else {
            println!("Error while loading scene");
        }
    }
}

pub fn create_world() -> World {
    let world: World = World::new();
    world
}
