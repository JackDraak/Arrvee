use crate::audio::AudioFrame;
use std::collections::HashMap;

/// Psychedelic Effect Manager - Handles dynamic effect selection and blending
/// Based on musical characteristics and user preferences
pub struct PsychedelicManager {
    /// Current effect weights (0.0 to 1.0)
    effect_weights: HashMap<String, f32>,

    /// Transition speeds for each effect
    transition_speeds: HashMap<String, f32>,

    /// Target weights for smooth transitions
    target_weights: HashMap<String, f32>,

    /// Effect intensity scaling factors
    intensity_scalers: HashMap<String, f32>,

    /// Time accumulator for animations
    time: f32,

    /// Configuration
    config: EffectConfig,
}

#[derive(Clone)]
pub struct EffectConfig {
    /// How aggressively effects respond to musical changes (0.0 to 1.0)
    pub responsiveness: f32,

    /// Base intensity multiplier (0.0 to 2.0)
    pub base_intensity: f32,

    /// How much beats influence effect switching (0.0 to 1.0)
    pub beat_sensitivity: f32,

    /// Smoothing factor for transitions (0.0 to 1.0, higher = smoother)
    pub transition_smoothing: f32,

    /// Enable automatic effect switching based on music
    pub auto_switch: bool,

    /// Manual effect override (None for auto, Some(effect_name) for manual)
    pub manual_override: Option<String>,
}

impl Default for EffectConfig {
    fn default() -> Self {
        Self {
            responsiveness: 1.0,  // Maximum responsiveness
            base_intensity: 1.0,
            beat_sensitivity: 0.9, // Higher beat sensitivity
            transition_smoothing: 0.5, // Less smoothing = more responsive
            auto_switch: true,
            manual_override: None,
        }
    }
}

impl PsychedelicManager {
    pub fn new() -> Self {
        let mut effect_weights = HashMap::new();
        let mut transition_speeds = HashMap::new();
        let mut target_weights = HashMap::new();
        let mut intensity_scalers = HashMap::new();

        // Initialize all effects
        let effects = vec![
            "llama_plasma",
            "geometric_kaleidoscope",
            "psychedelic_tunnel",
            "particle_swarm",
            "fractal_madness",
            "spectralizer_bars",
            "parametric_waves"
        ];

        for effect in effects {
            effect_weights.insert(effect.to_string(), 0.0);
            transition_speeds.insert(effect.to_string(), 4.0); // Faster transitions for real-time response
            target_weights.insert(effect.to_string(), 0.0);
            intensity_scalers.insert(effect.to_string(), 1.0);
        }

        // Start with plasma as the base effect
        effect_weights.insert("llama_plasma".to_string(), 0.3);
        target_weights.insert("llama_plasma".to_string(), 0.3);

        Self {
            effect_weights,
            transition_speeds,
            target_weights,
            intensity_scalers,
            time: 0.0,
            config: EffectConfig::default(),
        }
    }

    pub fn update(&mut self, delta_time: f32, audio_frame: &AudioFrame) {
        self.time += delta_time;

        if self.config.auto_switch && self.config.manual_override.is_none() {
            self.analyze_and_set_targets(audio_frame);
        }

        self.update_transitions(delta_time);
        self.update_intensity_scalers(audio_frame);
    }

    fn analyze_and_set_targets(&mut self, audio_frame: &AudioFrame) {
        // Clear all targets first
        for (_, weight) in self.target_weights.iter_mut() {
            *weight = 0.0;
        }

        // Base layer - always have some plasma
        let base_plasma = 0.1 + audio_frame.volume * 0.2;
        *self.target_weights.get_mut("llama_plasma").unwrap() = base_plasma;

        // Plasma dominance during bass (much more responsive thresholds)
        let bass_energy = audio_frame.frequency_bands.bass + audio_frame.frequency_bands.sub_bass;
        if bass_energy > 0.1 { // Much lower threshold for better response
            let plasma_boost = (bass_energy - 0.1) * 2.0 * self.config.responsiveness; // Stronger response
            *self.target_weights.get_mut("llama_plasma").unwrap() += plasma_boost;
        }

        // Kaleidoscope for harmonic content (much more responsive)
        if audio_frame.pitch_confidence > 0.2 && audio_frame.frequency_bands.mid > 0.05 { // Much lower thresholds
            let harmonic_strength = audio_frame.pitch_confidence * audio_frame.frequency_bands.mid;
            let kaleidoscope_weight = harmonic_strength * 1.5 * self.config.responsiveness; // Stronger response
            *self.target_weights.get_mut("geometric_kaleidoscope").unwrap() = kaleidoscope_weight;
        }

        // Tunnel for bright, present sounds
        if audio_frame.spectral_rolloff > 0.5 && audio_frame.frequency_bands.presence > 0.3 {
            let brightness = audio_frame.spectral_rolloff * audio_frame.frequency_bands.presence;
            let tunnel_weight = brightness * 0.6 * self.config.responsiveness;
            *self.target_weights.get_mut("psychedelic_tunnel").unwrap() = tunnel_weight;
        }

        // Particle swarm for chaotic, attack-heavy music
        if audio_frame.onset_strength > 0.3 || audio_frame.zero_crossing_rate > 0.4 {
            let chaos_level = (audio_frame.onset_strength + audio_frame.zero_crossing_rate) * 0.5;
            let particle_weight = chaos_level * 0.7 * self.config.responsiveness;
            *self.target_weights.get_mut("particle_swarm").unwrap() = particle_weight;
        }

        // Fractal madness for dynamic, evolving sounds
        if audio_frame.dynamic_range > 0.3 && audio_frame.spectral_flux > 0.2 {
            let evolution = audio_frame.dynamic_range * audio_frame.spectral_flux;
            let fractal_weight = evolution * 0.5 * self.config.responsiveness;
            *self.target_weights.get_mut("fractal_madness").unwrap() = fractal_weight;
        }

        // Spectralizer for when we want to see frequency content clearly
        let spectral_activity = audio_frame.frequency_bands.bass + audio_frame.frequency_bands.mid +
                               audio_frame.frequency_bands.treble + audio_frame.frequency_bands.presence;
        if spectral_activity > 0.4 && audio_frame.volume > 0.1 {
            let spectralizer_weight = spectral_activity * 0.3 * self.config.responsiveness;
            *self.target_weights.get_mut("spectralizer_bars").unwrap() = spectralizer_weight;
        }

        // Parametric waves for mathematically complex, parametric music
        // Activates for high pitch confidence with dynamic spectral content
        if audio_frame.pitch_confidence > 0.3 && audio_frame.spectral_flux > 0.15 {
            let mathematical_complexity = audio_frame.pitch_confidence * audio_frame.spectral_flux;
            let parametric_weight = mathematical_complexity * 1.2 * self.config.responsiveness;
            *self.target_weights.get_mut("parametric_waves").unwrap() = parametric_weight;
        }

        // Beat-driven effect boosting
        if audio_frame.beat_strength > 0.5 {
            let beat_boost = (audio_frame.beat_strength - 0.5) * 2.0 * self.config.beat_sensitivity;

            // Find the currently dominant effect and boost it
            let dominant_effect_name = self.target_weights.iter()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .map(|(name, _)| name.clone());

            if let Some(effect_name) = dominant_effect_name {
                if let Some(weight) = self.target_weights.get_mut(&effect_name) {
                    *weight += beat_boost;
                }
            }
        }

        // Clamp all weights to reasonable ranges
        for (_, weight) in self.target_weights.iter_mut() {
            *weight = weight.clamp(0.0, 1.5);
        }
    }

