// Minter-Inspired Psychedelic Effects Collection
// A collection of trippy, modular shader effects that respond to rich musical analysis

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
    spectral_centroid: f32,    // Brightness
    spectral_rolloff: f32,     // High frequency content
    pitch_confidence: f32,     // Harmonic vs percussive

    // Temporal dynamics
    zero_crossing_rate: f32,   // Texture/noisiness
    spectral_flux: f32,        // Rate of change
    onset_strength: f32,       // Note attacks
    dynamic_range: f32,        // Volume variation

    // Effect weights for dynamic blending
    plasma_weight: f32,
    kaleidoscope_weight: f32,
    tunnel_weight: f32,
    particle_weight: f32,
    fractal_weight: f32,
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

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> vec3<f32> {
    let c = v * s;
    let x = c * (1.0 - abs(((h * 6.0) % 2.0) - 1.0));
    let m = v - c;

    var rgb: vec3<f32>;
    let h_sector = h * 6.0;

    if (h_sector < 1.0) {
        rgb = vec3<f32>(c, x, 0.0);
    } else if (h_sector < 2.0) {
        rgb = vec3<f32>(x, c, 0.0);
    } else if (h_sector < 3.0) {
        rgb = vec3<f32>(0.0, c, x);
    } else if (h_sector < 4.0) {
        rgb = vec3<f32>(0.0, x, c);
    } else if (h_sector < 5.0) {
        rgb = vec3<f32>(x, 0.0, c);
    } else {
        rgb = vec3<f32>(c, 0.0, x);
    }

    return rgb + vec3<f32>(m);
}

