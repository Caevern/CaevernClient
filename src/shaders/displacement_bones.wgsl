struct Uniforms {
    model_mat : mat4x4<f32>,
    view_project_mat : mat4x4<f32>,
    normal_mat : mat4x4<f32>,
};

struct Model {
    model_mat : mat4x4<f32>,
    normal_mat : mat4x4<f32>
};

struct Bones {
    matrices: array<mat4x4<f32>>,
};

@binding(0) @group(0) var<uniform> uniforms : Uniforms;
@binding(4) @group(0) var displacement_tex: texture_2d<f32>;
@binding(5) @group(0) var displacement_sampler: sampler;
@binding(6) @group(0) var<uniform> model : Model;
@binding(7) @group(0) var<storage, read> bones: Bones;

struct Output {
    @builtin(position) position : vec4<f32>,
    @location(0) v_position : vec4<f32>,
    @location(1) v_normal : vec4<f32>,
    @location(2) v_color : vec4<f32>,
    @location(3) v_uv : vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) pos: vec4<f32>, @location(1) normal: vec4<f32>,
    @location(2) color: vec4<f32>, @location(3) uv: vec4<f32>,
    @location(4) bone_index: vec4<f32>, @location(5) bone_weight: vec4<f32>
    ) -> Output {
    var output: Output;

    // Sample displacement map
    let displacement: f32 = textureSampleLevel(displacement_tex, displacement_sampler, uv.xy, 0.0).r;
    let strength: f32 = 0.5; // scale factor for displacement

    // Apply displacement along vertex normal
    let displaced_pos: vec4<f32> = pos + vec4(normalize(normal.xyz) * displacement * strength, 0.0);

    let skin_mat =
        bone_weight.x * bones.matrices[u32(bone_index.x)]
        + bone_weight.y * bones.matrices[u32(bone_index.y)]
        + bone_weight.z * bones.matrices[u32(bone_index.z)]
        + bone_weight.w * bones.matrices[u32(bone_index.w)];
    let skinned_pos = skin_mat * displaced_pos;

    let m_position:vec4<f32> = model.model_mat * skinned_pos;
    output.position = uniforms.view_project_mat * m_position;
    output.v_position = m_position;
    output.v_normal =  model.normal_mat * normal;
    output.v_color = color;
    output.v_uv = uv;
    return output;
}

struct FragUniforms {
    light_position : vec4<f32>,
    eye_position : vec4<f32>,
};
@binding(1) @group(0) var<uniform> frag_uniforms : FragUniforms;

struct LightUniforms {
    color : vec4<f32>,
    specular_color : vec4<f32>,
    ambient_intensity: f32,
    diffuse_intensity :f32,
    specular_intensity: f32,
    specular_shininess: f32,
};

@binding(2) @group(0) var texture: texture_2d<f32>;
@binding(3) @group(0) var texture_sampler: sampler;

@fragment
fn fs_main(@location(0) v_position: vec4<f32>, @location(1) v_normal: vec4<f32>, @location(2) v_color: vec4<f32>, @location(3) v_uv: vec4<f32>) ->  @location(0) vec4<f32> {
    let N:vec3<f32> = normalize(v_normal.xyz);
    //let L:vec3<f32> = normalize(frag_uniforms.light_position.xyz - v_position.xyz);
    let L:vec3<f32> = normalize(vec3(0.5, 1.0, 0.5));
    let V:vec3<f32> = normalize(frag_uniforms.eye_position.xyz - v_position.xyz);
    let H:vec3<f32> = normalize(L + V);
    let texture_color: vec4<f32> = textureSample(texture, texture_sampler, v_uv.xy);

    let fog_color: vec3<f32> = vec3(0.2, 0.247, 0.314);
    let fog_start: f32 = 150.0;
    let fog_end: f32 = 200.0;
    let distance: f32 = length(v_position.xyz - frag_uniforms.eye_position.xyz);
    let fog_factor: f32 = clamp((fog_end - distance) / (fog_end - fog_start), 0.0, 1.0);

    let rgb_effect: vec3<f32> = texture_color.rgb * v_color.rgb;
    let alpha:f32 = texture_color.a * v_color.a;
    let final_color: vec4<f32> = vec4(rgb_effect, alpha);

    let blended_color: vec3<f32> = mix(fog_color, final_color.rgb, fog_factor);

    return vec4(blended_color, final_color.a);
}