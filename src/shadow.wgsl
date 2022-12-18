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

@group(0) @binding(0)
var<uniform> camera : mat4x4<f32>;

@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> @builtin(position) vec4<f32>{

    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    
    return camera * model_matrix * vec4<f32>(model.vertex_position, 1.0);
}