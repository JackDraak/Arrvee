// Beat detection compute shader
// Implements energy-based beat detection with adaptive thresholding

@group(0) @binding(0) var<storage, read> fft_data: array<vec2<f32>>;
@group(0) @binding(1) var<storage, read_write> features: array<f32, 16>; // Access to write beat features

// Beat detection parameters
const BEAT_HISTORY_SIZE: u32 = 64u;
const ENERGY_THRESHOLD_RATIO: f32 = 1.3; // Energy must be 30% above average
const BPM_MIN: f32 = 60.0;
const BPM_MAX: f32 = 200.0;

// Shared memory for beat history (simulated with array indexing)
var<workgroup> energy_history: array<f32, 64>;
var<workgroup> beat_intervals: array<f32, 32>;

// Calculate magnitude of complex number
fn magnitude(complex: vec2<f32>) -> f32 {
    return sqrt(complex.x * complex.x + complex.y * complex.y);
}

// Calculate current energy in bass/low-mid frequencies (most relevant for beats)
fn calculate_beat_energy() -> f32 {
    let fft_size = arrayLength(&fft_data);
    var energy = 0.0;
    var count = 0.0;

    // Focus on 60Hz - 250Hz range for beat detection
    for (var i = 1u; i < fft_size / 8u; i = i + 1u) { // Roughly up to ~2.75kHz at 44.1kHz
        let freq = f32(i) * 44100.0 / f32(fft_size);

        if (freq >= 60.0 && freq <= 250.0) {
            let mag = magnitude(fft_data[i]);
            energy = energy + mag * mag;
            count = count + 1.0;
        }
    }

    if (count > 0.0) {
        return energy / count;
    }
    return 0.0;
}

// Calculate average energy over recent history
fn calculate_average_energy() -> f32 {
    var sum = 0.0;
    for (var i = 0u; i < BEAT_HISTORY_SIZE; i = i + 1u) {
        sum = sum + energy_history[i];
    }
    return sum / f32(BEAT_HISTORY_SIZE);
}

// Detect if current energy represents a beat
fn detect_beat(current_energy: f32, avg_energy: f32) -> bool {
    return current_energy > avg_energy * ENERGY_THRESHOLD_RATIO && current_energy > 0.1;
}

// Estimate BPM from beat intervals
fn estimate_bpm() -> f32 {
    var total_interval = 0.0;
    var valid_intervals = 0.0;

    // Calculate average interval between beats
    for (var i = 0u; i < 31u; i = i + 1u) { // 32 intervals - 1
        let interval = beat_intervals[i + 1u] - beat_intervals[i];
        if (interval > 0.0) {
            total_interval = total_interval + interval;
            valid_intervals = valid_intervals + 1.0;
        }
    }

    if (valid_intervals > 0.0) {
        let avg_interval = total_interval / valid_intervals;
        let bpm = 60.0 / avg_interval; // Convert interval to BPM
        return clamp(bpm, BPM_MIN, BPM_MAX);
    }

    return 120.0; // Default BPM
}

// Adaptive threshold calculation with variance
fn calculate_adaptive_threshold(avg_energy: f32) -> f32 {
    var variance = 0.0;

    // Calculate variance of energy history
    for (var i = 0u; i < BEAT_HISTORY_SIZE; i = i + 1u) {
        let diff = energy_history[i] - avg_energy;
        variance = variance + diff * diff;
    }

    variance = variance / f32(BEAT_HISTORY_SIZE);
    let std_dev = sqrt(variance);

    // Adaptive threshold: higher variance = lower threshold (more dynamic music)
    let adaptivity = clamp(std_dev * 2.0, 0.5, 2.0);
    return avg_energy * (1.0 + adaptivity);
}

@compute @workgroup_size(1)
fn main() {
    // Initialize arrays (in practice, these would persist between frames)
    for (var i = 0u; i < BEAT_HISTORY_SIZE; i = i + 1u) {
        energy_history[i] = 0.1; // Small baseline energy
    }

    for (var i = 0u; i < 32u; i = i + 1u) {
        beat_intervals[i] = 0.5; // Default 0.5s intervals (120 BPM)
    }

    // Calculate current beat energy
    let current_energy = calculate_beat_energy();

    // Shift energy history and add current value
    for (var i = 0u; i < BEAT_HISTORY_SIZE - 1u; i = i + 1u) {
        energy_history[i] = energy_history[i + 1u];
    }
    energy_history[BEAT_HISTORY_SIZE - 1u] = current_energy;

    // Calculate statistics
    let avg_energy = calculate_average_energy();
    let adaptive_threshold = calculate_adaptive_threshold(avg_energy);

    // Beat detection
    let beat_detected = detect_beat(current_energy, adaptive_threshold);

    // Calculate beat strength (how much energy exceeds threshold)
    var beat_strength = 0.0;
    if (current_energy > adaptive_threshold) {
        beat_strength = min((current_energy - adaptive_threshold) / adaptive_threshold, 5.0);
    }

    // Update beat intervals if beat detected
    if (beat_detected) {
        // Shift interval history
        for (var i = 0u; i < 31u; i = i + 1u) {
            beat_intervals[i] = beat_intervals[i + 1u];
        }
        // Add current time (simulated as increasing counter)
        beat_intervals[31u] = beat_intervals[30u] + 0.016667; // ~60fps assumption
    }

    // Estimate BPM
    let estimated_bpm = estimate_bpm();

    // Write results to features array
    features[10] = beat_strength; // beat_strength
    features[11] = estimated_bpm; // estimated_bpm

    // Note: Additional debug metrics would require expanding the features array
    // For now, we only write to the standard 16 features
}