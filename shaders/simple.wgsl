struct Uniforms {
    view_proj: mat4x4<f32>,
    time: f32,
    bass: f32,
    mid: f32,
    treble: f32,
    beat_strength: f32,
    volume: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.color = model.color;
    out.tex_coords = model.tex_coords;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Simple gradient based on audio
    let intensity = uniforms.bass + uniforms.mid + uniforms.treble;
    let beat_pulse = 1.0 + uniforms.beat_strength * 0.3;

    let color = vec3<f32>(
        0.5 + intensity * 0.5,
        0.3 + uniforms.bass * 0.7,
        0.7 + uniforms.treble * 0.3
    ) * beat_pulse;

    return vec4<f32>(color, 1.0);
}