    fn update_transitions(&mut self, delta_time: f32) {
        for (effect_name, current_weight) in self.effect_weights.iter_mut() {
            if let Some(target_weight) = self.target_weights.get(effect_name) {
                if let Some(transition_speed) = self.transition_speeds.get(effect_name) {
                    let diff = target_weight - *current_weight;

                    // Enhanced smoothing with exponential decay
                    let smoothing_factor = 1.0 - (-transition_speed * delta_time).exp();
                    let change = diff * smoothing_factor * self.config.transition_smoothing;

                    *current_weight += change;
                    *current_weight = current_weight.clamp(0.0, 1.5); // Lower ceiling for smoother visuals
                }
            }
        }
    }

    fn update_intensity_scalers(&mut self, audio_frame: &AudioFrame) {
        // Global intensity based on volume and beat strength
        let global_intensity = self.config.base_intensity *
            (0.7 + audio_frame.volume * 0.3) *
            (1.0 + audio_frame.beat_strength * 0.4);

        // Per-effect intensity adjustments
        for (effect_name, scaler) in self.intensity_scalers.iter_mut() {
            *scaler = global_intensity;

            // Effect-specific intensity modulations
            match effect_name.as_str() {
                "llama_plasma" => {
                    *scaler *= 1.0 + audio_frame.spectral_flux * 0.3;
                }
                "geometric_kaleidoscope" => {
                    let bpm_factor = (audio_frame.estimated_bpm / 120.0).clamp(0.5, 2.0);
                    *scaler *= bpm_factor;
                }
                "psychedelic_tunnel" => {
                    *scaler *= 1.0 + audio_frame.frequency_bands.presence * 0.4;
                }
                "particle_swarm" => {
                    *scaler *= 1.0 + audio_frame.onset_strength * 0.5;
                }
                "fractal_madness" => {
                    *scaler *= 1.0 + audio_frame.dynamic_range * 0.3;
                }
                _ => {}
            }
        }
    }

    /// Get current effect weights for the shader
    pub fn get_effect_weights(&self) -> &HashMap<String, f32> {
        &self.effect_weights
    }

    /// Get current intensity scalers for the shader
    pub fn get_intensity_scalers(&self) -> &HashMap<String, f32> {
        &self.intensity_scalers
    }

    /// Manually override effect selection
    pub fn set_manual_effect(&mut self, effect_name: Option<String>) {
        self.config.manual_override = effect_name;

        if let Some(effect) = &self.config.manual_override {
            // Set the manual effect to full weight, others to zero
            for (name, target) in self.target_weights.iter_mut() {
                *target = if name == effect { 1.0 } else { 0.0 };
            }
        }
    }

    /// Get configuration for external modification
    pub fn config_mut(&mut self) -> &mut EffectConfig {
        &mut self.config
    }

    pub fn config(&self) -> &EffectConfig {
        &self.config
    }

    /// Get a summary of current effect states
    pub fn get_debug_info(&self) -> String {
        let mut info = String::new();
        info.push_str("Psychedelic Effects Status:\n");

        for (effect, weight) in &self.effect_weights {
            if *weight > 0.01 {
                info.push_str(&format!("  {}: {:.2}\n", effect, weight));
            }
        }

        if let Some(manual) = &self.config.manual_override {
            info.push_str(&format!("Manual Override: {}\n", manual));
        } else {
            info.push_str("Auto Mode\n");
        }

        info
    }
}