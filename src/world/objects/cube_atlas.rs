pub fn create_cube(
    position: (f64, f64, f64),
    scale: (f64, f64, f64),
) -> (Vec<[f64; 3]>, Vec<[f32; 2]>, Vec<[i8; 3]>, Vec<[f32; 3]>) {
    let mut vertices: Vec<[f64; 3]> = Vec::new();
    let mut normals: Vec<[i8; 3]> = Vec::new();
    let mut colors: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();

    let atlas_width = 8 as f32;
    let atlas_height = 8 as f32;

    let atlas_offset = vec![
        (0.0, 0.0),
        (0.0, 0.0),
        (0.0, 0.0),
        (0.0, 0.0),
        (0.0, 0.0),
        (0.0, 0.0),
    ];

    vertices.push([
        1.0 * scale.0 - position.0,
        -1.0 * scale.1 + position.1,
        1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        1.0 * scale.0 - position.0,
        -1.0 * scale.1 + position.1,
        -1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        1.0 * scale.0 - position.0,
        1.0 * scale.1 + position.1,
        1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        1.0 * scale.0 - position.0,
        1.0 * scale.1 + position.1,
        1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        1.0 * scale.0 - position.0,
        -1.0 * scale.1 + position.1,
        -1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        1.0 * scale.0 - position.0,
        1.0 * scale.1 + position.1,
        -1.0 * scale.2 + position.2,
    ]);

    let uv_x = (atlas_offset[0].0 as f32 % atlas_width).floor();
    let uv_y = (atlas_offset[0].1 as f32 / atlas_height).floor();
    uvs.push([
        0.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        1.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        1.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        1.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        0.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        0.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        0.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        0.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        1.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        1.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        1.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        0.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);

    normals.push([1, 0, 0]);
    normals.push([1, 0, 0]);
    normals.push([1, 0, 0]);
    normals.push([1, 0, 0]);
    normals.push([1, 0, 0]);
    normals.push([1, 0, 0]);

    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);

    vertices.push([
        -1.0 * scale.0 - position.0,
        -1.0 * scale.1 + position.1,
        -1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        -1.0 * scale.0 - position.0,
        -1.0 * scale.1 + position.1,
        1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        -1.0 * scale.0 - position.0,
        1.0 * scale.1 + position.1,
        -1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        -1.0 * scale.0 - position.0,
        1.0 * scale.1 + position.1,
        -1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        -1.0 * scale.0 - position.0,
        -1.0 * scale.1 + position.1,
        1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        -1.0 * scale.0 - position.0,
        1.0 * scale.1 + position.1,
        1.0 * scale.2 + position.2,
    ]);

    let uv_x = (atlas_offset[1].0 as f32 % atlas_width).floor();
    let uv_y = (atlas_offset[1].1 as f32 / atlas_height).floor();
    uvs.push([
        0.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        1.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        1.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        1.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        0.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        0.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        0.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        0.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        1.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        1.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        1.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        0.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);

    normals.push([-1, 0, 0]);
    normals.push([-1, 0, 0]);
    normals.push([-1, 0, 0]);
    normals.push([-1, 0, 0]);
    normals.push([-1, 0, 0]);
    normals.push([-1, 0, 0]);

    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);

    vertices.push([
        -1.0 * scale.0 - position.0,
        1.0 * scale.1 + position.1,
        1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        1.0 * scale.0 - position.0,
        1.0 * scale.1 + position.1,
        1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        -1.0 * scale.0 - position.0,
        1.0 * scale.1 + position.1,
        -1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        -1.0 * scale.0 - position.0,
        1.0 * scale.1 + position.1,
        -1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        1.0 * scale.0 - position.0,
        1.0 * scale.1 + position.1,
        1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        1.0 * scale.0 - position.0,
        1.0 * scale.1 + position.1,
        -1.0 * scale.2 + position.2,
    ]);

    let uv_x = (atlas_offset[2].0 as f32 % atlas_width).floor();
    let uv_y = (atlas_offset[2].1 as f32 / atlas_height).floor();
    uvs.push([
        0.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        1.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        1.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        1.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        0.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        0.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        0.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        0.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        1.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        1.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        1.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        0.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);

    normals.push([0, 1, 0]);
    normals.push([0, 1, 0]);
    normals.push([0, 1, 0]);
    normals.push([0, 1, 0]);
    normals.push([0, 1, 0]);
    normals.push([0, 1, 0]);

    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);

    vertices.push([
        -1.0 * scale.0 - position.0,
        -1.0 * scale.1 + position.1,
        -1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        1.0 * scale.0 - position.0,
        -1.0 * scale.1 + position.1,
        -1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        -1.0 * scale.0 - position.0,
        -1.0 * scale.1 + position.1,
        1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        -1.0 * scale.0 - position.0,
        -1.0 * scale.1 + position.1,
        1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        1.0 * scale.0 - position.0,
        -1.0 * scale.1 + position.1,
        -1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        1.0 * scale.0 - position.0,
        -1.0 * scale.1 + position.1,
        1.0 * scale.2 + position.2,
    ]);

    let uv_x = (atlas_offset[3].0 as f32 % atlas_width).floor();
    let uv_y = (atlas_offset[3].1 as f32 / atlas_height).floor();
    uvs.push([
        0.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        1.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        1.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        1.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        0.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        0.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        0.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        0.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        1.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        1.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        1.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        0.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);

    normals.push([0, -1, 0]);
    normals.push([0, -1, 0]);
    normals.push([0, -1, 0]);
    normals.push([0, -1, 0]);
    normals.push([0, -1, 0]);
    normals.push([0, -1, 0]);

    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);

    vertices.push([
        -1.0 * scale.0 - position.0,
        -1.0 * scale.1 + position.1,
        1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        1.0 * scale.0 - position.0,
        -1.0 * scale.1 + position.1,
        1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        -1.0 * scale.0 - position.0,
        1.0 * scale.1 + position.1,
        1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        -1.0 * scale.0 - position.0,
        1.0 * scale.1 + position.1,
        1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        1.0 * scale.0 - position.0,
        -1.0 * scale.1 + position.1,
        1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        1.0 * scale.0 - position.0,
        1.0 * scale.1 + position.1,
        1.0 * scale.2 + position.2,
    ]);

    let uv_x = (atlas_offset[4].0 as f32 % atlas_width).floor();
    let uv_y = (atlas_offset[4].1 as f32 / atlas_height).floor();
    uvs.push([
        0.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        1.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        1.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        1.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        0.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        0.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        0.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        0.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        1.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        1.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        1.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        0.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);

    normals.push([0, 0, 1]);
    normals.push([0, 0, 1]);
    normals.push([0, 0, 1]);
    normals.push([0, 0, 1]);
    normals.push([0, 0, 1]);
    normals.push([0, 0, 1]);

    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);

    vertices.push([
        1.0 * scale.0 - position.0,
        -1.0 * scale.1 + position.1,
        -1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        -1.0 * scale.0 - position.0,
        -1.0 * scale.1 + position.1,
        -1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        1.0 * scale.0 - position.0,
        1.0 * scale.1 + position.1,
        -1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        1.0 * scale.0 - position.0,
        1.0 * scale.1 + position.1,
        -1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        -1.0 * scale.0 - position.0,
        -1.0 * scale.1 + position.1,
        -1.0 * scale.2 + position.2,
    ]);
    vertices.push([
        -1.0 * scale.0 - position.0,
        1.0 * scale.1 + position.1,
        -1.0 * scale.2 + position.2,
    ]);

    let uv_x = (atlas_offset[5].0 as f32 % atlas_width).floor();
    let uv_y = (atlas_offset[5].1 as f32 / atlas_height).floor();
    uvs.push([
        0.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        1.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        1.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        1.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        0.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        0.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        0.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        0.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        1.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        1.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);
    uvs.push([
        1.0 / atlas_width + 1.0 / atlas_width * (uv_x),
        0.0 / atlas_height + 1.0 / atlas_height * (uv_y),
    ]);

    normals.push([0, 0, -1]);
    normals.push([0, 0, -1]);
    normals.push([0, 0, -1]);
    normals.push([0, 0, -1]);
    normals.push([0, 0, -1]);
    normals.push([0, 0, -1]);

    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);
    colors.push([1.0, 1.0, 1.0]);

    return (vertices, uvs, normals, colors);
}
