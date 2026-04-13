// Rancer WGPU Shader
// Vertex and fragment shader for rendering strokes with line width support

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
    zoom: f32,
    pan_offset: vec2<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // Apply pan and zoom transformation
    let world_pos = (vertex.position - uniforms.pan_offset) * uniforms.zoom;
    
    // Convert from canvas coordinates to clip space (-1 to 1)
    let pos = world_pos / uniforms.canvas_size * 2.0 - 1.0;
    output.clip_position = vec4<f32>(pos.x, -pos.y, 0.0, 1.0);
    output.color = vertex.color;
    
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}

// ─── Textured Quad Shader ─────────────────────────────────────────────────

struct TexturedVertexInput {
    @location(0) position: vec2<f32>,
    @location(1) texcoord: vec2<f32>,
};

struct TexturedVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texcoord: vec2<f32>,
};

struct TexturedUniforms {
    canvas_size: vec2<f32>,
    zoom: f32,
    pan_offset: vec2<f32>,
    image_size: vec2<f32>,
    image_offset: vec2<f32>,
};

@group(0) @binding(0) var<uniform> textured_uniforms: TexturedUniforms;
@group(0) @binding(1) var textureSampler: sampler;
@group(0) @binding(2) var inputTexture: texture_2d<f32>;

@vertex
fn vs_textured(vertex: TexturedVertexInput) -> TexturedVertexOutput {
    var output: TexturedVertexOutput;
    
    // Apply layer offset, then pan and zoom transformation
    let world_pos = (vertex.position + textured_uniforms.image_offset - textured_uniforms.pan_offset) * textured_uniforms.zoom;
    
    // Convert from canvas coordinates to clip space (-1 to 1)
    let pos = world_pos / textured_uniforms.canvas_size * 2.0 - 1.0;
    output.clip_position = vec4<f32>(pos.x, -pos.y, 0.0, 1.0);
    output.texcoord = vertex.texcoord;
    
    return output;
}

@fragment
fn fs_textured(input: TexturedVertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(inputTexture, textureSampler, input.texcoord);
    return tex_color;
}
