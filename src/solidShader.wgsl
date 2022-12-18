// vertex shader
struct VertexInput{
    @location(0) vertex_position: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
}

struct InstanceInput{
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
}

struct VertexOutput{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) shadowPos: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> camera : mat4x4<f32>;
@group(0) @binding(1)
var<uniform> light : mat4x4<f32>;

@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput{
    var output:VertexOutput;
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    let pos = vec4<f32>(model.vertex_position, 1.0);
    let pos_from_light = light * model_matrix * pos;

    output.clip_position = camera * model_matrix * pos;
    output.shadowPos = vec3<f32>(pos_from_light.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5), pos_from_light.z);
    return output;
}

@group(1) @binding(0)
var t_depth: texture_depth_2d;
@group(1) @binding(1)
var s_depth: sampler_comparison;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32>{
    var shadow = textureSampleCompare(
        t_depth,
        s_depth,
        in.shadowPos.xy,
        in.shadowPos.z - 0.005
    );
    let light_factor = min(0.3 + shadow * 1.0, 1.0);
    let color = light_factor * vec3<f32>(0.8,0.8,0.8);
    return vec4<f32>(color, 1.0);
}