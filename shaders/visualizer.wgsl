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
    @location(2) world_pos: vec2<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.color = model.color;
    out.tex_coords = model.tex_coords;
    out.world_pos = model.position.xy;
    return out;
}

fn hue_to_rgb(hue: f32) -> vec3<f32> {
    let h = (hue % 1.0) * 6.0;
    let x = 1.0 - abs((h % 2.0) - 1.0);

    if (h < 1.0) {
        return vec3<f32>(1.0, x, 0.0);
    } else if (h < 2.0) {
        return vec3<f32>(x, 1.0, 0.0);
    } else if (h < 3.0) {
        return vec3<f32>(0.0, 1.0, x);
    } else if (h < 4.0) {
        return vec3<f32>(0.0, x, 1.0);
    } else if (h < 5.0) {
        return vec3<f32>(x, 0.0, 1.0);
    } else {
        return vec3<f32>(1.0, 0.0, x);
    }
}

fn plasma_effect(pos: vec2<f32>, time: f32, audio_influence: f32) -> vec3<f32> {
    let speed = 0.5 + audio_influence * 2.0;
    let scale = 3.0 + uniforms.bass * 5.0;

    let wave1 = sin(pos.x * scale + time * speed);
    let wave2 = sin(pos.y * scale + time * speed * 0.8);
    let wave3 = sin((pos.x + pos.y) * scale * 0.7 + time * speed * 1.2);
    let wave4 = sin(sqrt(pos.x * pos.x + pos.y * pos.y) * scale * 0.5 + time * speed * 0.6);

    let combined = (wave1 + wave2 + wave3 + wave4) * 0.25;

    let hue = combined * 0.5 + 0.5 + uniforms.mid * 0.3;
    let brightness = 0.5 + abs(combined) * 0.5 + uniforms.treble * 0.4;

    return hue_to_rgb(hue) * brightness;
}

fn waveform_bars(pos: vec2<f32>) -> f32 {
    let bar_count = 64.0;
    let bar_width = 2.0 / bar_count;
    let bar_index = floor((pos.x + 1.0) / bar_width);
    let bar_center = (bar_index + 0.5) * bar_width - 1.0;

    if (abs(pos.x - bar_center) < bar_width * 0.4) {
        let frequency_index = bar_index / bar_count;
        let height = 0.3 + uniforms.bass * frequency_index + uniforms.mid * (1.0 - frequency_index);

        if (abs(pos.y) < height) {
            return 1.0;
        }
    }

    return 0.0;
}

fn radial_pattern(pos: vec2<f32>, time: f32) -> f32 {
    let distance = length(pos);
    let angle = atan2(pos.y, pos.x);

    let wave_count = 8.0 + uniforms.treble * 16.0;
    let radial_wave = sin(angle * wave_count + time * 2.0 + distance * 10.0);
    let distance_wave = sin(distance * 20.0 - time * 3.0);

    let intensity = (radial_wave * distance_wave + 1.0) * 0.5;
    return intensity * (1.0 - smoothstep(0.0, 1.0, distance));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pos = in.world_pos;
    let time = uniforms.time;

    let audio_influence = uniforms.bass + uniforms.mid + uniforms.treble;
    let beat_pulse = 1.0 + uniforms.beat_strength * 0.5;

    let plasma = plasma_effect(pos * beat_pulse, time, audio_influence);
    let bars = waveform_bars(pos);
    let radial = radial_pattern(pos, time);

    let base_color = plasma * (0.7 + uniforms.volume * 0.3);
    let accent_color = vec3<f32>(1.0, 0.5, 0.0) * bars * 0.8;
    let radial_color = vec3<f32>(0.2, 0.8, 1.0) * radial * 0.4;

    let final_color = base_color + accent_color + radial_color;

    let brightness = 0.8 + uniforms.volume * 0.2 + uniforms.beat_strength * 0.3;

    return vec4<f32>(final_color * brightness, 1.0);
}