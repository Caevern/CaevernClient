use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

#[derive(Debug, Serialize, Deserialize)]
pub struct Scene {
    pub name: String,
    pub id: u32,
    pub creator: String,
    pub objects: Vec<Object>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Object {
    #[serde(default = "default_on")]
    pub is_static: bool,
    #[serde(default = "default_off")]
    pub render_bones: bool,

    pub reference: String,

    #[serde(default = "default_texture")]
    pub texture: String,

    #[serde(default = "default_materials")]
    pub materials: Vec<Material>,

    #[serde(default = "default_skeleton")]
    pub skeleton: Vec<BoneIdentifier>,

    #[serde(default = "default_displace")]
    pub displace: String,

    pub transform: Transform,
}

fn default_off() -> bool {
    false
}
fn default_on() -> bool {
    true
}

fn default_texture() -> String {
    "textures/missing.png".to_string()
}

fn default_materials() -> Vec<Material> {
    Vec::new()
}
fn default_skeleton() -> Vec<BoneIdentifier> {
    Vec::new()
}

fn default_displace() -> String {
    "".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Material {
    pub name: String,
    pub texture: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BoneIdentifier {
    pub body_part: String,
    pub bone_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transform {
    #[serde(default = "default_position")]
    pub position: [f32; 3],

    #[serde(default = "default_rotation")]
    pub rotation: [f32; 3],

    #[serde(default = "default_scale")]
    pub scale: [f32; 3],
}

fn default_position() -> [f32; 3] {
    [0.0, 0.0, 0.0]
}

fn default_rotation() -> [f32; 3] {
    [0.0, 0.0, 0.0]
}

fn default_scale() -> [f32; 3] {
    [1.0, 1.0, 1.0]
}

pub fn load_scene(path: &str) -> Result<Scene, Box<dyn std::error::Error>> {
    let data = Assets::get(path).expect("Failed to get scene").data;
    let scene: Scene = serde_json::from_slice(&data)?;
    Ok(scene)
}
