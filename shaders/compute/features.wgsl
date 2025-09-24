// Audio feature extraction compute shader
// Processes FFT output to extract musical features

@group(0) @binding(0) var<storage, read> fft_data: array<vec2<f32>>;
@group(0) @binding(1) var<storage, read_write> features: array<f32, 16>; // GpuAudioFeatures as flat array

// Frequency band boundaries (Hz) for 44.1kHz sample rate
const SUB_BASS_MAX: f32 = 60.0;
const BASS_MAX: f32 = 250.0;
const MID_MAX: f32 = 4000.0;
const TREBLE_MAX: f32 = 12000.0;
const PRESENCE_MAX: f32 = 20000.0;

// Sample rate constant (TODO: make configurable)
const SAMPLE_RATE: f32 = 44100.0;

// Calculate magnitude of complex number
fn magnitude(complex: vec2<f32>) -> f32 {
    return sqrt(complex.x * complex.x + complex.y * complex.y);
}

// Convert FFT bin to frequency
fn bin_to_frequency(bin: u32, fft_size: u32) -> f32 {
    return f32(bin) * SAMPLE_RATE / f32(fft_size);
}

// Extract frequency bands from FFT data
fn extract_frequency_bands() {
    let fft_size = arrayLength(&fft_data);
    var sub_bass = 0.0;
    var bass = 0.0;
    var mid = 0.0;
    var treble = 0.0;
    var presence = 0.0;

    var sub_bass_count = 0.0;
    var bass_count = 0.0;
    var mid_count = 0.0;
    var treble_count = 0.0;
    var presence_count = 0.0;

    // Process each frequency bin
    for (var i = 1u; i < fft_size / 2u; i = i + 1u) { // Skip DC component
        let freq = bin_to_frequency(i, fft_size);
        let mag = magnitude(fft_data[i]);

        if (freq <= SUB_BASS_MAX) {
            sub_bass = sub_bass + mag;
            sub_bass_count = sub_bass_count + 1.0;
        } else if (freq <= BASS_MAX) {
            bass = bass + mag;
            bass_count = bass_count + 1.0;
        } else if (freq <= MID_MAX) {
            mid = mid + mag;
            mid_count = mid_count + 1.0;
        } else if (freq <= TREBLE_MAX) {
            treble = treble + mag;
            treble_count = treble_count + 1.0;
        } else if (freq <= PRESENCE_MAX) {
            presence = presence + mag;
            presence_count = presence_count + 1.0;
        }
    }

    // Normalize by count and scale for visualization (raw values)
    features[0] = sub_bass / max(sub_bass_count, 1.0); // sub_bass
    features[1] = bass / max(bass_count, 1.0); // bass
    features[2] = mid / max(mid_count, 1.0); // mid
    features[3] = treble / max(treble_count, 1.0); // treble
    features[4] = presence / max(presence_count, 1.0); // presence
}

// Calculate spectral centroid (brightness)
fn calculate_spectral_centroid() {
    let fft_size = arrayLength(&fft_data);
    var weighted_sum = 0.0;
    var magnitude_sum = 0.0;

    for (var i = 1u; i < fft_size / 2u; i = i + 1u) {
        let freq = bin_to_frequency(i, fft_size);
        let mag = magnitude(fft_data[i]);

        weighted_sum = weighted_sum + freq * mag;
        magnitude_sum = magnitude_sum + mag;
    }

    if (magnitude_sum > 0.0) {
        features[5] = weighted_sum / magnitude_sum; // spectral_centroid (raw Hz)
    } else {
        features[5] = 0.0;
    }
}

// Calculate spectral rolloff (high frequency content)
fn calculate_spectral_rolloff() {
    let fft_size = arrayLength(&fft_data);
    var total_energy = 0.0;

    // Calculate total energy
    for (var i = 1u; i < fft_size / 2u; i = i + 1u) {
        let mag = magnitude(fft_data[i]);
        total_energy = total_energy + mag * mag;
    }

    let threshold = total_energy * 0.85; // 85% of energy
    var cumulative_energy = 0.0;

    // Find frequency where 85% of energy is contained
    for (var i = 1u; i < fft_size / 2u; i = i + 1u) {
        let mag = magnitude(fft_data[i]);
        cumulative_energy = cumulative_energy + mag * mag;

        if (cumulative_energy >= threshold) {
            features[6] = bin_to_frequency(i, fft_size); // spectral_rolloff (raw Hz)
            return;
        }
    }

    features[6] = SAMPLE_RATE / 2.0; // Nyquist frequency as fallback
}

