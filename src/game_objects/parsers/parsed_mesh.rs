use std::collections::HashMap;

use crate::renderer::{skinned_vertex::SkinnedVertex, transform::Transform};

pub struct ParsedMesh {
    pub vertices: Vec<(Vec<SkinnedVertex>, String)>,
    pub bones: HashMap<i64, (usize, Transform, String, i64, usize)>,
}
