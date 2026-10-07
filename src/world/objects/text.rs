use std::collections::HashMap;

pub fn create_plane_with_text(position: (f32, f32, f32), scale: (f32, f32, f32), font_map: &HashMap<String, (f32, f32, f32, f32, f32)>, color: [f32; 3], text: &str) -> Vec<(Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, String)> {
    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut colors: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();

    for (i, character) in text.chars().enumerate() {
        if let Some(character_bounds) = font_map.get(&format!("{}", character)) {
            let from = (character_bounds.0, character_bounds.1);
            let to = (character_bounds.2, character_bounds.3);

            let width: f32 = ((to.0 - from.0) as f32 * font_map.len() as f32) * 0.5;

            vertices.push([( 1.0 * width - i as f32 * 1.25) * scale.0 - position.0, (-1.0 + character_bounds.4 as f32) * scale.1 + position.1, position.2]);
            vertices.push([(-1.0 * width - i as f32 * 1.25) * scale.0 - position.0, (-1.0 + character_bounds.4 as f32) * scale.1 + position.1, position.2]);
            vertices.push([( 1.0 * width - i as f32 * 1.25) * scale.0 - position.0, ( 1.0 + character_bounds.4 as f32) * scale.1 + position.1, position.2]);
            vertices.push([( 1.0 * width - i as f32 * 1.25) * scale.0 - position.0, ( 1.0 + character_bounds.4 as f32) * scale.1 + position.1, position.2]);
            vertices.push([(-1.0 * width - i as f32 * 1.25) * scale.0 - position.0, (-1.0 + character_bounds.4 as f32) * scale.1 + position.1, position.2]);
            vertices.push([(-1.0 * width - i as f32 * 1.25) * scale.0 - position.0, ( 1.0 + character_bounds.4 as f32) * scale.1 + position.1, position.2]);

            uvs.push([from.0, to.1]);
            uvs.push([to.0, to.1]);
            uvs.push([from.0, from.1]);
            uvs.push([from.0, from.1]);
            uvs.push([to.0, to.1]);
            uvs.push([to.0, from.1]);

            normals.push([0.0, 0.0, -1.0]);
            normals.push([0.0, 0.0, -1.0]);
            normals.push([0.0, 0.0, -1.0]);
            normals.push([0.0, 0.0, -1.0]);
            normals.push([0.0, 0.0, -1.0]);
            normals.push([0.0, 0.0, -1.0]);

            colors.push(color);
            colors.push(color);
            colors.push(color);
            colors.push(color);
            colors.push(color);
            colors.push(color);
        }
    }

    return vec![(vertices, normals, colors, uvs, "default".to_string())];
}