// Calculate spectral flux (rate of change)
fn calculate_spectral_flux() {
    // For simplicity, calculate variance across frequency bins
    let fft_size = arrayLength(&fft_data);
    var mean_magnitude = 0.0;
    var count = 0.0;

    // Calculate mean
    for (var i = 1u; i < fft_size / 2u; i = i + 1u) {
        mean_magnitude = mean_magnitude + magnitude(fft_data[i]);
        count = count + 1.0;
    }
    mean_magnitude = mean_magnitude / count;

    // Calculate variance
    var variance = 0.0;
    for (var i = 1u; i < fft_size / 2u; i = i + 1u) {
        let diff = magnitude(fft_data[i]) - mean_magnitude;
        variance = variance + diff * diff;
    }

    features[7] = sqrt(variance / count); // spectral_flux (raw variance)
}

// Calculate zero crossing rate approximation
fn calculate_zero_crossing_rate() {
    // Approximate from high frequency content
    let fft_size = arrayLength(&fft_data);
    var high_freq_energy = 0.0;
    var total_energy = 0.0;

    for (var i = 1u; i < fft_size / 2u; i = i + 1u) {
        let freq = bin_to_frequency(i, fft_size);
        let mag = magnitude(fft_data[i]);
        let energy = mag * mag;

        total_energy = total_energy + energy;
        if (freq > 2000.0) {
            high_freq_energy = high_freq_energy + energy;
        }
    }

    if (total_energy > 0.0) {
        features[8] = high_freq_energy / total_energy; // zero_crossing_rate (raw ratio)
    } else {
        features[8] = 0.0;
    }
}

// Calculate onset strength (energy increase)
fn calculate_onset_strength() {
    // Use high frequency energy as proxy for onset detection
    let fft_size = arrayLength(&fft_data);
    var onset_energy = 0.0;

    for (var i = 1u; i < fft_size / 4u; i = i + 1u) { // Focus on lower frequencies for onsets
        let mag = magnitude(fft_data[i]);
        onset_energy = onset_energy + mag * mag;
    }

    features[9] = onset_energy; // onset_strength (raw energy)
}

// Calculate volume (RMS)
fn calculate_volume() {
    let fft_size = arrayLength(&fft_data);
    var energy = 0.0;

    for (var i = 1u; i < fft_size / 2u; i = i + 1u) {
        let mag = magnitude(fft_data[i]);
        energy = energy + mag * mag;
    }

    features[12] = sqrt(energy / f32(fft_size)); // volume (raw RMS)
}

// Calculate dynamic range
fn calculate_dynamic_range() {
    let fft_size = arrayLength(&fft_data);
    var max_magnitude = 0.0;
    var min_magnitude = 999999.0;

    for (var i = 1u; i < fft_size / 2u; i = i + 1u) {
        let mag = magnitude(fft_data[i]);
        max_magnitude = max(max_magnitude, mag);
        min_magnitude = min(min_magnitude, mag);
    }

    if (max_magnitude > 0.0) {
        features[13] = clamp((max_magnitude - min_magnitude) / max_magnitude, 0.0, 1.0); // dynamic_range
    } else {
        features[13] = 0.0;
    }
}

// Calculate pitch confidence (harmonic vs inharmonic content)
fn calculate_pitch_confidence() {
    // Look for harmonic peaks in the spectrum
    let fft_size = arrayLength(&fft_data);
    var harmonic_energy = 0.0;
    var total_energy = 0.0;

    // Simple harmonic detection - look for peaks at regular intervals
    for (var i = 1u; i < fft_size / 8u; i = i + 1u) {
        let mag = magnitude(fft_data[i]);
        total_energy = total_energy + mag * mag;

        // Check if this bin has nearby harmonics
        let fundamental_freq = bin_to_frequency(i, fft_size);
        if (fundamental_freq > 80.0 && fundamental_freq < 2000.0) {
            // Look for 2nd harmonic
            let harmonic_bin = u32(round(f32(i) * 2.0));
            if (harmonic_bin < fft_size / 2u) {
                let harmonic_mag = magnitude(fft_data[harmonic_bin]);
                if (harmonic_mag > mag * 0.3) { // Harmonic should be reasonably strong
                    harmonic_energy = harmonic_energy + mag * mag;
                }
            }
        }
    }

    if (total_energy > 0.0) {
        features[14] = clamp(harmonic_energy / total_energy * 3.0, 0.0, 1.0); // pitch_confidence
    } else {
        features[14] = 0.0;
    }
}

@compute @workgroup_size(1)
fn main() {
    // Initialize all features to 0
    for (var i = 0u; i < 16u; i = i + 1u) {
        features[i] = 0.0;
    }

    // Extract all audio features
    extract_frequency_bands();
    calculate_spectral_centroid();
    calculate_spectral_rolloff();
    calculate_spectral_flux();
    calculate_zero_crossing_rate();
    calculate_onset_strength();
    calculate_volume();
    calculate_dynamic_range();
    calculate_pitch_confidence();
}