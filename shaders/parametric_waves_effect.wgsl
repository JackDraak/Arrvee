// Parametric Waves Effect - Standalone shader for mathematical audio-reactive patterns
// This is a modular effect shader that can be loaded independently

struct Uniforms {
    view_proj: mat4x4<f32>,
    time: f32,

    // Frequency bands (5-band analysis)
    sub_bass: f32,
    bass: f32,
    mid: f32,
    treble: f32,
    presence: f32,

    // Beat and rhythm
    beat_strength: f32,
    estimated_bpm: f32,
    volume: f32,

    // Spectral characteristics
    spectral_centroid: f32,
    spectral_rolloff: f32,
    pitch_confidence: f32,

    // Temporal dynamics
    zero_crossing_rate: f32,
    spectral_flux: f32,
    onset_strength: f32,
    dynamic_range: f32,

    // Effect weights for dynamic blending
    plasma_weight: f32,
    kaleidoscope_weight: f32,
    tunnel_weight: f32,
    particle_weight: f32,
    fractal_weight: f32,
    spectralizer_weight: f32,

    // 3D projection controls
    projection_mode: f32,

    // Visual controls
    palette_index: f32,
    smoothing_factor: f32,

    _padding: vec3<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

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
    out.clip_position = uniforms.view_proj * vec4<f32>(model.position, 1.0);
    out.color = model.color;
    out.tex_coords = model.tex_coords;
    out.world_pos = model.position.xy;
    return out;
}

// Color palette function (reused from main shader)
fn get_palette_color(palette_id: i32, t: f32) -> vec3<f32> {
    let wrapped_t = fract(t);

    switch palette_id {
        case 0: { // Rainbow
            return vec3<f32>(
                0.5 + 0.5 * cos(6.28318 * (wrapped_t + 0.0)),
                0.5 + 0.5 * cos(6.28318 * (wrapped_t + 0.33)),
                0.5 + 0.5 * cos(6.28318 * (wrapped_t + 0.67))
            );
        }
        case 1: { // Neon Cyber
            return vec3<f32>(
                0.2 + 0.8 * cos(6.28318 * (wrapped_t + 0.0)),
                0.1 + 0.9 * cos(6.28318 * (wrapped_t + 0.15)),
                0.8 + 0.2 * cos(6.28318 * (wrapped_t + 0.9))
            );
        }
        case 2: { // Warm Sunset
            return vec3<f32>(
                0.8 + 0.2 * cos(6.28318 * (wrapped_t + 0.0)),
                0.3 + 0.4 * cos(6.28318 * (wrapped_t + 0.1)),
                0.1 + 0.3 * cos(6.28318 * (wrapped_t + 0.8))
            );
        }
        case 3: { // Deep Ocean
            return vec3<f32>(
                0.1 + 0.3 * cos(6.28318 * (wrapped_t + 0.7)),
                0.2 + 0.5 * cos(6.28318 * (wrapped_t + 0.4)),
                0.6 + 0.4 * cos(6.28318 * (wrapped_t + 0.0))
            );
        }
        case 4: { // Purple Haze
            return vec3<f32>(
                0.5 + 0.5 * cos(6.28318 * (wrapped_t + 0.8)),
                0.2 + 0.3 * cos(6.28318 * (wrapped_t + 0.5)),
                0.7 + 0.3 * cos(6.28318 * (wrapped_t + 0.0))
            );
        }
        default: { // Electric Green
            return vec3<f32>(
                0.2 + 0.4 * cos(6.28318 * (wrapped_t + 0.6)),
                0.7 + 0.3 * cos(6.28318 * (wrapped_t + 0.0)),
                0.1 + 0.4 * cos(6.28318 * (wrapped_t + 0.3))
            );
        }
    }
}

