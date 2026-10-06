use std::collections::HashMap;

use fontdue::Font;
use image::{DynamicImage, Rgba, RgbaImage};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

pub fn load_font_atlas(path: &str) -> DynamicImage {
    let font_data = Assets::get(path).expect("Failed to load embedded texture");
    let font = Font::from_bytes(font_data.data.as_ref(), fontdue::FontSettings::default()).unwrap();

    let size = 48.0;
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890-+_:;.!?[]".chars().collect();

    let mut glyphs = Vec::new();
    let mut atlas_width = 0;
    let mut atlas_height = 0;

    for c in &chars {
        let (metrics, bitmap) = font.rasterize(*c, size);
        glyphs.push((c, metrics, bitmap));

        atlas_width += metrics.width; // side-by-side packing
        atlas_height = atlas_height.max(metrics.height);
    }

    let mut atlas = RgbaImage::new(atlas_width as u32, atlas_height as u32);

    let mut x_offset = 0;
    for (_, metrics, bitmap) in glyphs {
        let glyph_width = metrics.width as u32;
        let glyph_height = metrics.height as u32;

        let y_offset = atlas_height as u32 - glyph_height;

        for y in 0..glyph_height {
            for x in 0..glyph_width {
                let alpha = bitmap[(y * glyph_width + x) as usize];
                atlas.put_pixel(
                    x_offset + x,
                    y_offset + y,
                    Rgba([255, 255, 255, alpha]),
                );
            }
        }

        x_offset += glyph_width;
    }

    println!("Created Font!");
    DynamicImage::ImageRgba8(atlas)
}

pub fn load_font_uvs(path: &str) -> HashMap<String, (f32, f32, f32, f32, f32)> {
    let font_data = Assets::get(path).expect("Failed to load embedded texture");
    let font = Font::from_bytes(font_data.data.as_ref(), fontdue::FontSettings::default()).unwrap();

    let size = 48.0;
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890-+_:;.!?[]".chars().collect();

    let mut glyphs = Vec::new();
    let mut atlas_width = 0;
    let mut atlas_height = 0;

    for c in &chars {
        let (metrics, _) = font.rasterize(*c, size);
        glyphs.push((c, metrics));

        atlas_width += metrics.width; // side-by-side packing
        atlas_height = atlas_height.max(metrics.height);
    }

    let mut mapped_characters: HashMap<String, (f32, f32, f32, f32, f32)> = HashMap::new();

    let mut x_offset = 0;
    for (c, metrics) in glyphs {
        let glyph_width = metrics.width as u32;

        mapped_characters.insert(c.to_string(), (
            x_offset as f32 / atlas_width as f32,
            0.0,
            (x_offset + glyph_width) as f32 / atlas_width as f32,
            1.0,
            (metrics.ymin as f32 / atlas_height as f32) * 2.0)
        );

        x_offset += glyph_width;
    }

    println!("Mapped Font!");
    mapped_characters
}
