use std::collections::HashMap;

use crate::renderer::{skinned_vertex::SkinnedVertex, transform::Transform};

pub fn parse(
    path: &str,
    global_transform: Transform,
) -> (
    Vec<(
        Vec<SkinnedVertex>,
        Vec<[i8; 3]>,
        Vec<[f32; 3]>,
        Vec<[f32; 2]>,
        String,
    )>,
    HashMap<i64, (usize, Transform, String, i64, usize)>,
) {
    (Vec::new(), HashMap::new())
}
