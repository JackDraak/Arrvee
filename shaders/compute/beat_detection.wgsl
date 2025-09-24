// Beat detection compute shader
// Implements energy-based beat detection with adaptive thresholding

@group(0) @binding(0) var<storage, read> fft_data: array<vec2<f32>>;
@group(0) @binding(1) var<storage, read_write> features: array<f32, 16>; // Access to write beat features
@group(0) @binding(2) var<storage, read_write> time_data: array<f32, 4>; // [current_time, delta_time, frame_count, last_beat_time]

// Beat detection parameters
const BEAT_HISTORY_SIZE: u32 = 64u;
const ENERGY_THRESHOLD_RATIO: f32 = 1.4; // Energy must be 40% above average
const BPM_MIN: f32 = 60.0;
const BPM_MAX: f32 = 200.0;
const MIN_BEAT_INTERVAL: f32 = 0.3; // Minimum 0.3s between beats (200 BPM max)
const MAX_BEAT_INTERVAL: f32 = 1.0; // Maximum 1.0s between beats (60 BPM min)

// Persistent storage arrays (simulated - in reality these would be persistent buffers)
var<workgroup> energy_history: array<f32, 64>;
var<workgroup> beat_timestamps: array<f32, 32>;

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

// Estimate BPM from recent beat timestamps
fn estimate_bpm(current_time: f32) -> f32 {
    var total_interval = 0.0;
    var valid_intervals = 0.0;

    // Calculate intervals between recent beats (only use last 16 beats for responsiveness)
    for (var i = 16u; i < 31u; i = i + 1u) {
        let current_beat = beat_timestamps[i + 1u];
        let previous_beat = beat_timestamps[i];

        if (current_beat > previous_beat && current_beat > 0.0 && previous_beat > 0.0) {
            let interval = current_beat - previous_beat;
            if (interval >= MIN_BEAT_INTERVAL && interval <= MAX_BEAT_INTERVAL) {
                total_interval = total_interval + interval;
                valid_intervals = valid_intervals + 1.0;
            }
        }
    }

    if (valid_intervals >= 4.0) { // Need at least 4 valid intervals for stable BPM
        let avg_interval = total_interval / valid_intervals;
        let bpm = 60.0 / avg_interval;
        return clamp(bpm, BPM_MIN, BPM_MAX);
    }

    return 120.0; // Default BPM when insufficient data
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

// Enhanced beat detection with proper time management and hysteresis
fn detect_beat_with_hysteresis(current_energy: f32, threshold: f32, last_beat_time: f32, current_time: f32) -> bool {
    let time_since_last_beat = current_time - last_beat_time;
    let energy_exceeds = current_energy > threshold;
    let sufficient_time_passed = time_since_last_beat >= MIN_BEAT_INTERVAL;

    return energy_exceeds && sufficient_time_passed && current_energy > 0.1;
}

@compute @workgroup_size(1)
fn main() {
    // Read time data
    let current_time = time_data[0];
    let delta_time = time_data[1];
    let frame_count = time_data[2];
    let last_beat_time = time_data[3];

    // Initialize arrays if this is first frame
    if (frame_count < 1.0) {
        for (var i = 0u; i < BEAT_HISTORY_SIZE; i = i + 1u) {
            energy_history[i] = 0.1;
        }
        for (var i = 0u; i < 32u; i = i + 1u) {
            beat_timestamps[i] = current_time;
        }
    }

    // Calculate current beat energy with NaN protection
    var current_energy = calculate_beat_energy();
    if (current_energy != current_energy) { // NaN check
        current_energy = 0.0;
    }
    current_energy = clamp(current_energy, 0.0, 10.0);

    // Update energy history with circular buffer
    let history_index = u32(frame_count) % BEAT_HISTORY_SIZE;
    energy_history[history_index] = current_energy;

    // Calculate statistics with safety checks
    let avg_energy = max(calculate_average_energy(), 0.01);
    let adaptive_threshold = calculate_adaptive_threshold(avg_energy);

    // Enhanced beat detection with hysteresis
    let beat_detected = detect_beat_with_hysteresis(current_energy, adaptive_threshold, last_beat_time, current_time);

    // Calculate beat strength with enhanced formula
    var beat_strength = 0.0;
    if (current_energy > adaptive_threshold && current_energy > 0.1) {
        let strength_ratio = (current_energy - adaptive_threshold) / max(adaptive_threshold, 0.1);
        beat_strength = clamp(strength_ratio * 2.0, 0.0, 5.0);
    }

    // Update beat timestamps if beat detected
    var new_last_beat_time = last_beat_time;
    if (beat_detected) {
        // Shift timestamp history
        for (var i = 0u; i < 31u; i = i + 1u) {
            beat_timestamps[i] = beat_timestamps[i + 1u];
        }
        beat_timestamps[31u] = current_time;
        new_last_beat_time = current_time;
    }

    // Estimate BPM with improved algorithm
    let estimated_bpm = estimate_bpm(current_time);

    // Write results to features array with bounds checking
    features[10] = clamp(beat_strength, 0.0, 5.0);
    features[11] = clamp(estimated_bpm, BPM_MIN, BPM_MAX);

    // Update time data for next frame
    time_data[3] = new_last_beat_time; // Update last beat time

    // Debug: Write energy and threshold to additional features (indices 14, 15 are available)
    features[14] = clamp(current_energy, 0.0, 10.0);
    features[15] = clamp(adaptive_threshold, 0.0, 10.0);
}