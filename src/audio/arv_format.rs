use anyhow::Result;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use super::prescan::{PrescanData, PrescanFrame, FileInfo, AnalysisStatistics};

/// Arrvee Audio-Visual (.arv) - Proprietary binary format for ultra-efficient prescan data
///
/// Format specification:
/// - Magic bytes: "ARVV" (4 bytes)
/// - Version: u8 (1 byte)
/// - Header: FileInfo + Statistics (variable)
/// - Frame count: u32 (4 bytes)
/// - Frames: Packed binary data (16 bytes per frame)
///
/// Per-frame data (16 bytes total):
/// - 5x frequency bands: u16 (0-65535 maps to 0.0-1.0) = 10 bytes
/// - 3x spectral features: u16 = 6 bytes
/// - Beat data: u8 (packed bits) + u8 (beat_strength scaled) = 2 bytes
/// - Reserved: 2 bytes for future expansion
///
/// Total compression: ~85% smaller than JSON

#[allow(dead_code)]
const MAGIC_BYTES: &[u8; 4] = b"ARVV";
#[allow(dead_code)]
const FORMAT_VERSION: u8 = 1;
#[allow(dead_code)]
const BYTES_PER_FRAME: usize = 16;

#[allow(dead_code)]
#[repr(packed)]
#[derive(Clone, Copy)]
struct PackedFrame {
    // Frequency bands (5x u16 = 10 bytes)
    bass: u16,
    mid: u16,
    treble: u16,
    sub_bass: u16,
    presence: u16,

    // Key spectral features (3x u16 = 6 bytes)
    spectral_centroid: u16,
    pitch_confidence: u16,
    onset_strength: u16,

    // Beat/rhythm data (2 bytes)
    beat_data: u8,    // bit 0: beat_detected, bits 1-7: reserved
    beat_strength: u8, // 0-255 mapped from 0.0-5.0

    // Reserved for future features
    reserved: u16,
}

impl PackedFrame {
    /// Convert normalized float (0.0-1.0) to u16 (0-65535)
    fn pack_float(value: f32) -> u16 {
        (value.clamp(0.0, 1.0) * 65535.0) as u16
    }

    /// Convert u16 (0-65535) back to normalized float (0.0-1.0)
    fn unpack_float(value: u16) -> f32 {
        value as f32 / 65535.0
    }

    /// Pack beat strength (0.0-5.0 range) into u8
    fn pack_beat_strength(strength: f32) -> u8 {
        (strength.clamp(0.0, 5.0) * 51.0) as u8 // 255/5 = 51
    }

    /// Unpack beat strength from u8 to 0.0-5.0 range
    fn unpack_beat_strength(value: u8) -> f32 {
        value as f32 / 51.0
    }

    fn from_prescan_frame(frame: &PrescanFrame, _timestamp: f32) -> Self {
        Self {
            bass: Self::pack_float(frame.frequency_bands.bass),
            mid: Self::pack_float(frame.frequency_bands.mid),
            treble: Self::pack_float(frame.frequency_bands.treble),
            sub_bass: Self::pack_float(frame.frequency_bands.sub_bass),
            presence: Self::pack_float(frame.frequency_bands.presence),

            spectral_centroid: Self::pack_float(frame.spectral_centroid),
            pitch_confidence: Self::pack_float(frame.pitch_confidence),
            onset_strength: Self::pack_float(frame.onset_strength),

            beat_data: if frame.beat_detected { 1 } else { 0 },
            beat_strength: Self::pack_beat_strength(frame.beat_strength),

            reserved: 0,
        }
    }

    fn to_prescan_frame(&self, timestamp: f32, estimated_bpm: f32) -> PrescanFrame {
        PrescanFrame {
            timestamp,
            frequency_bands: super::FrequencyBands {
                bass: Self::unpack_float(self.bass),
                mid: Self::unpack_float(self.mid),
                treble: Self::unpack_float(self.treble),
                sub_bass: Self::unpack_float(self.sub_bass),
                presence: Self::unpack_float(self.presence),
            },
            beat_detected: (self.beat_data & 1) != 0,
            beat_strength: Self::unpack_beat_strength(self.beat_strength),
            estimated_bpm,
            spectral_centroid: Self::unpack_float(self.spectral_centroid),
            spectral_rolloff: 0.0, // Not stored to save space, derived if needed
            pitch_confidence: Self::unpack_float(self.pitch_confidence),
            zero_crossing_rate: 0.0, // Not stored, less critical for visualization
            spectral_flux: 0.0, // Not stored, less critical
            onset_strength: Self::unpack_float(self.onset_strength),
            dynamic_range: 0.0, // Derived from volume variance if needed
            volume: 0.0, // Not stored, derived from frequency bands if needed
        }
    }
}

#[allow(dead_code)]
pub struct ArvFormat;

