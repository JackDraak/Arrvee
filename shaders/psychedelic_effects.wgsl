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
    spectralizer_weight: f32,
    parametric_weight: f32,

    // 3D projection controls
    projection_mode: f32,  // 0=sphere, 1=cylinder, 2=torus, 3=flat, -1=auto

    // Visual controls
    palette_index: f32,    // Current color palette (0-5)
    smoothing_factor: f32, // Global smoothing sensitivity (0.1-2.0)
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

// Color palette system
fn get_current_palette_color(t: f32) -> vec3<f32> {
    return get_palette_color(i32(uniforms.palette_index), t);
}

fn get_palette_color(palette_index: i32, t: f32) -> vec3<f32> {
    let normalized_t = fract(t); // Ensure t is in [0,1] range

    if (palette_index == 0) {
        // High Saturation Rainbow - always vibrant!
        return hsv_to_rgb(normalized_t, 1.0, 1.0);
    } else if (palette_index == 1) {
        // Neon Cyber palette (electric blues, magentas, cyans) - BRIGHTER
        if (normalized_t < 0.33) {
            return mix(vec3<f32>(0.2, 1.0, 1.0), vec3<f32>(1.0, 0.2, 1.0), normalized_t * 3.0);
        } else if (normalized_t < 0.66) {
            return mix(vec3<f32>(1.0, 0.2, 1.0), vec3<f32>(0.2, 1.0, 1.0), (normalized_t - 0.33) * 3.0);
        } else {
            return mix(vec3<f32>(0.2, 1.0, 1.0), vec3<f32>(0.2, 1.0, 1.0), (normalized_t - 0.66) * 3.0);
        }
    } else if (palette_index == 2) {
        // Fire palette (reds, oranges, yellows) - BRIGHTER
        if (normalized_t < 0.5) {
            return mix(vec3<f32>(1.0, 0.2, 0.0), vec3<f32>(1.0, 0.7, 0.0), normalized_t * 2.0);
        } else {
            return mix(vec3<f32>(1.0, 0.7, 0.0), vec3<f32>(1.0, 1.0, 0.2), (normalized_t - 0.5) * 2.0);
        }
    } else if (palette_index == 3) {
        // Ocean palette (bright blues, teals, aquas) - BRIGHTER
        if (normalized_t < 0.5) {
            return mix(vec3<f32>(0.0, 0.4, 1.0), vec3<f32>(0.0, 1.0, 1.0), normalized_t * 2.0);
        } else {
            return mix(vec3<f32>(0.0, 1.0, 1.0), vec3<f32>(0.6, 1.0, 1.0), (normalized_t - 0.5) * 2.0);
        }
    } else if (palette_index == 4) {
        // Retro synthwave (purple, pink, orange) - BRIGHTER
        if (normalized_t < 0.33) {
            return mix(vec3<f32>(0.6, 0.0, 1.0), vec3<f32>(1.0, 0.0, 1.0), normalized_t * 3.0);
        } else if (normalized_t < 0.66) {
            return mix(vec3<f32>(1.0, 0.0, 1.0), vec3<f32>(1.0, 0.6, 0.8), (normalized_t - 0.33) * 3.0);
        } else {
            return mix(vec3<f32>(1.0, 0.6, 0.8), vec3<f32>(1.0, 0.8, 0.0), (normalized_t - 0.66) * 3.0);
        }
    } else {
        // Fallback high-saturation rainbow
        return hsv_to_rgb(normalized_t, 1.0, 1.0);
    }
}

fn calculate_palette_index() -> i32 {
    // Cycle through palettes based on time and spectral characteristics
    let palette_cycle = uniforms.time * 0.1 + uniforms.spectral_centroid * 0.001;
    return i32(palette_cycle) % 6;
}

// Fixed dynamic range utilities with better brightness
fn calculate_dynamic_contrast() -> f32 {
    // Much more conservative contrast range
    let base_contrast = 0.8 + uniforms.dynamic_range * 0.4; // 0.8 to 1.2 range
    let beat_contrast = uniforms.beat_strength * 0.2; // Smaller beat impact
    return clamp(base_contrast + beat_contrast, 0.7, 1.5); // Conservative range
}

fn calculate_dynamic_saturation() -> f32 {
    // More conservative saturation with higher baseline
    let base_saturation = 0.6 + uniforms.volume * 0.3; // Higher baseline
    let presence_boost = uniforms.presence * 0.2; // Smaller boost
    return clamp(base_saturation + presence_boost, 0.4, 1.0); // Conservative range
}

fn apply_dynamic_range(color: vec3<f32>, contrast: f32, saturation_boost: f32) -> vec3<f32> {
    // Much more conservative dynamic range processing
    let contrasted = mix(color, pow(color, vec3<f32>(1.0 / contrast)), 0.5); // 50% blend
    let luminance = dot(contrasted, vec3<f32>(0.299, 0.587, 0.114));
    let saturated = mix(vec3<f32>(luminance), contrasted, saturation_boost);
    return saturated;
}

