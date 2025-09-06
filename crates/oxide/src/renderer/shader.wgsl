struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct CameraUniform {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
};

struct MeshTransform {
    model: mat4x4<f32>,
};

struct MaterialProperties {
    base_color_factor: vec4<f32>,
    metallic_factor: f32,
    roughness_factor: f32,
    // Padding to meet alignment requirements
    _padding: vec2<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> transform: MeshTransform;
@group(2) @binding(0) var base_color_texture: texture_2d<f32>;
@group(2) @binding(1) var base_color_sampler: sampler;
@group(2) @binding(2) var<uniform> material_props: MaterialProperties; // ← ADD THIS LINE

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;

    let world_position = transform.model * vec4<f32>(model.position, 1.0);
    out.clip_position = camera.view_proj * world_position;
    out.tex_coords = model.tex_coords;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample the texture at the correct UV coordinates
    let texture_color = textureSample(base_color_texture, base_color_sampler, in.tex_coords);

    // Apply any material color tint
    let final_color = texture_color * material_props.base_color_factor;

    return final_color;
}