fn get_current_palette_color(t: f32) -> vec3<f32> {
    return get_palette_color(i32(uniforms.palette_index), t);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Get screen-space UV coordinates
    let uv = (in.tex_coords - 0.5) * 2.0;

    // Audio-reactive parameters
    let bass_energy = uniforms.bass + uniforms.sub_bass;
    let treble_energy = uniforms.treble + uniforms.presence;

    // Pattern frequency driven by spectral centroid and onset strength
    let base_frequency = 2.0 + uniforms.spectral_centroid * 0.0001;
    let onset_boost = uniforms.onset_strength * 5.0;
    let frequency = base_frequency + onset_boost;

    // Animation speed synchronized to BPM with beat emphasis
    let bpm_factor = uniforms.estimated_bpm / 120.0;
    let speed = 1.0 + bpm_factor * 2.0 + uniforms.beat_strength * 3.0;

    // Overall intensity from volume and dynamic range
    let intensity = 0.3 + uniforms.volume * 0.7 + uniforms.dynamic_range * 0.5;

    // Generate mathematical patterns
    let angle = atan2(uv.y, uv.x);
    let radius = length(uv);

    // Multiple wave patterns responding to different audio features
    let wave1 = sin(radius * frequency - uniforms.time * speed);
    let wave2 = cos(angle * (4.0 + bass_energy * 8.0) + uniforms.time * speed * 0.7);
    let wave3 = sin(length(uv * (2.0 + treble_energy * 3.0)) * 3.0 - uniforms.time * speed * 1.3);

    // Beat-driven pulse wave
    let beat_pulse = sin(uniforms.time * speed * 4.0) * uniforms.beat_strength;
    let wave4 = cos(radius * 8.0 + beat_pulse * 10.0);

    // Spectral flux creates texture variation
    let texture_noise = sin(uv.x * 20.0 + uniforms.spectral_flux * 50.0) *
                       cos(uv.y * 15.0 + uniforms.spectral_flux * 30.0) * 0.1;

    // Combine patterns
    let pattern = (wave1 + wave2 + wave3 + wave4 + texture_noise) * intensity;

    // Color generation using current palette
    let color_t1 = (pattern + 1.0) * 0.5; // Normalize to 0-1
    let color_t2 = (sin(pattern * 1.5) + 1.0) * 0.5;
    let color_t3 = (cos(pattern * 2.0) + 1.0) * 0.5;

    let color1 = get_current_palette_color(color_t1);
    let color2 = get_current_palette_color(color_t2 + 0.33);
    let color3 = get_current_palette_color(color_t3 + 0.67);

    // Mix colors based on pattern values
    var final_color = color1 * (0.5 + 0.5 * cos(pattern));
    final_color = final_color + color2 * (0.3 + 0.3 * sin(pattern * 1.5));
    final_color = final_color + color3 * (0.2 + 0.2 * cos(pattern * 2.0));
    final_color = final_color / 3.0; // Normalize after mixing

    // Beat-driven chromatic aberration
    final_color.r = final_color.r + sin(uniforms.time * 0.5 + uniforms.beat_strength * 5.0) * 0.1;
    final_color.b = final_color.b + cos(uniforms.time * 0.3 + uniforms.beat_strength * 3.0) * 0.1;

    // Onset strength creates sudden color shifts
    let onset_shift = uniforms.onset_strength * sin(pattern * 3.0) * 0.2;
    final_color.g = final_color.g + onset_shift;

    // Zero crossing rate affects color saturation
    let saturation_factor = 1.0 + uniforms.zero_crossing_rate * 0.5;
    final_color = final_color * saturation_factor;

    // Radial gradient with bass-responsive falloff
    let gradient_power = 0.7 + bass_energy * 0.5;
    let gradient = 1.0 - pow(radius, gradient_power);
    final_color = final_color * gradient;

    // Dynamic range affects overall brightness
    let brightness_factor = 0.8 + uniforms.dynamic_range * 0.4;
    final_color = final_color * brightness_factor;

    // Ensure color values stay in valid range
    final_color = clamp(final_color, vec3<f32>(0.0), vec3<f32>(1.0));

    return vec4<f32>(final_color, 1.0);
}