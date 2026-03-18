// Rancer WGPU Shader
// Simple vertex and fragment shader for rendering strokes

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

struct Uniforms {
    canvas_size: vec2<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // Convert from canvas coordinates to clip space (-1 to 1)
    let pos = vertex.position / uniforms.canvas_size * 2.0 - 1.0;
    output.clip_position = vec4<f32>(pos.x, -pos.y, 0.0, 1.0);
    output.color = vertex.color;
    
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
