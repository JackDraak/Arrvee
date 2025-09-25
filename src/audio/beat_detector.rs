use super::FrequencyBands;
use std::collections::VecDeque;

#[allow(dead_code)]
pub struct BeatDetector {
    sample_rate: f32,
    bass_history: VecDeque<f32>,
    kick_history: VecDeque<f32>,
    last_beat_time: f32,
    time_counter: f32,
    min_beat_interval: f32,
    history_size: usize,
}

impl BeatDetector {
    pub fn new(sample_rate: f32) -> Self {
        let history_size = (sample_rate * 0.5) as usize / 1024;

        Self {
            sample_rate,
            bass_history: VecDeque::with_capacity(history_size),
            kick_history: VecDeque::with_capacity(history_size),
            last_beat_time: 0.0,
            time_counter: 0.0,
            min_beat_interval: 0.3,
            history_size,
        }
    }

    pub fn detect_beat(&mut self, bands: &FrequencyBands) -> (bool, f32) {
        self.time_counter += 1024.0 / self.sample_rate;

        let bass_energy = bands.bass + bands.sub_bass;
        let kick_energy = bands.bass * 1.5 + bands.sub_bass * 0.8;

        self.bass_history.push_back(bass_energy);
        self.kick_history.push_back(kick_energy);

        if self.bass_history.len() > self.history_size {
            self.bass_history.pop_front();
        }
        if self.kick_history.len() > self.history_size {
            self.kick_history.pop_front();
        }

        if self.bass_history.len() < 10 {
            return (false, 0.0);
        }

        let bass_avg = self.bass_history.iter().sum::<f32>() / self.bass_history.len() as f32;
        let kick_avg = self.kick_history.iter().sum::<f32>() / self.kick_history.len() as f32;

        let bass_variance = self.calculate_variance(&self.bass_history, bass_avg);
        let kick_variance = self.calculate_variance(&self.kick_history, kick_avg);

        let bass_threshold = bass_avg + bass_variance.sqrt() * 1.5;
        let kick_threshold = kick_avg + kick_variance.sqrt() * 2.0;

        let time_since_last_beat = self.time_counter - self.last_beat_time;
        let can_beat = time_since_last_beat > self.min_beat_interval;

        let bass_beat = bass_energy > bass_threshold && bass_energy > 0.01;
        let kick_beat = kick_energy > kick_threshold && kick_energy > 0.015;

        let beat_detected = can_beat && (bass_beat || kick_beat);
        let beat_strength = if beat_detected {
            ((bass_energy / bass_avg.max(0.001)) + (kick_energy / kick_avg.max(0.001))) / 2.0
        } else {
            0.0
        };

        if beat_detected {
            self.last_beat_time = self.time_counter;
        }

        (beat_detected, beat_strength.min(5.0))
    }

    fn calculate_variance(&self, data: &VecDeque<f32>, mean: f32) -> f32 {
        if data.len() < 2 {
            return 0.0;
        }

        let sum_sq_diff: f32 = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum();

        sum_sq_diff / (data.len() - 1) as f32
    }
}