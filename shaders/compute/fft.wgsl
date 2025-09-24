// GPU-accelerated FFT compute shader using Cooley-Tukey algorithm
// Optimized for real-time audio analysis

@group(0) @binding(0) var<storage, read> audio_input: array<f32>;
@group(0) @binding(1) var<storage, read_write> fft_output: array<vec2<f32>>; // Complex numbers

var<workgroup> shared_data: array<vec2<f32>, 64>;

// Twiddle factors for FFT (pre-computed)
fn get_twiddle_factor(k: u32, n: u32) -> vec2<f32> {
    let angle = -2.0 * 3.14159265359 * f32(k) / f32(n);
    return vec2<f32>(cos(angle), sin(angle));
}

// Apply Hann window for better frequency resolution
fn hann_window(index: u32, size: u32) -> f32 {
    let n = f32(index);
    let N = f32(size);
    return 0.5 * (1.0 - cos(2.0 * 3.14159265359 * n / (N - 1.0)));
}

// Bit-reverse for FFT reordering
fn bit_reverse(input_x: u32, bits: u32) -> u32 {
    var x = input_x;
    var result = 0u;
    for (var i = 0u; i < bits; i = i + 1u) {
        result = (result << 1u) | (x & 1u);
        x = x >> 1u;
    }
    return result;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    let total_size = arrayLength(&audio_input);

    if (index >= total_size) {
        return;
    }

    // Apply windowing function and convert to complex
    let windowed_sample = audio_input[index] * hann_window(index, total_size);
    let complex_sample = vec2<f32>(windowed_sample, 0.0);

    // Bit-reverse ordering for FFT
    let bits = u32(log2(f32(total_size)));
    let reversed_index = bit_reverse(index, bits);

    // Store in shared memory for local processing
    let local_index = global_id.x % 64u;
    shared_data[local_index] = complex_sample;

    workgroupBarrier();

    // Cooley-Tukey FFT algorithm
    var n = 2u;
    while (n <= 64u) {
        let half_n = n / 2u;
        let group_size = 64u / n;
        let group_id = local_index / n;
        let element_id = local_index % n;

        if (element_id < half_n) {
            let twiddle = get_twiddle_factor(element_id, n);
            let base_index = group_id * n;

            let even_index = base_index + element_id;
            let odd_index = base_index + element_id + half_n;

            let even_val = shared_data[even_index];
            let odd_val = shared_data[odd_index];

            // Complex multiplication: (a + bi) * (c + di) = (ac - bd) + (ad + bc)i
            let twiddle_odd = vec2<f32>(
                twiddle.x * odd_val.x - twiddle.y * odd_val.y,
                twiddle.x * odd_val.y + twiddle.y * odd_val.x
            );

            shared_data[even_index] = even_val + twiddle_odd;
            shared_data[odd_index] = even_val - twiddle_odd;
        }

        workgroupBarrier();
        n = n * 2u;
    }

    // Write back to global memory
    if (reversed_index < arrayLength(&fft_output)) {
        fft_output[reversed_index] = shared_data[local_index];
    }
}