// Parametric wave shader - audio-reactive mathematical patterns
// Converted from GLSL and enhanced for audio responsiveness

struct Uniforms {
    view_proj: mat4x4<f32>,
    time: f32,

    // Audio analysis data
    sub_bass: f32,
    bass: f32,
    mid: f32,
    treble: f32,
    presence: f32,
    beat_strength: f32,
    estimated_bpm: f32,
    volume: f32,
    spectral_centroid: f32,
    spectral_rolloff: f32,
    pitch_confidence: f32,
    zero_crossing_rate: f32,
    spectral_flux: f32,
    onset_strength: f32,
    dynamic_range: f32,

    // Effect weights
    plasma_weight: f32,
    kaleidoscope_weight: f32,
    tunnel_weight: f32,
    particle_weight: f32,
    fractal_weight: f32,
    spectralizer_weight: f32,

    // Visual controls
    projection_mode: f32,
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

// Audio-reactive parameters derived from analysis
struct AudioParams {
    color1: vec3<f32>,      // Primary color from palette
    color2: vec3<f32>,      // Secondary color from palette
    frequency: f32,         // Pattern frequency from spectral data
    speed: f32,             // Animation speed from BPM
    intensity: f32,         // Overall intensity from volume/beat
}

// Extract audio-reactive parameters
fn get_audio_params() -> AudioParams {
    var params: AudioParams;

    // Dynamic color selection based on frequency content
    let bass_dominance = uniforms.bass + uniforms.sub_bass;
    let mid_dominance = uniforms.mid;
    let treble_dominance = uniforms.treble + uniforms.presence;

    // Primary color shifts based on dominant frequency range
    if (bass_dominance > mid_dominance && bass_dominance > treble_dominance) {
        // Bass-heavy: warm colors (reds, oranges)
        params.color1 = vec3<f32>(1.0, 0.3 + bass_dominance * 0.7, 0.1);
    } else if (treble_dominance > bass_dominance && treble_dominance > mid_dominance) {
        // Treble-heavy: cool colors (blues, cyans)
        params.color1 = vec3<f32>(0.1, 0.3 + treble_dominance * 0.7, 1.0);
    } else {
        // Mid-heavy: green/yellow spectrum
        params.color1 = vec3<f32>(0.3 + mid_dominance * 0.7, 1.0, 0.2);
    }

    // Secondary color based on pitch confidence and spectral characteristics
    let harmonic_factor = uniforms.pitch_confidence;
    params.color2 = vec3<f32>(
        0.5 + harmonic_factor * 0.5,
        0.2 + uniforms.spectral_flux * 2.0,
        0.8 - uniforms.zero_crossing_rate * 0.6
    );

    // Pattern frequency driven by spectral centroid (brightness)
    params.frequency = 2.0 + uniforms.spectral_centroid * 0.0001 + uniforms.onset_strength * 5.0;

    // Animation speed synchronized to BPM
    let bpm_factor = uniforms.estimated_bpm / 120.0; // Normalize around 120 BPM
    params.speed = 1.0 + bpm_factor * 2.0 + uniforms.beat_strength * 3.0;

    // Intensity responds to volume and dynamic range
    params.intensity = 0.3 + uniforms.volume * 0.7 + uniforms.dynamic_range * 0.5;

    return params;
}

// Enhanced pattern generation with audio responsiveness
fn generate_pattern(uv: vec2<f32>, params: AudioParams) -> f32 {
    let angle = atan2(uv.y, uv.x);
    let radius = length(uv);

    // Multiple wave patterns that respond to different audio features
    let wave1 = sin(radius * params.frequency - uniforms.time * params.speed);
    let wave2 = cos(angle * (4.0 + uniforms.bass * 8.0) + uniforms.time * params.speed * 0.7);
    let wave3 = sin(length(uv * (2.0 + uniforms.treble * 3.0)) * 3.0 - uniforms.time * params.speed * 1.3);

    // Beat-driven pulse waves
    let beat_pulse = sin(uniforms.time * params.speed * 4.0) * uniforms.beat_strength;
    let wave4 = cos(radius * 8.0 + beat_pulse * 10.0);

    // Spectral flux creates texture variation
    let texture_noise = sin(uv.x * 20.0 + uniforms.spectral_flux * 50.0) *
                       cos(uv.y * 15.0 + uniforms.spectral_flux * 30.0) * 0.1;

    // Combine all patterns with intensity control
    return (wave1 + wave2 + wave3 + wave4 + texture_noise) * params.intensity;
}

// Advanced color blending with audio-reactive chromatic effects
fn apply_audio_chromatic_effects(color: vec3<f32>, pattern: f32) -> vec3<f32> {
    var enhanced_color = color;

    // Beat-driven chromatic aberration
    enhanced_color.r = enhanced_color.r + sin(uniforms.time * 0.5 + uniforms.beat_strength * 5.0) * 0.1;
    enhanced_color.b = enhanced_color.b + cos(uniforms.time * 0.3 + uniforms.beat_strength * 3.0) * 0.1;

    // Onset strength creates sudden color shifts
    let onset_shift = uniforms.onset_strength * sin(pattern * 3.0) * 0.2;
    enhanced_color.g = enhanced_color.g + onset_shift;

    // Zero crossing rate affects color saturation
    let saturation_factor = 1.0 + uniforms.zero_crossing_rate * 0.5;
    enhanced_color = enhanced_color * saturation_factor;

    return enhanced_color;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Normalize coordinates to screen center
    let resolution = vec2<f32>(1200.0, 800.0); // TODO: Make this dynamic
    let uv = (in.world_pos - 0.5 * resolution) / min(resolution.y, resolution.x);

    // Get audio-reactive parameters
    let params = get_audio_params();

    // Generate complex pattern
    let pattern = generate_pattern(uv, params);

    // Create dynamic color palette based on pattern and audio
    var color = vec3<f32>(
        params.color1.r * (0.5 + 0.5 * cos(pattern)),
        params.color1.g * (0.5 + 0.5 * sin(pattern * 1.5)),
        params.color2.b * (0.5 + 0.5 * cos(pattern * 2.0))
    );

    // Apply audio-driven chromatic effects
    color = apply_audio_chromatic_effects(color, pattern);

    // Radial gradient with bass-responsive falloff
    let radius = length(uv);
    let gradient_power = 0.7 + uniforms.bass * 0.5; // Bass extends the gradient
    let gradient = 1.0 - pow(radius, gradient_power);
    color = color * gradient;

    // Dynamic range affects overall brightness
    let brightness_factor = 0.8 + uniforms.dynamic_range * 0.4;
    color = color * brightness_factor;

    // Ensure color values stay in valid range
    color = clamp(color, vec3<f32>(0.0), vec3<f32>(1.0));

    return vec4<f32>(color, 1.0);
}