impl ArvFormat {
    /// Save prescan data in compact ARV binary format
    pub fn save_arv<P: AsRef<std::path::Path>>(prescan_data: &PrescanData, path: P) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // Write magic bytes and version
        writer.write_all(MAGIC_BYTES)?;
        writer.write_all(&[FORMAT_VERSION])?;

        // Write file info as JSON (small, infrequent)
        let file_info_json = serde_json::to_string(&prescan_data.file_info)?;
        let file_info_len = file_info_json.len() as u32;
        writer.write_all(&file_info_len.to_le_bytes())?;
        writer.write_all(file_info_json.as_bytes())?;

        // Write statistics as JSON (small, infrequent)
        let stats_json = serde_json::to_string(&prescan_data.statistics)?;
        let stats_len = stats_json.len() as u32;
        writer.write_all(&stats_len.to_le_bytes())?;
        writer.write_all(stats_json.as_bytes())?;

        // Write frame count
        let frame_count = prescan_data.frames.len() as u32;
        writer.write_all(&frame_count.to_le_bytes())?;

        // Write packed frames
        for frame in &prescan_data.frames {
            let packed = PackedFrame::from_prescan_frame(frame, frame.timestamp);
            let bytes = unsafe {
                std::slice::from_raw_parts(
                    &packed as *const PackedFrame as *const u8,
                    BYTES_PER_FRAME
                )
            };
            writer.write_all(bytes)?;
        }

        Ok(())
    }

    /// Load prescan data from ARV binary format
    pub fn load_arv<P: AsRef<std::path::Path>>(path: P) -> Result<PrescanData> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        // Verify magic bytes
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if &magic != MAGIC_BYTES {
            return Err(anyhow::anyhow!("Invalid ARV file: bad magic bytes"));
        }

        // Read version
        let mut version = [0u8; 1];
        reader.read_exact(&mut version)?;
        if version[0] != FORMAT_VERSION {
            return Err(anyhow::anyhow!("Unsupported ARV version: {}", version[0]));
        }

        // Read file info
        let mut len_bytes = [0u8; 4];
        reader.read_exact(&mut len_bytes)?;
        let file_info_len = u32::from_le_bytes(len_bytes) as usize;
        let mut file_info_json = vec![0u8; file_info_len];
        reader.read_exact(&mut file_info_json)?;
        let file_info: FileInfo = serde_json::from_slice(&file_info_json)?;

        // Read statistics
        reader.read_exact(&mut len_bytes)?;
        let stats_len = u32::from_le_bytes(len_bytes) as usize;
        let mut stats_json = vec![0u8; stats_len];
        reader.read_exact(&mut stats_json)?;
        let statistics: AnalysisStatistics = serde_json::from_slice(&stats_json)?;

        // Read frame count
        reader.read_exact(&mut len_bytes)?;
        let frame_count = u32::from_le_bytes(len_bytes) as usize;

        // Read packed frames
        let mut frames = Vec::with_capacity(frame_count);
        let mut packed_data = vec![0u8; BYTES_PER_FRAME];

        for i in 0..frame_count {
            reader.read_exact(&mut packed_data)?;

            let packed_frame = unsafe {
                *(packed_data.as_ptr() as *const PackedFrame)
            };

            // Calculate timestamp from frame index
            let timestamp = i as f32 / file_info.frame_rate;

            // Use BPM from statistics (more efficient than storing per-frame)
            let estimated_bpm = statistics.average_bpm;

            let frame = packed_frame.to_prescan_frame(timestamp, estimated_bpm);
            frames.push(frame);
        }

        Ok(PrescanData {
            file_info,
            frames,
            statistics,
        })
    }

    /// Get compression ratio compared to JSON
    pub fn compression_ratio(arv_size: u64, json_size: u64) -> f64 {
        1.0 - (arv_size as f64 / json_size as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_float_packing() {
        // Test edge cases
        assert_eq!(PackedFrame::pack_float(0.0), 0);
        assert_eq!(PackedFrame::pack_float(1.0), 65535);
        assert_eq!(PackedFrame::pack_float(0.5), 32767);

        // Test round-trip precision
        let original = 0.12345;
        let packed = PackedFrame::pack_float(original);
        let unpacked = PackedFrame::unpack_float(packed);
        assert!((original - unpacked).abs() < 0.0002); // ~16-bit precision
    }

    #[test]
    fn test_beat_strength_packing() {
        assert_eq!(PackedFrame::pack_beat_strength(0.0), 0);
        assert_eq!(PackedFrame::pack_beat_strength(5.0), 255);

        let original = 2.5;
        let packed = PackedFrame::pack_beat_strength(original);
        let unpacked = PackedFrame::unpack_beat_strength(packed);
        assert!((original - unpacked).abs() < 0.1); // ~8-bit precision for beat strength
    }
}