use std::collections::HashMap;

use crate::renderer::simple_vertex::SimpleVertex;

pub fn create_plane_with_text(
    position: (f32, f32, f32),
    scale: (f32, f32, f32),
    font_map: &HashMap<String, (f32, f32, f32, f32, f32)>,
    color: [f32; 3],
    text: &str,
) -> Vec<(Vec<SimpleVertex>, String)> {
    let mut vertices: Vec<SimpleVertex> = Vec::new();

    for (i, character) in text.chars().enumerate() {
        if let Some(character_bounds) = font_map.get(&format!("{}", character)) {
            let from = (character_bounds.0, character_bounds.1);
            let to = (character_bounds.2, character_bounds.3);

            let width: f32 = ((to.0 - from.0) as f32 * font_map.len() as f32) * 0.5;

            vertices.push(SimpleVertex {
                position: [
                    (1.0 * width - i as f32 * 1.25) * scale.0 - position.0,
                    (-1.0 + character_bounds.4 as f32) * scale.1 + position.1,
                    position.2,
                ],
                normal: [0.0, 0.0, -1.0],
                color: color,
                uv: [from.0, to.1],
            });
            vertices.push(SimpleVertex {
                position: [
                    (-1.0 * width - i as f32 * 1.25) * scale.0 - position.0,
                    (-1.0 + character_bounds.4 as f32) * scale.1 + position.1,
                    position.2,
                ],
                normal: [0.0, 0.0, -1.0],
                color: color,
                uv: [to.0, to.1],
            });
            vertices.push(SimpleVertex {
                position: [
                    (1.0 * width - i as f32 * 1.25) * scale.0 - position.0,
                    (1.0 + character_bounds.4 as f32) * scale.1 + position.1,
                    position.2,
                ],
                normal: [0.0, 0.0, -1.0],
                color: color,
                uv: [from.0, from.1],
            });
            vertices.push(SimpleVertex {
                position: [
                    (1.0 * width - i as f32 * 1.25) * scale.0 - position.0,
                    (1.0 + character_bounds.4 as f32) * scale.1 + position.1,
                    position.2,
                ],
                normal: [0.0, 0.0, -1.0],
                color: color,
                uv: [from.0, from.1],
            });
            vertices.push(SimpleVertex {
                position: [
                    (-1.0 * width - i as f32 * 1.25) * scale.0 - position.0,
                    (-1.0 + character_bounds.4 as f32) * scale.1 + position.1,
                    position.2,
                ],
                normal: [0.0, 0.0, -1.0],
                color: color,
                uv: [to.0, to.1],
            });
            vertices.push(SimpleVertex {
                position: [
                    (-1.0 * width - i as f32 * 1.25) * scale.0 - position.0,
                    (1.0 + character_bounds.4 as f32) * scale.1 + position.1,
                    position.2,
                ],
                normal: [0.0, 0.0, -1.0],
                color: color,
                uv: [to.0, from.1],
            });
        }
    }

    return vec![(vertices, "default".to_string())];
}
