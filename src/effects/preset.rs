use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizerPreset {
    pub name: String,
    pub shader_name: String,
    pub parameters: PresetParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetParameters {
    pub plasma_intensity: f32,
    pub bar_sensitivity: f32,
    pub color_shift_speed: f32,
    pub beat_response: f32,
    pub background_dim: f32,
}

impl Default for PresetParameters {
    fn default() -> Self {
        Self {
            plasma_intensity: 1.0,
            bar_sensitivity: 1.0,
            color_shift_speed: 1.0,
            beat_response: 1.0,
            background_dim: 0.1,
        }
    }
}

pub struct PresetManager {
    presets: Vec<VisualizerPreset>,
    current_preset: usize,
}

impl PresetManager {
    pub fn new() -> Self {
        let presets = vec![
            VisualizerPreset {
                name: "Plasma Dreams".to_string(),
                shader_name: "visualizer".to_string(),
                parameters: PresetParameters {
                    plasma_intensity: 1.2,
                    bar_sensitivity: 0.8,
                    color_shift_speed: 0.7,
                    beat_response: 1.1,
                    background_dim: 0.05,
                },
            },
            VisualizerPreset {
                name: "Spectrum Bars".to_string(),
                shader_name: "visualizer".to_string(),
                parameters: PresetParameters {
                    plasma_intensity: 0.3,
                    bar_sensitivity: 1.5,
                    color_shift_speed: 0.4,
                    beat_response: 0.8,
                    background_dim: 0.2,
                },
            },
            VisualizerPreset {
                name: "Radial Waves".to_string(),
                shader_name: "visualizer".to_string(),
                parameters: PresetParameters {
                    plasma_intensity: 0.6,
                    bar_sensitivity: 0.4,
                    color_shift_speed: 1.3,
                    beat_response: 1.4,
                    background_dim: 0.1,
                },
            },
            VisualizerPreset {
                name: "Beat Sync".to_string(),
                shader_name: "visualizer".to_string(),
                parameters: PresetParameters {
                    plasma_intensity: 0.9,
                    bar_sensitivity: 1.2,
                    color_shift_speed: 0.8,
                    beat_response: 2.0,
                    background_dim: 0.15,
                },
            },
        ];

        Self {
            presets,
            current_preset: 0,
        }
    }

    pub fn get_current_preset(&self) -> &VisualizerPreset {
        &self.presets[self.current_preset]
    }

    pub fn set_current_preset(&mut self, index: usize) {
        if index < self.presets.len() {
            self.current_preset = index;
        }
    }

    pub fn get_presets(&self) -> &[VisualizerPreset] {
        &self.presets
    }

    pub fn current_preset_index(&self) -> usize {
        self.current_preset
    }
}