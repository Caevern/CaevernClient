use std::collections::HashMap;

use crate::{renderer::texture_object::TextureObject, setup::fonts::load_font_atlas};

pub fn register_default_textures(
    textures: &mut HashMap<String, TextureObject>,
    device: &wgpu::Device,
) {
    textures.insert(
        "textures/missing.png".to_string(),
        TextureObject::create("textures/missing.png", &device),
    );
    textures.insert(
        "textures/tablet.png".to_string(),
        TextureObject::create("textures/tablet.png", &device),
    );
    textures.insert(
        "textures/displacement.png".to_string(),
        TextureObject::create("textures/displacement.png", &device),
    );

    textures.insert(
        "fonts/NotoSansJP.ttf".to_string(),
        TextureObject::load_from_dynamic_image(load_font_atlas("fonts/NotoSansJP.ttf"), &device),
    );
}