// Enhanced smoothing and tweening utilities
fn smooth_step_custom(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

fn smooth_transition(value: f32, smoothing_strength: f32) -> f32 {
    // Apply global smoothing with user-controlled intensity
    let smooth_factor = uniforms.smoothing_factor * smoothing_strength;
    let time_variance = sin(uniforms.time * 0.5) * (1.0 - smooth_factor) * 0.1;
    return value * (1.0 - smooth_factor * 0.3) + time_variance;
}

fn ultra_smooth_audio(raw_value: f32, response_speed: f32) -> f32 {
    // Ultra-smooth audio response for chaos reduction
    let smooth_intensity = uniforms.smoothing_factor;
    let dampened_response = raw_value * (0.3 + (1.0 - smooth_intensity) * 0.7);
    let time_smoothed = smooth_audio_parameter(dampened_response, response_speed);
    return smooth_frequency_response(time_smoothed, smooth_intensity);
}

fn exponential_smooth(current: f32, target_value: f32, smoothing_factor: f32) -> f32 {
    return mix(current, target_value, smoothing_factor);
}

fn smooth_audio_parameter(value: f32, time_factor: f32) -> f32 {
    // Apply temporal smoothing to reduce rapid changes with global smoothing factor
    let smooth_intensity = uniforms.smoothing_factor;
    let smooth_time = uniforms.time * time_factor * smooth_intensity;
    let base_smoothing = 0.8 + (1.0 - smooth_intensity) * 0.2; // More smoothing = higher baseline
    let variation = (1.0 - smooth_intensity) * 0.3; // Less variation when smoothing is high
    let smoothed = value * (base_smoothing + variation * sin(smooth_time * 0.5));
    return clamp(smoothed, 0.0, 1.0);
}

fn smooth_frequency_response(raw_value: f32, frequency_weight: f32) -> f32 {
    // Smooth frequency band responses to prevent jarring changes with global smoothing
    let smooth_threshold = 0.3 * uniforms.smoothing_factor;
    let base_smooth = smooth_step_custom(0.0, smooth_threshold, raw_value);
    let enhancement_factor = 0.6 + (1.0 - uniforms.smoothing_factor) * 0.4; // Less enhancement when more smoothing
    let enhanced = base_smooth * (enhancement_factor + frequency_weight * (1.0 - uniforms.smoothing_factor) * 0.3);
    return clamp(enhanced, 0.0, 1.0);
}

// ============================================================================
// 3D SURFACE PROJECTION FUNCTIONS
// ============================================================================

fn sphere_uv_from_screen(screen_pos: vec2<f32>) -> vec3<f32> {
    // Convert screen coordinates to sphere UV coordinates
    // This creates multiple sphere projections across the screen
    let p = screen_pos * 2.5; // Scale up for multiple spheres

    // Create a grid of spheres
    let sphere_id = floor(p);
    let local_p = fract(p) * 2.0 - 1.0; // -1 to 1 range within each sphere

    // Calculate distance from sphere center
    let dist = length(local_p);

    if (dist > 1.0) {
        // Outside sphere - return fallback UV
        return vec3<f32>(screen_pos.x, screen_pos.y, dist);
    }

    // Calculate Z coordinate for sphere surface
    let z = sqrt(1.0 - dist * dist);

    // Create 3D point on sphere surface
    let sphere_point = vec3<f32>(local_p.x, local_p.y, z);

    // Rotate sphere based on time and audio
    let rotation_speed = uniforms.estimated_bpm / 60.0 * 0.3; // Sync to BPM
    let rotation_angle = uniforms.time * rotation_speed + sphere_id.x * 1.57 + sphere_id.y * 3.14;
    let beat_modulation = 1.0 + uniforms.beat_strength * 0.2;

    let cos_rot = cos(rotation_angle * beat_modulation);
    let sin_rot = sin(rotation_angle * beat_modulation);

    // Rotate around Y axis
    let rotated_point = vec3<f32>(
        sphere_point.x * cos_rot + sphere_point.z * sin_rot,
        sphere_point.y,
        -sphere_point.x * sin_rot + sphere_point.z * cos_rot
    );

    // Convert 3D point to UV coordinates for texture sampling
    let u = atan2(rotated_point.z, rotated_point.x) / (2.0 * 3.14159) + 0.5;
    let v = asin(clamp(rotated_point.y, -1.0, 1.0)) / 3.14159 + 0.5;

    // Add sphere_id influence for variation
    let varied_u = u + sin(sphere_id.x * 2.1 + sphere_id.y * 3.7) * 0.1;
    let varied_v = v + cos(sphere_id.x * 1.3 + sphere_id.y * 2.9) * 0.1;

    return vec3<f32>(fract(varied_u), fract(varied_v), 1.0 - dist); // Return UV + inverse distance for effects
}

fn cylinder_uv_from_screen(screen_pos: vec2<f32>) -> vec3<f32> {
    // Create cylindrical projection for tunnel-like effects
    let center = vec2<f32>(0.5, 0.5);
    let p = screen_pos - center;

    // Calculate angle and distance from center
    let angle = atan2(p.y, p.x);
    let dist = length(p) * 2.0; // Scale for better coverage

    // Audio-reactive cylinder warping
    let bass_warp = 1.0 + uniforms.bass * 0.3;
    let time_warp = uniforms.time * 0.5 + uniforms.onset_strength * 2.0;

    // Create cylindrical coordinates with warping
    let u = (angle / (2.0 * 3.14159) + 0.5 + sin(time_warp) * 0.1) * bass_warp;
    let v = (screen_pos.y + cos(uniforms.time * 0.7 + angle) * 0.1) * (1.0 + uniforms.mid * 0.2);

    return vec3<f32>(fract(u), fract(v), clamp(1.0 - dist, 0.0, 1.0));
}

fn torus_uv_from_screen(screen_pos: vec2<f32>) -> vec3<f32> {
    // Create torus (donut) projection
    let center = vec2<f32>(0.5, 0.5);
    let p = screen_pos - center;

    // Torus parameters - audio reactive
    let major_radius = 0.25 + uniforms.volume * 0.15;
    let minor_radius = 0.08 + uniforms.treble * 0.1;

    let dist_from_center = length(p);
    let angle_around_torus = atan2(p.y, p.x) + uniforms.time * 0.3;

    // Calculate position on torus surface
    let torus_center_dist = abs(dist_from_center - major_radius);

    if (torus_center_dist > minor_radius * 2.0) {
        // Outside torus - return fallback
        return vec3<f32>(screen_pos.x, screen_pos.y, 0.1);
    }

    // Calculate UV coordinates on torus surface
    let u = angle_around_torus / (2.0 * 3.14159) + 0.5;
    let v = (torus_center_dist / minor_radius) * (1.0 + uniforms.presence * 0.3);

    // Add beat synchronization
    let beat_phase = uniforms.time * uniforms.estimated_bpm / 60.0;
    let u_modulated = u + sin(beat_phase * 2.0) * 0.05;
    let v_modulated = v + cos(beat_phase * 3.0) * 0.03;

    return vec3<f32>(fract(u_modulated), fract(v_modulated), 1.0 - torus_center_dist / minor_radius);
}

fn apply_surface_projection(screen_pos: vec2<f32>, projection_type: i32) -> vec3<f32> {
    // Select projection type based on audio characteristics or manual selection
    if (projection_type == 0) {
        return sphere_uv_from_screen(screen_pos);
    } else if (projection_type == 1) {
        return cylinder_uv_from_screen(screen_pos);
    } else if (projection_type == 2) {
        return torus_uv_from_screen(screen_pos);
    } else {
        // Flat projection (default)
        return vec3<f32>(screen_pos.x, screen_pos.y, 1.0);
    }
}

// ============================================================================
// PSYCHEDELIC EFFECTS COLLECTION
// ============================================================================

// Effect 1: Llama Plasma Fields - Driven by frequency bands and spectral flux
fn llama_plasma(pos: vec2<f32>) -> vec3<f32> {
    // Ultra-smooth time evolution with enhanced smoothing
    let smooth_flux = ultra_smooth_audio(uniforms.spectral_flux, 2.0);
    let time_speed = smooth_transition(uniforms.time, 1.0) * (0.3 + smooth_flux * 0.7); // More controlled

    // Ultra-smooth beat pulsing
    let smooth_beat = ultra_smooth_audio(uniforms.beat_strength, 1.5);
    let beat_pulse = 1.0 + smooth_transition(smooth_beat, 0.8) * 0.5; // Even more conservative

    // Smooth frequency-driven scaling with reduced ranges
    let smooth_sub_bass = smooth_frequency_response(uniforms.sub_bass, 0.3);
    let smooth_bass = smooth_frequency_response(uniforms.bass, 0.4);
    let smooth_mid = smooth_frequency_response(uniforms.mid, 0.5);
    let smooth_treble = smooth_frequency_response(uniforms.treble, 0.6);

    let scale1 = 2.0 + smooth_sub_bass * 6.0;  // Reduced from 15.0
    let scale2 = 3.0 + smooth_bass * 5.0;      // Reduced from 12.0
    let scale3 = 4.0 + smooth_mid * 4.0;       // Reduced from 8.0
    let scale4 = 1.0 + smooth_treble * 8.0;    // Reduced from 20.0

    // Multi-layered plasma with enhanced contrast
    let wave1 = sin(pos.x * scale1 + time_speed) * cos(pos.y * scale1 + time_speed * 0.7);
    let wave2 = sin((pos.x + pos.y) * scale2 * 0.7 + time_speed * 1.3);
    let wave3 = sin(length(pos) * scale3 + time_speed * 0.8);
    let wave4 = cos(atan2(pos.y, pos.x) * scale4 + time_speed * 2.0) * uniforms.treble;

    // Combine waves with dynamic amplitude
    let plasma_raw = (wave1 + wave2 + wave3 + wave4) * 0.25 * beat_pulse;

    // Gentle dynamic range processing
    let dynamic_factor = calculate_dynamic_contrast();
    let plasma = plasma_raw * 0.8; // More conservative processing

    // Smooth color evolution
    let smooth_centroid = smooth_audio_parameter(uniforms.spectral_centroid, 1.0);
    let hue_base = uniforms.time * 0.05 + smooth_centroid * 0.3; // Slower color changes
    let hue = hue_base + plasma * 0.2; // Reduced color variation

    // Smooth saturation changes
    let base_saturation = calculate_dynamic_saturation();
    let smooth_presence = smooth_audio_parameter(uniforms.presence, 1.2);
    let saturation = mix(0.4, 0.9, base_saturation + smooth_presence * 0.3); // Narrower range

    // Fixed brightness with better baseline
    let brightness_base = 0.4 + uniforms.volume * 0.4; // Higher baseline
    let brightness_plasma = abs(plasma) * 0.4; // Reduced intensity
    let brightness = clamp(brightness_base + brightness_plasma, 0.2, 1.2); // Conservative range

    // Use palette system instead of HSV
    let palette_index = calculate_palette_index();
    let palette_t = hue + brightness * 0.3; // Use hue calculation as palette index
    let base_color = get_current_palette_color(palette_t) * brightness;

    // Apply minimal dynamic range processing
    return apply_dynamic_range(base_color, dynamic_factor, base_saturation + 0.3);
}

// Effect 2: Geometric Kaleidoscope - Controlled by BPM and pitch confidence
fn geometric_kaleidoscope(pos: vec2<f32>) -> vec3<f32> {
    // Smooth BPM changes to prevent jarring speed shifts
    let smooth_bpm = exponential_smooth(120.0, uniforms.estimated_bpm, 0.1);
    let bpm_factor = smooth_bpm / 120.0;

    // Smooth beat synchronization
    let smooth_beat = smooth_audio_parameter(uniforms.beat_strength, 1.0);
    let beat_sync = 1.0 + smooth_beat * 1.5; // Reduced from 3.0
    let rotation_speed = uniforms.time * bpm_factor * 0.3 * beat_sync; // Slower rotation

    // Rotate position based on BPM with dynamic distortion
    let cos_r = cos(rotation_speed);
    let sin_r = sin(rotation_speed);
    let rotated_pos = vec2<f32>(
        pos.x * cos_r - pos.y * sin_r,
        pos.x * sin_r + pos.y * cos_r
    );

    let distance = length(rotated_pos);
    let angle = atan2(rotated_pos.y, rotated_pos.x);

    // Smooth kaleidoscope segments to prevent jarring changes
    let smooth_pitch = smooth_audio_parameter(uniforms.pitch_confidence, 0.8);
    let smooth_onset = smooth_audio_parameter(uniforms.onset_strength, 1.5);

    let segments_base = 6.0 + smooth_pitch * 8.0; // Much more conservative range
    let segments = segments_base + smooth_onset * 3.0; // Gentle segment changes
    let segment_angle = (3.14159 * 2.0) / segments;
    let folded_angle = abs((angle % segment_angle) - segment_angle * 0.5);

    // Smooth ring patterns
    let ring_frequency_base = 8.0 + smooth_onset * 12.0; // Reduced from 40.0
    let smooth_dynamic = smooth_audio_parameter(uniforms.dynamic_range, 1.0);
    let ring_frequency = ring_frequency_base * (1.0 + smooth_dynamic * 0.8); // Reduced multiplier
    let ring_pattern = sin(distance * ring_frequency - uniforms.time * 2.0 * beat_sync); // Slower

    // Dynamic radial spokes with frequency content
    let spoke_density = 10.0 + uniforms.treble * 30.0;
    let spoke_pattern = sin(folded_angle * spoke_density + uniforms.time * 2.0);

    // Combine patterns with dynamic amplitude
    let pattern_raw = (ring_pattern + spoke_pattern) * 0.5;

    // Apply extreme contrast based on audio dynamics
    let contrast = calculate_dynamic_contrast();
    let pattern = sign(pattern_raw) * pow(abs(pattern_raw), 1.0 / contrast);

    // Much brighter intensity calculation
    let base_intensity = smoothstep(-0.5, 0.3, pattern);
    let distance_fade = 1.0 - smoothstep(0.8, 1.8, distance); // Larger visible area
    let volume_gate = 0.6 + smoothstep(0.02, 0.1, uniforms.volume) * 0.4; // Higher baseline

    let intensity = base_intensity * distance_fade * volume_gate * (1.0 + smooth_beat * 0.8);

    // Use palette system for better colors
    let palette_index = calculate_palette_index();
    let color_t = uniforms.zero_crossing_rate * 0.5 + folded_angle * 0.1 + uniforms.time * 0.05;

    // Much brighter final result
    let brightness_multiplier = 0.8 + uniforms.volume * 0.6 + intensity * 0.8; // Much higher
    let final_color = get_current_palette_color(color_t) * brightness_multiplier;

    // Minimal processing to preserve brightness
    return clamp(final_color, vec3<f32>(0.0), vec3<f32>(1.8));
}

// Effect 3: Tunnel Vision - Minter-style tunnel with spectral rolloff control
fn psychedelic_tunnel(pos: vec2<f32>) -> vec3<f32> {
    let distance = length(pos);
    let angle = atan2(pos.y, pos.x);

    // Dynamic tunnel depth with extreme range
    let tunnel_depth_base = 2.0 + uniforms.spectral_rolloff * 20.0; // Much deeper range
    let tunnel_depth = tunnel_depth_base * (1.0 + uniforms.dynamic_range * 3.0);
    let z = tunnel_depth / max(distance, 0.05);

    // Dynamic tunnel twist with beat synchronization
    let twist_speed = uniforms.time * (1.0 + uniforms.beat_strength * 4.0);
    let twist = twist_speed + uniforms.spectral_flux * 10.0;
    let twisted_angle = angle + z * (0.3 + uniforms.onset_strength * 2.0) + twist;

    // Multi-layer tunnel patterns with dynamic frequency
    let stripe_freq_base = 10.0 + uniforms.presence * 60.0; // Much wider frequency range
    let stripe_frequency = stripe_freq_base * (1.0 + uniforms.dynamic_range * 2.0);
    let stripes = sin(z * stripe_frequency + uniforms.time * 8.0);

    // Dynamic spiral density
    let spiral_density = 4.0 + uniforms.treble * 20.0;
    let spiral = sin(twisted_angle * spiral_density + z * (5.0 + uniforms.bass * 15.0));

    // Additional layer for high-frequency detail
    let fine_detail = sin(z * 100.0 + uniforms.time * 20.0) * uniforms.treble * 0.3;

    // Combine patterns with dynamic amplitude
    let pattern_raw = (stripes + spiral + fine_detail) / 3.0;

    // Apply extreme contrast
    let contrast = calculate_dynamic_contrast();
    let pattern = sign(pattern_raw) * pow(abs(pattern_raw), 1.0 / contrast);

    // Much brighter tunnel calculation
    let brightness_base = smoothstep(-0.3, 0.7, pattern);
    let volume_gate = 0.7 + smoothstep(0.01, 0.08, uniforms.volume) * 0.3; // Higher baseline

    // Moderate beat pulsing
    let smooth_beat_tunnel = smooth_audio_parameter(uniforms.beat_strength, 2.0);
    let beat_pulse = 1.0 + smooth_beat_tunnel * 2.0; // Reduced from 4.0
    let tunnel_brightness = brightness_base * beat_pulse * volume_gate;

    // Use palette system for better colors
    let palette_index = calculate_palette_index();
    let color_t = z * 0.03 + uniforms.time * 0.08 + uniforms.mid * 0.2;

    // Dynamic distance fade - more forgiving
    let fade = 1.0 - smoothstep(0.0, 1.5, distance); // Fixed range

    // Much brighter final result
    let brightness_multiplier = 0.9 + uniforms.volume * 0.7 + tunnel_brightness * 0.6;
    let final_color = get_current_palette_color(color_t) * brightness_multiplier * fade;

    // Minimal processing to preserve brightness
    return clamp(final_color, vec3<f32>(0.0), vec3<f32>(1.8));
}

// Effect 4: Particle Swarm - Triggered by onset strength and zero crossing rate
fn particle_swarm(pos: vec2<f32>) -> vec3<f32> {
    var color = vec3<f32>(0.0);

    // Smooth particle count to prevent jarring changes
    let smooth_zero_crossing = smooth_audio_parameter(uniforms.zero_crossing_rate, 1.5);
    let smooth_onset = smooth_audio_parameter(uniforms.onset_strength, 1.2);
    let chaos_factor = smooth_zero_crossing + smooth_onset * 0.3;
    let particle_count = clamp(15.0 + chaos_factor * 30.0, 5.0, 50.0); // Safe bounds
    let volume_gate = max(smoothstep(0.05, 0.15, uniforms.volume), 0.4); // Brighter baseline

    for (var i = 0; i < i32(particle_count); i++) {
        let seed = f32(i) * 0.073; // Slightly different seed spacing

        // Smooth particle movement
        let smooth_onset_local = smooth_audio_parameter(uniforms.onset_strength, 1.8);
        let smooth_beat_local = smooth_audio_parameter(uniforms.beat_strength, 1.5);

        let speed_base = 0.5 + smooth_onset_local * 2.0; // Reduced from 5.0
        let beat_speed_boost = 1.0 + smooth_beat_local * 1.2; // Reduced from 3.0
        let particle_time = uniforms.time * speed_base * beat_speed_boost + seed * 10.0;

        // Smooth swirling motion
        let smooth_bass_local = smooth_frequency_response(uniforms.bass, 0.5);
        let smooth_treble_local = smooth_frequency_response(uniforms.treble, 0.6);
        let smooth_flux_local = smooth_audio_parameter(uniforms.spectral_flux, 1.0);
        let smooth_dynamic_local = smooth_audio_parameter(uniforms.dynamic_range, 1.2);

        let swirl_radius_base = 0.3 + smooth_bass_local * 0.6; // Reduced range
        let swirl_radius = swirl_radius_base * (1.0 + smooth_dynamic_local * 0.8);

        let swirl_speed_base = 1.0 + smooth_treble_local * 3.0; // Reduced from 8.0
        let swirl_speed = swirl_speed_base * (1.0 + smooth_flux_local * 1.0); // Reduced from 3.0

        // Complex particle motion with chaotic elements
        let chaos_offset = sin(seed * 100.0 + uniforms.time * 5.0) * chaos_factor * 0.3;

        let particle_pos = vec2<f32>(
            sin(particle_time * swirl_speed + seed) * swirl_radius + chaos_offset,
            cos(particle_time * swirl_speed * 0.7 + seed * 2.0) * swirl_radius * 0.8 +
            sin(particle_time * 3.0 + seed * 5.0) * uniforms.mid * 0.4
        );

        // Dynamic particle size with explosive beats
        let pulse_size_base = 0.02 + uniforms.beat_strength * 0.15; // Much larger size range
        let pulse_size = pulse_size_base * (1.0 + uniforms.volume * 2.0);
        let distance_to_particle = length(pos - particle_pos);

        if (distance_to_particle < pulse_size) {
            // Much brighter particle calculation
            let brightness_raw = 1.0 - (distance_to_particle / pulse_size);
            let particle_brightness = pow(brightness_raw, 0.7) * volume_gate; // Gentler curve

            // Use palette system for consistent bright colors
            let palette_index = calculate_palette_index();
            let color_t = seed + uniforms.time * 0.3 + uniforms.spectral_centroid * 0.2;

            // Much higher intensity scaling
            let intensity_base = 1.0 + uniforms.volume * 1.2; // Even higher baseline
            let beat_boost = 1.2 + smooth_beat_local * 2.0;
            let intensity = particle_brightness * intensity_base * beat_boost;

            let particle_color = get_current_palette_color(color_t) * intensity;

            // Add more brightness for visible particles
            color = color + particle_color * 1.2; // Higher contribution
        }
    }

    return color;
}

// Effect 5: Fractal Madness - Modulated by dynamic range and spectral flux
fn fractal_madness(pos: vec2<f32>) -> vec3<f32> {
    // Smooth scaling to prevent jarring changes
    let smooth_dynamic_range = smooth_audio_parameter(uniforms.dynamic_range, 1.0);
    let smooth_beat_strength = smooth_audio_parameter(uniforms.beat_strength, 1.3);

    let scale_base = 2.0 + smooth_dynamic_range * 4.0; // More conservative range
    let scale = scale_base * (1.0 + smooth_beat_strength * 1.2); // Reduced from 3.0

    // Ultra-smooth time evolution
    let smooth_spectral_flux = ultra_smooth_audio(uniforms.spectral_flux, 1.5);
    let time_speed = 0.2 + smooth_transition(smooth_spectral_flux, 1.0) * 0.5; // Even more reduced
    let time_offset = uniforms.time * time_speed;

    var p = pos * scale;
    var intensity = 0.0;
    var amplitude = 1.0;
    let volume_gate = max(smoothstep(0.02, 0.12, uniforms.volume), 0.4); // Brighter baseline

    // Safe multi-octave fractal with bounded octaves
    let octave_count = clamp(3 + i32(uniforms.onset_strength * 4.0), 3, 8); // Safe bounds 3-8

    for (var i = 0; i < octave_count; i++) {
        // Dynamic noise sampling with frequency content
        let noise_offset = vec2<f32>(
            time_offset + uniforms.bass * 2.0,
            time_offset * 0.7 + uniforms.treble * 1.5
        );

        intensity = intensity + smooth_noise(p + noise_offset) * amplitude;

        // Dynamic scaling factor
        let scale_factor = 1.8 + uniforms.zero_crossing_rate * 0.8;
        p = p * scale_factor;
        amplitude = amplitude * (0.4 + uniforms.presence * 0.3);

        // Dynamic position offset based on frequency content
        p = p + vec2<f32>(
            uniforms.bass * (1.0 + uniforms.dynamic_range),
            uniforms.treble * (1.0 + uniforms.spectral_flux * 2.0)
        );
    }

    // Multi-layer fractal distortion
    let distortion_base = uniforms.pitch_confidence * 4.0; // Stronger distortion
    let distortion = distortion_base + uniforms.onset_strength * 2.0;

    // Complex distortion function
    let intensity_raw = sin(intensity * 3.14159 * distortion) * 0.5 + 0.5;
    let intensity_warped = sin(intensity_raw * 6.28318 + uniforms.time) * 0.3 + 0.7;

    // Apply dynamic contrast to fractal patterns
    let contrast = calculate_dynamic_contrast();
    let final_intensity = pow(intensity_warped, 1.0 / contrast) * volume_gate;

    // Dynamic color evolution
    let hue_base = final_intensity + uniforms.time * 0.2;
    let hue = hue_base + uniforms.presence * 0.5 + uniforms.spectral_centroid * 0.3;

    // Extreme saturation variation
    let saturation_base = calculate_dynamic_saturation();
    let saturation = mix(0.1, 1.4, saturation_base + uniforms.volume * 0.6);

    // Dynamic brightness with enhanced visibility
    let brightness_base = final_intensity * (0.6 + uniforms.volume * 1.5);
    let brightness = brightness_base * (1.5 + uniforms.beat_strength * 2.5);

    let base_color = hsv_to_rgb(hue, saturation, brightness);

    // Apply dynamic range processing
    return apply_dynamic_range(base_color, contrast, saturation_base + 0.2);
}

// Effect 6: Mirroring Kaleidoscope - Enhanced symmetrical patterns
fn mirroring_kaleidoscope(pos: vec2<f32>) -> vec3<f32> {
    // Multiple mirror layers for complex symmetry
    var p = pos;

    // First mirror layer - vertical and horizontal
    p = vec2<f32>(abs(p.x), abs(p.y));

    // Diagonal mirrors
    if (p.x + p.y > 1.0) {
        p = vec2<f32>(1.0 - p.y, 1.0 - p.x);
    }

    // Circular mirror boundary
    let distance_from_center = length(p);
    if (distance_from_center > 0.8) {
        p = p * (1.6 - distance_from_center) / distance_from_center;
    }

    // Smooth rotation with BPM
    let smooth_bpm = exponential_smooth(120.0, uniforms.estimated_bpm, 0.05);
    let smooth_beat = smooth_audio_parameter(uniforms.beat_strength, 1.2);

    let rotation_speed = uniforms.time * (smooth_bpm / 120.0) * 0.2;
    let cos_r = cos(rotation_speed);
    let sin_r = sin(rotation_speed);

    p = vec2<f32>(
        p.x * cos_r - p.y * sin_r,
        p.x * sin_r + p.y * cos_r
    );

    // Multi-layer patterns
    let smooth_onset = smooth_audio_parameter(uniforms.onset_strength, 1.0);
    let smooth_pitch = smooth_audio_parameter(uniforms.pitch_confidence, 0.8);

    // Concentric rings
    let ring_freq = 8.0 + smooth_onset * 15.0;
    let rings = sin(length(p) * ring_freq - uniforms.time * 2.0);

    // Radial spokes
    let spoke_count = 6.0 + smooth_pitch * 12.0;
    let angle = atan2(p.y, p.x);
    let spokes = sin(angle * spoke_count + uniforms.time * 1.5);

    // Grid pattern
    let grid_scale = 4.0 + uniforms.treble * 8.0;
    let grid = sin(p.x * grid_scale) * sin(p.y * grid_scale);

    // Combine patterns
    let pattern = (rings + spokes + grid * 0.5) / 2.5;

    // Enhanced brightness calculation
    let base_intensity = smoothstep(-0.2, 0.6, pattern);
    let volume_boost = 0.8 + uniforms.volume * 0.4;
    let beat_boost = 1.0 + smooth_beat * 0.6;

    let intensity = base_intensity * volume_boost * beat_boost;

    // Use palette system
    let palette_index = calculate_palette_index();
    let color_t = length(p) * 0.3 + angle * 0.1 + uniforms.time * 0.06;

    let final_color = get_current_palette_color(color_t) * intensity;

    return clamp(final_color, vec3<f32>(0.0), vec3<f32>(1.5));
}

// Effect 7: Spectralizer - Classic spectrum analyzer bars
fn spectralizer_bars(pos: vec2<f32>) -> vec3<f32> {
    // Map position to frequency bands
    let x_normalized = (pos.x + 1.0) * 0.5; // Convert from [-1,1] to [0,1]
    let frequency_index = clamp(x_normalized * 5.0, 0.0, 4.99);
    let band_index = i32(frequency_index);
    let band_blend = fract(frequency_index);

    // Get frequency band values
    var band_value: f32;
    if (band_index == 0) {
        band_value = mix(uniforms.sub_bass, uniforms.bass, band_blend);
    } else if (band_index == 1) {
        band_value = mix(uniforms.bass, uniforms.mid, band_blend);
    } else if (band_index == 2) {
        band_value = mix(uniforms.mid, uniforms.treble, band_blend);
    } else if (band_index == 3) {
        band_value = mix(uniforms.treble, uniforms.presence, band_blend);
    } else {
        band_value = uniforms.presence;
    }

    // Apply beat boost and volume scaling
    let boosted_value = band_value * (1.0 + uniforms.beat_strength * 2.0) * (0.5 + uniforms.volume * 1.5);

    // Create vertical bars
    let bar_height = boosted_value * 1.5; // Scale to screen space
    let y_normalized = (pos.y + 1.0) * 0.5; // Convert from [-1,1] to [0,1]

    // Bar visualization with gradient
    let bar_intensity = smoothstep(0.0, bar_height, 1.0 - y_normalized);

    // Add some width variation and glow
    let bar_width = 0.15 + sin(uniforms.time * 2.0 + f32(band_index)) * 0.05;
    let distance_to_center = abs(fract(frequency_index) - 0.5) * 2.0;
    let width_fade = smoothstep(bar_width, 0.0, distance_to_center);

    let final_intensity = bar_intensity * width_fade;

    // Color based on frequency and palette
    let palette_index = calculate_palette_index();
    let color_t = f32(band_index) * 0.2 + uniforms.time * 0.1 + bar_intensity * 0.3;
    let base_color = get_current_palette_color(color_t);

    // Add glow effect
    let glow = smoothstep(0.0, 0.3, final_intensity) * 0.3;
    let final_color = base_color * (final_intensity + glow);

    return clamp(final_color, vec3<f32>(0.0), vec3<f32>(1.5));
}

// Effect 7: Parametric Waves - Mathematical audio-reactive patterns
fn parametric_waves(pos: vec2<f32>) -> vec3<f32> {
    // Ultra-smooth audio parameters to prevent jarring changes
    let smooth_bass = ultra_smooth_audio(uniforms.bass + uniforms.sub_bass, 3.0);
    let smooth_treble = ultra_smooth_audio(uniforms.treble + uniforms.presence, 3.0);
    let smooth_spectral_flux = ultra_smooth_audio(uniforms.spectral_flux, 4.0);
    let smooth_onset = ultra_smooth_audio(uniforms.onset_strength, 2.0);

    // Pattern frequency driven by spectral centroid and onset strength
    let base_frequency = 2.0 + uniforms.spectral_centroid * 0.0001;
    let onset_boost = smooth_onset * 5.0;
    let frequency = base_frequency + onset_boost;

    // Animation speed synchronized to BPM with beat emphasis
    let bpm_factor = uniforms.estimated_bpm / 120.0;
    let beat_smooth = ultra_smooth_audio(uniforms.beat_strength, 1.5);
    let speed = 0.5 + bpm_factor * 1.0 + beat_smooth * 1.5; // More controlled speed

    // Overall intensity from volume and dynamic range
    let volume_smooth = ultra_smooth_audio(uniforms.volume, 2.0);
    let intensity = 0.3 + volume_smooth * 0.4 + uniforms.dynamic_range * 0.3; // Less intense

    // Generate mathematical patterns
    let angle = atan2(pos.y, pos.x);
    let radius = length(pos);

    // Multiple wave patterns responding to different audio features
    let wave1 = sin(radius * frequency - uniforms.time * speed);
    let wave2 = cos(angle * (4.0 + smooth_bass * 4.0) + uniforms.time * speed * 0.7); // Reduced bass response
    let wave3 = sin(length(pos * (2.0 + smooth_treble * 2.0)) * 3.0 - uniforms.time * speed * 1.3); // Reduced treble response

    // Beat-driven pulse wave (smoother)
    let beat_pulse = sin(uniforms.time * speed * 2.0) * beat_smooth * 0.5; // Reduced amplitude
    let wave4 = cos(radius * 6.0 + beat_pulse * 5.0); // Reduced beat influence

    // Spectral flux creates texture variation (much smoother)
    let texture_noise = sin(pos.x * 15.0 + smooth_spectral_flux * 25.0) *
                       cos(pos.y * 12.0 + smooth_spectral_flux * 20.0) * 0.05; // Much subtler

    // Combine patterns with controlled intensity
    let pattern = (wave1 + wave2 + wave3 + wave4 + texture_noise) * intensity * 0.6; // Reduced overall amplitude

    // Color generation using current palette (smoother transitions)
    let color_t1 = smooth_transition((pattern + 1.0) * 0.5, 2.0);
    let color_t2 = smooth_transition((sin(pattern * 1.2) + 1.0) * 0.5, 2.0); // Reduced frequency
    let color_t3 = smooth_transition((cos(pattern * 1.8) + 1.0) * 0.5, 2.0); // Reduced frequency

    let color1 = get_current_palette_color(color_t1);
    let color2 = get_current_palette_color(color_t2 + 0.33);
    let color3 = get_current_palette_color(color_t3 + 0.67);

    // Mix colors based on pattern values (gentler mixing)
    var final_color = color1 * (0.4 + 0.3 * cos(pattern)); // Reduced variation
    final_color = final_color + color2 * (0.2 + 0.2 * sin(pattern * 1.2)); // Reduced variation
    final_color = final_color + color3 * (0.1 + 0.1 * cos(pattern * 1.5)); // Reduced variation
    final_color = final_color / 2.2; // Normalize after mixing

    // Subtle beat-driven chromatic effects
    final_color.r = final_color.r + sin(uniforms.time * 0.3 + beat_smooth * 2.0) * 0.05; // Much subtler
    final_color.b = final_color.b + cos(uniforms.time * 0.2 + beat_smooth * 1.5) * 0.05; // Much subtler

    // Gentle onset-driven color shifts
    let onset_shift = smooth_onset * sin(pattern * 2.0) * 0.1; // Much subtler
    final_color.g = final_color.g + onset_shift;

    // Zero crossing rate affects saturation gently
    let saturation_factor = 1.0 + uniforms.zero_crossing_rate * 0.2; // Much subtler
    final_color = final_color * saturation_factor;

    // Radial gradient with smooth bass response
    let gradient_power = 0.6 + smooth_bass * 0.3; // Gentler gradient
    let gradient = 1.0 - pow(radius * 0.8, gradient_power); // Reduced radius impact
    final_color = final_color * gradient;

    // Dynamic range affects overall brightness smoothly
    let brightness_factor = 0.7 + uniforms.dynamic_range * 0.3; // More controlled brightness
    final_color = final_color * brightness_factor;

    // Apply dynamic range and saturation boost
    final_color = apply_dynamic_range(final_color, 1.2, 1.1); // Gentle enhancement

    // Ensure color values stay in valid range
    return clamp(final_color, vec3<f32>(0.0), vec3<f32>(1.0));
}

// ============================================================================
// EFFECT BLENDING AND MAIN SHADER
// ============================================================================

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let screen_pos = in.world_pos;

    // Determine projection type based on manual setting or intelligent selection
    var projection_type = 3; // Default to flat projection

    // Check if manual projection mode is set
    if (uniforms.projection_mode >= 0.0) {
        projection_type = i32(uniforms.projection_mode);
    } else {
        // Intelligent projection selection based on effect weights and audio
        let max_weight = max(max(max(max(max(uniforms.plasma_weight, uniforms.kaleidoscope_weight),
                                       uniforms.tunnel_weight), uniforms.particle_weight),
                                 uniforms.fractal_weight), uniforms.spectralizer_weight);

        // Choose projection based on dominant effect and audio characteristics
        if (uniforms.tunnel_weight == max_weight && uniforms.tunnel_weight > 0.3) {
            projection_type = 1; // Cylinder for tunnel effects
        } else if (uniforms.kaleidoscope_weight == max_weight && uniforms.kaleidoscope_weight > 0.3) {
            projection_type = 2; // Torus for kaleidoscope effects
        } else if ((uniforms.plasma_weight == max_weight || uniforms.particle_weight == max_weight) && max_weight > 0.4) {
            projection_type = 0; // Sphere for plasma and particles
        } else if (uniforms.bass > 0.6) {
            projection_type = 0; // Spheres for heavy bass
        } else if (uniforms.presence > 0.5) {
            projection_type = 1; // Cylinder for bright sounds
        }
    }

    // Apply 3D surface projection to get modified UV coordinates
    let projection_result = apply_surface_projection(screen_pos, projection_type);
    let pos = vec2<f32>(projection_result.x, projection_result.y);
    let depth_factor = projection_result.z; // Use depth for intensity modulation

    // Calculate individual effects with projected coordinates
    let plasma = llama_plasma(pos) * (1.0 + depth_factor * 0.3);
    let kaleidoscope = geometric_kaleidoscope(pos) * (1.0 + depth_factor * 0.2);
    let tunnel = psychedelic_tunnel(pos) * (1.0 + depth_factor * 0.4);
    let particles = particle_swarm(pos) * (1.0 + depth_factor * 0.5);
    let fractal = fractal_madness(pos) * (1.0 + depth_factor * 0.25);
    let spectralizer = spectralizer_bars(pos) * (1.0 + depth_factor * 0.1);
    let parametric = parametric_waves(pos) * (1.0 + depth_factor * 0.3);

    // Dynamic effect blending using manager-calculated weights
    var final_color = vec3<f32>(0.0);

    // Blend effects using dynamic weights from the psychedelic manager
    final_color = final_color + plasma * uniforms.plasma_weight;
    final_color = final_color + kaleidoscope * uniforms.kaleidoscope_weight;
    final_color = final_color + tunnel * uniforms.tunnel_weight;
    final_color = final_color + particles * uniforms.particle_weight;
    final_color = final_color + fractal * uniforms.fractal_weight;
    final_color = final_color + spectralizer * uniforms.spectralizer_weight;
    final_color = final_color + parametric * uniforms.parametric_weight;

    // Smooth global processing for stability
    let global_contrast = calculate_dynamic_contrast();
    let global_saturation = calculate_dynamic_saturation();

    // Very gentle dynamic range processing
    final_color = apply_dynamic_range(final_color, global_contrast, global_saturation);

    // Smooth beat and volume response
    let smooth_beat_global = smooth_audio_parameter(uniforms.beat_strength, 2.0);
    let smooth_volume_global = smooth_audio_parameter(uniforms.volume, 1.5);

    let beat_boost = 0.9 + smooth_beat_global * 0.3; // Very conservative
    let volume_boost = 0.7 + smooth_volume_global * 0.4; // Stable baseline
    final_color = final_color * beat_boost * volume_boost;

    // Stable brightness control
    let brightness_control = 0.8 + smooth_volume_global * 0.3; // High, stable baseline
    final_color = final_color * brightness_control;

    // Gentle saturation enhancement
    let smooth_dynamic_global = smooth_audio_parameter(uniforms.dynamic_range, 1.8);
    let saturation_push = 0.95 + smooth_dynamic_global * 0.1; // Very subtle
    let luminance = dot(final_color, vec3<f32>(0.299, 0.587, 0.114));
    final_color = mix(vec3<f32>(luminance), final_color, saturation_push);

    // Higher brightness ceiling
    final_color = clamp(final_color, vec3<f32>(0.0), vec3<f32>(2.0));

    return vec4<f32>(final_color, 1.0);
}