fn noise2d(pos: vec2<f32>) -> f32 {
    return fract(sin(dot(pos.xy, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

fn smooth_noise(pos: vec2<f32>) -> f32 {
    let i = floor(pos);
    let f = fract(pos);

    let a = noise2d(i);
    let b = noise2d(i + vec2<f32>(1.0, 0.0));
    let c = noise2d(i + vec2<f32>(0.0, 1.0));
    let d = noise2d(i + vec2<f32>(1.0, 1.0));

    let u = f * f * (3.0 - 2.0 * f);

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// ============================================================================
// PSYCHEDELIC EFFECTS COLLECTION
// ============================================================================

// Effect 1: Llama Plasma Fields - Driven by frequency bands and spectral flux
fn llama_plasma(pos: vec2<f32>) -> vec3<f32> {
    let time_speed = uniforms.time * (0.5 + uniforms.spectral_flux * 2.0);
    let beat_pulse = 1.0 + uniforms.beat_strength * 0.3;

    // Multi-layered plasma with frequency-driven scaling
    let scale1 = 2.0 + uniforms.sub_bass * 8.0;
    let scale2 = 3.0 + uniforms.bass * 6.0;
    let scale3 = 4.0 + uniforms.mid * 4.0;

    let wave1 = sin(pos.x * scale1 + time_speed) * cos(pos.y * scale1 + time_speed * 0.7);
    let wave2 = sin((pos.x + pos.y) * scale2 * 0.7 + time_speed * 1.3);
    let wave3 = sin(length(pos) * scale3 + time_speed * 0.8);
    let wave4 = cos(atan2(pos.y, pos.x) * 8.0 + time_speed * 2.0) * uniforms.treble;

    let plasma = (wave1 + wave2 + wave3 + wave4) * 0.25 * beat_pulse;

    // Color cycling based on spectral centroid (brightness)
    let hue_base = uniforms.time * 0.1 + uniforms.spectral_centroid * 0.5;
    let hue = hue_base + plasma * 0.3;
    let saturation = 0.7 + uniforms.presence * 0.3;
    let brightness = 0.6 + abs(plasma) * 0.4 + uniforms.volume * 0.2;

    return hsv_to_rgb(hue, saturation, brightness);
}

// Effect 2: Geometric Kaleidoscope - Controlled by BPM and pitch confidence
fn geometric_kaleidoscope(pos: vec2<f32>) -> vec3<f32> {
    let bpm_factor = uniforms.estimated_bpm / 120.0; // Normalize around 120 BPM
    let rotation_speed = uniforms.time * bpm_factor * 0.5;

    // Rotate position based on BPM
    let cos_r = cos(rotation_speed);
    let sin_r = sin(rotation_speed);
    let rotated_pos = vec2<f32>(
        pos.x * cos_r - pos.y * sin_r,
        pos.x * sin_r + pos.y * cos_r
    );

    let distance = length(rotated_pos);
    let angle = atan2(rotated_pos.y, rotated_pos.x);

    // Kaleidoscope segments driven by pitch confidence
    let segments = 6.0 + uniforms.pitch_confidence * 12.0;
    let segment_angle = (3.14159 * 2.0) / segments;
    let folded_angle = abs((angle % segment_angle) - segment_angle * 0.5);

    // Concentric rings modulated by onset strength
    let ring_frequency = 10.0 + uniforms.onset_strength * 20.0;
    let ring_pattern = sin(distance * ring_frequency - uniforms.time * 3.0);

    // Radial spokes
    let spoke_pattern = sin(folded_angle * 20.0 + uniforms.time * 2.0);

    let pattern = (ring_pattern + spoke_pattern) * 0.5;
    let intensity = smoothstep(0.0, 0.3, pattern) * (1.0 - smoothstep(0.8, 1.2, distance));

    // Colors based on dynamic range and zero crossing rate
    let hue = uniforms.zero_crossing_rate * 0.8 + folded_angle * 0.1;
    let saturation = 0.8 + uniforms.dynamic_range * 0.2;

    return hsv_to_rgb(hue, saturation, intensity);
}

// Effect 3: Tunnel Vision - Minter-style tunnel with spectral rolloff control
fn psychedelic_tunnel(pos: vec2<f32>) -> vec3<f32> {
    let distance = length(pos);
    let angle = atan2(pos.y, pos.x);

    // Tunnel depth modulated by spectral rolloff
    let tunnel_depth = 5.0 + uniforms.spectral_rolloff * 10.0;
    let z = tunnel_depth / max(distance, 0.1);

    // Twisted tunnel walls
    let twist = uniforms.time * 2.0 + uniforms.spectral_flux * 5.0;
    let twisted_angle = angle + z * 0.5 + twist;

    // Tunnel stripes
    let stripe_frequency = 20.0 + uniforms.presence * 30.0;
    let stripes = sin(z * stripe_frequency + uniforms.time * 8.0);
    let spiral = sin(twisted_angle * 8.0 + z * 10.0);

    let pattern = (stripes + spiral) * 0.5;
    let tunnel_brightness = smoothstep(-0.3, 0.3, pattern);

    // Beat-driven pulsing
    let beat_pulse = 1.0 + uniforms.beat_strength * 0.5;
    tunnel_brightness *= beat_pulse;

    // Color shift through the tunnel
    let hue = z * 0.1 + uniforms.time * 0.2 + uniforms.mid * 0.3;
    let saturation = 0.9;

    // Distance fade
    let fade = 1.0 - smoothstep(0.0, 1.5, distance);

    return hsv_to_rgb(hue, saturation, tunnel_brightness * fade);
}

// Effect 4: Particle Swarm - Triggered by onset strength and zero crossing rate
fn particle_swarm(pos: vec2<f32>) -> vec3<f32> {
    var color = vec3<f32>(0.0);

    // Number of particles based on zero crossing rate (more chaotic = more particles)
    let particle_count = 20.0 + uniforms.zero_crossing_rate * 40.0;

    for (var i = 0; i < i32(particle_count); i++) {
        let seed = f32(i) * 0.1;

        // Particle movement driven by onset strength
        let speed = 0.5 + uniforms.onset_strength * 2.0;
        let particle_time = uniforms.time * speed + seed * 10.0;

        // Swirling motion modulated by frequency bands
        let swirl_radius = 0.3 + uniforms.bass * 0.5;
        let swirl_speed = 2.0 + uniforms.treble * 3.0;

        let particle_pos = vec2<f32>(
            sin(particle_time * swirl_speed + seed) * swirl_radius,
            cos(particle_time * swirl_speed * 0.7 + seed * 2.0) * swirl_radius * 0.8
        );

        // Beat-driven pulsing
        let pulse_size = 0.05 + uniforms.beat_strength * 0.03;
        let distance_to_particle = length(pos - particle_pos);

        if (distance_to_particle < pulse_size) {
            let particle_brightness = 1.0 - (distance_to_particle / pulse_size);
            let particle_hue = seed + uniforms.time * 0.3 + uniforms.spectral_centroid * 0.2;
            color += hsv_to_rgb(particle_hue, 0.8, particle_brightness * 0.3);
        }
    }

    return color;
}

// Effect 5: Fractal Madness - Modulated by dynamic range and spectral flux
fn fractal_madness(pos: vec2<f32>) -> vec3<f32> {
    let scale = 3.0 + uniforms.dynamic_range * 5.0;
    let time_offset = uniforms.time * (0.5 + uniforms.spectral_flux);

    var p = pos * scale;
    var intensity = 0.0;
    var amplitude = 1.0;

    // Multi-octave fractal noise
    for (var i = 0; i < 5; i++) {
        intensity += smooth_noise(p + vec2<f32>(time_offset)) * amplitude;
        p *= 2.0;
        amplitude *= 0.5;
        p += vec2<f32>(uniforms.bass * 0.5, uniforms.treble * 0.3);
    }

    // Fractal distortion based on pitch confidence
    let distortion = uniforms.pitch_confidence * 2.0;
    intensity = sin(intensity * 3.14159 * distortion) * 0.5 + 0.5;

    // Color based on fractal patterns
    let hue = intensity + uniforms.time * 0.1 + uniforms.presence * 0.2;
    let saturation = 0.7 + uniforms.volume * 0.3;
    let brightness = intensity * (0.5 + uniforms.beat_strength * 0.5);

    return hsv_to_rgb(hue, saturation, brightness);
}

// ============================================================================
// EFFECT BLENDING AND MAIN SHADER
// ============================================================================

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pos = in.world_pos;

    // Calculate individual effects
    let plasma = llama_plasma(pos);
    let kaleidoscope = geometric_kaleidoscope(pos);
    let tunnel = psychedelic_tunnel(pos);
    let particles = particle_swarm(pos);
    let fractal = fractal_madness(pos);

    // Dynamic effect blending using manager-calculated weights
    var final_color = vec3<f32>(0.0);

    // Blend effects using dynamic weights from the psychedelic manager
    final_color += plasma * uniforms.plasma_weight;
    final_color += kaleidoscope * uniforms.kaleidoscope_weight;
    final_color += tunnel * uniforms.tunnel_weight;
    final_color += particles * uniforms.particle_weight;
    final_color += fractal * uniforms.fractal_weight;

    // Beat-driven global intensity boost
    let beat_boost = 1.0 + uniforms.beat_strength * 0.4;
    final_color *= beat_boost;

    // Ensure we don't exceed maximum brightness while preserving psychedelic chaos
    final_color = clamp(final_color, vec3<f32>(0.0), vec3<f32>(2.0));

    return vec4<f32>(final_color, 1.0);
}