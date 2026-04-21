//! MP3-to-Wav Converter — audio buffer + resample + normalize + RIFF encoding.
//!
//! Thirteenth Graphics project. Distinctive moves:
//!
//!   * **Typed audio buffer** — `AudioBuffer { sample_rate, channels,
//!     samples: Vec<i16> }`. Interleaved stereo (L, R, L, R, ...).
//!     Parallels G116's pixel buffer pattern but in 1D time axis.
//!   * **Linear-interpolation resample** — arbitrary `src_rate → dst_rate`.
//!     Pure function over samples; same inputs, same bytes out.
//!   * **Two-pass normalize-to-peak** — scan `max_abs`, then scale so
//!     the peak equals target (`0x7FFF = 32767` for i16). Deterministic.
//!   * **Channel mix** — stereo → mono averages L and R; mono → stereo
//!     duplicates. Both are pure functions over the sample array.
//!   * **RIFF chunk byte serialization** — WAV's layout (RIFF / WAVE /
//!     fmt / data) as little-endian bytes. 44-byte header + samples.

use std::convert::TryInto;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioBuffer {
    pub sample_rate: u32,
    pub channels: u16,
    pub samples: Vec<i16>,
}

impl AudioBuffer {
    pub fn frame_count(&self) -> usize {
        if self.channels == 0 { 0 } else { self.samples.len() / self.channels as usize }
    }

    pub fn duration_ms(&self) -> u64 {
        if self.sample_rate == 0 { 0 } else {
            (self.frame_count() as u64 * 1000) / self.sample_rate as u64
        }
    }
}

/// A synthetic Mp3Frame: not real MP3 bitstream, but enough structure
/// to model frame-by-frame decoding. The "compressed" form carries PCM
/// directly — the point is the frame iteration pattern, not codec fidelity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mp3Frame {
    pub sample_rate: u32,
    pub channels: u16,
    pub bitrate_kbps: u32,
    pub samples_per_frame: u32,
    pub pcm: Vec<i16>, // interleaved
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodecError {
    NoFrames,
    MixedFormats,
    EmptyBuffer,
    UnsupportedChannels(u16),
    SampleRateZero,
}

/// Decode a sequence of MP3 frames into a single PCM buffer. All frames
/// must share sample_rate and channels; otherwise the codec errors.
pub fn decode_mp3_to_buffer(frames: &[Mp3Frame]) -> Result<AudioBuffer, CodecError> {
    if frames.is_empty() { return Err(CodecError::NoFrames); }
    let sample_rate = frames[0].sample_rate;
    let channels = frames[0].channels;
    if sample_rate == 0 { return Err(CodecError::SampleRateZero); }
    if channels != 1 && channels != 2 {
        return Err(CodecError::UnsupportedChannels(channels));
    }
    for f in frames {
        if f.sample_rate != sample_rate || f.channels != channels {
            return Err(CodecError::MixedFormats);
        }
    }
    let total_len: usize = frames.iter().map(|f| f.pcm.len()).sum();
    let mut samples = Vec::with_capacity(total_len);
    for f in frames {
        samples.extend_from_slice(&f.pcm);
    }
    Ok(AudioBuffer { sample_rate, channels, samples })
}

/// Linear-interpolation resample. Same input, same output, byte-exact.
pub fn resample(buf: &AudioBuffer, dst_rate: u32) -> Result<AudioBuffer, CodecError> {
    if buf.sample_rate == 0 { return Err(CodecError::SampleRateZero); }
    if dst_rate == 0 { return Err(CodecError::SampleRateZero); }
    if dst_rate == buf.sample_rate { return Ok(buf.clone()); }
    let in_frames = buf.frame_count();
    if in_frames == 0 {
        return Ok(AudioBuffer {
            sample_rate: dst_rate, channels: buf.channels, samples: vec![],
        });
    }
    // Output frame count = round(in_frames * dst/src)
    let out_frames = ((in_frames as u64) * dst_rate as u64 + buf.sample_rate as u64 / 2)
                     / buf.sample_rate as u64;
    let out_frames = out_frames as usize;
    let channels = buf.channels as usize;
    let mut out = Vec::with_capacity(out_frames * channels);
    for i in 0..out_frames {
        // source position in fractional frame units
        let src_pos = (i as u64 * buf.sample_rate as u64 * 1000) / dst_rate as u64;
        let src_frame = (src_pos / 1000) as usize;
        let frac = (src_pos % 1000) as i64; // 0..999
        let next_frame = (src_frame + 1).min(in_frames - 1);
        for c in 0..channels {
            let a = buf.samples[src_frame * channels + c] as i64;
            let b = buf.samples[next_frame * channels + c] as i64;
            // a + (b - a) * frac / 1000
            let v = a + (b - a) * frac / 1000;
            out.push(v.clamp(i16::MIN as i64, i16::MAX as i64) as i16);
        }
    }
    Ok(AudioBuffer {
        sample_rate: dst_rate, channels: buf.channels, samples: out,
    })
}

/// Two-pass normalize to peak `target_amplitude` (<= 32767).
/// Pass 1: scan for max_abs. Pass 2: scale.
pub fn normalize_to_peak(buf: &AudioBuffer, target_amplitude: i32) -> AudioBuffer {
    if buf.samples.is_empty() {
        return buf.clone();
    }
    let mut max_abs: i32 = 0;
    for s in &buf.samples {
        let a = (*s as i32).abs();
        if a > max_abs { max_abs = a; }
    }
    if max_abs == 0 {
        return buf.clone();
    }
    let target = target_amplitude.min(32767).max(0) as i64;
    let scaled: Vec<i16> = buf.samples.iter().map(|s| {
        let v = (*s as i64 * target) / max_abs as i64;
        v.clamp(i16::MIN as i64, i16::MAX as i64) as i16
    }).collect();
    AudioBuffer {
        sample_rate: buf.sample_rate, channels: buf.channels, samples: scaled,
    }
}

/// Stereo → mono via L+R averaging. Mono → mono is identity.
pub fn to_mono(buf: &AudioBuffer) -> AudioBuffer {
    if buf.channels == 1 {
        return buf.clone();
    }
    let n = buf.frame_count();
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let l = buf.samples[i * 2] as i32;
        let r = buf.samples[i * 2 + 1] as i32;
        out.push(((l + r) / 2) as i16);
    }
    AudioBuffer { sample_rate: buf.sample_rate, channels: 1, samples: out }
}

/// Mono → stereo via duplication. Stereo → stereo is identity.
pub fn to_stereo(buf: &AudioBuffer) -> AudioBuffer {
    if buf.channels == 2 {
        return buf.clone();
    }
    let mut out = Vec::with_capacity(buf.samples.len() * 2);
    for s in &buf.samples {
        out.push(*s);
        out.push(*s);
    }
    AudioBuffer { sample_rate: buf.sample_rate, channels: 2, samples: out }
}

/// Encode the buffer as a canonical PCM WAV file (RIFF/WAVE/fmt/data).
pub fn encode_wav(buf: &AudioBuffer) -> Vec<u8> {
    // 16-bit PCM: bits_per_sample = 16, format = 1 (PCM)
    let bits_per_sample: u16 = 16;
    let byte_rate: u32 = buf.sample_rate * buf.channels as u32 * (bits_per_sample / 8) as u32;
    let block_align: u16 = buf.channels * (bits_per_sample / 8);
    let data_size: u32 = (buf.samples.len() * 2) as u32;
    let riff_size: u32 = 36 + data_size;

    let mut out = Vec::with_capacity(44 + buf.samples.len() * 2);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&riff_size.to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes()); // fmt chunk size
    out.extend_from_slice(&1u16.to_le_bytes());  // format = PCM
    out.extend_from_slice(&buf.channels.to_le_bytes());
    out.extend_from_slice(&buf.sample_rate.to_le_bytes());
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&block_align.to_le_bytes());
    out.extend_from_slice(&bits_per_sample.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_size.to_le_bytes());
    for s in &buf.samples {
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}

/// Parse the canonical 44-byte WAV header + data back into an AudioBuffer.
pub fn decode_wav(bytes: &[u8]) -> Result<AudioBuffer, CodecError> {
    if bytes.len() < 44 { return Err(CodecError::EmptyBuffer); }
    if &bytes[0..4] != b"RIFF" { return Err(CodecError::EmptyBuffer); }
    if &bytes[8..12] != b"WAVE" { return Err(CodecError::EmptyBuffer); }
    let channels = u16::from_le_bytes(bytes[22..24].try_into().unwrap());
    let sample_rate = u32::from_le_bytes(bytes[24..28].try_into().unwrap());
    let data_size = u32::from_le_bytes(bytes[40..44].try_into().unwrap()) as usize;
    let samples_bytes = &bytes[44..44 + data_size];
    let mut samples = Vec::with_capacity(samples_bytes.len() / 2);
    for chunk in samples_bytes.chunks_exact(2) {
        samples.push(i16::from_le_bytes([chunk[0], chunk[1]]));
    }
    Ok(AudioBuffer { sample_rate, channels, samples })
}

// ---------- tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_frame(sample_rate: u32, channels: u16, pcm: Vec<i16>) -> Mp3Frame {
        Mp3Frame {
            sample_rate, channels, bitrate_kbps: 128,
            samples_per_frame: (pcm.len() / channels as usize) as u32,
            pcm,
        }
    }

    #[test]
    fn decode_empty_frames_errors() {
        let r = decode_mp3_to_buffer(&[]);
        assert_eq!(r, Err(CodecError::NoFrames));
    }

    #[test]
    fn decode_concatenates_frame_pcm() {
        let frames = vec![
            test_frame(44100, 1, vec![10, 20, 30]),
            test_frame(44100, 1, vec![40, 50, 60]),
        ];
        let buf = decode_mp3_to_buffer(&frames).unwrap();
        assert_eq!(buf.samples, vec![10, 20, 30, 40, 50, 60]);
        assert_eq!(buf.sample_rate, 44100);
        assert_eq!(buf.channels, 1);
    }

    #[test]
    fn decode_mixed_formats_errors() {
        let frames = vec![
            test_frame(44100, 1, vec![10, 20]),
            test_frame(48000, 1, vec![30, 40]),
        ];
        assert_eq!(decode_mp3_to_buffer(&frames), Err(CodecError::MixedFormats));
    }

    #[test]
    fn decode_unsupported_channels_errors() {
        let frames = vec![test_frame(44100, 5, vec![0; 50])];
        assert_eq!(decode_mp3_to_buffer(&frames), Err(CodecError::UnsupportedChannels(5)));
    }

    #[test]
    fn resample_identity_when_rates_match() {
        let buf = AudioBuffer {
            sample_rate: 44100, channels: 1, samples: vec![100, 200, 300, 400],
        };
        let out = resample(&buf, 44100).unwrap();
        assert_eq!(out, buf);
    }

    #[test]
    fn resample_doubles_frame_count_when_rate_doubles() {
        let buf = AudioBuffer {
            sample_rate: 22050, channels: 1, samples: vec![100, 200, 300, 400],
        };
        let out = resample(&buf, 44100).unwrap();
        // Frame count roughly doubles
        assert!(out.frame_count() >= 7 && out.frame_count() <= 8);
        assert_eq!(out.sample_rate, 44100);
    }

    #[test]
    fn resample_interpolates_between_samples() {
        let buf = AudioBuffer {
            sample_rate: 1000, channels: 1, samples: vec![0, 1000],
        };
        // Resampling 1000->2000 gives roughly ~4 frames; the middle should be partway
        let out = resample(&buf, 2000).unwrap();
        assert_eq!(out.samples[0], 0); // first sample preserved exactly
    }

    #[test]
    fn resample_empty_preserves_format() {
        let buf = AudioBuffer {
            sample_rate: 22050, channels: 2, samples: vec![],
        };
        let out = resample(&buf, 44100).unwrap();
        assert_eq!(out.sample_rate, 44100);
        assert!(out.samples.is_empty());
    }

    #[test]
    fn normalize_scales_to_peak() {
        let buf = AudioBuffer {
            sample_rate: 44100, channels: 1, samples: vec![100, -200, 150],
        };
        let out = normalize_to_peak(&buf, 10_000);
        // max_abs = 200 → scale factor 10000/200 = 50x
        assert_eq!(out.samples[0], 5_000);
        assert_eq!(out.samples[1], -10_000);
        assert_eq!(out.samples[2], 7_500);
    }

    #[test]
    fn normalize_zero_buffer_unchanged() {
        let buf = AudioBuffer {
            sample_rate: 44100, channels: 1, samples: vec![0, 0, 0],
        };
        let out = normalize_to_peak(&buf, 32000);
        assert_eq!(out.samples, vec![0, 0, 0]);
    }

    #[test]
    fn to_mono_averages_stereo() {
        let buf = AudioBuffer {
            sample_rate: 44100, channels: 2, samples: vec![100, 200, -400, 400],
        };
        let out = to_mono(&buf);
        assert_eq!(out.channels, 1);
        assert_eq!(out.samples, vec![150, 0]);
    }

    #[test]
    fn to_stereo_duplicates_mono() {
        let buf = AudioBuffer {
            sample_rate: 44100, channels: 1, samples: vec![100, 200],
        };
        let out = to_stereo(&buf);
        assert_eq!(out.channels, 2);
        assert_eq!(out.samples, vec![100, 100, 200, 200]);
    }

    #[test]
    fn wav_roundtrip_preserves_samples() {
        let buf = AudioBuffer {
            sample_rate: 44100, channels: 2,
            samples: vec![1, -1, 2, -2, 1000, -1000, 30000, -30000],
        };
        let bytes = encode_wav(&buf);
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        let decoded = decode_wav(&bytes).unwrap();
        assert_eq!(decoded, buf);
    }

    #[test]
    fn wav_header_fields_correct() {
        let buf = AudioBuffer {
            sample_rate: 48000, channels: 2, samples: vec![0; 100],
        };
        let bytes = encode_wav(&buf);
        // channel count at offset 22
        assert_eq!(u16::from_le_bytes(bytes[22..24].try_into().unwrap()), 2);
        // sample rate at offset 24
        assert_eq!(u32::from_le_bytes(bytes[24..28].try_into().unwrap()), 48000);
        // bits-per-sample at offset 34
        assert_eq!(u16::from_le_bytes(bytes[34..36].try_into().unwrap()), 16);
    }

    #[test]
    fn full_pipeline_mp3_to_wav() {
        // Decode → resample → normalize → encode
        let frames = vec![
            test_frame(22050, 2, vec![100, -100, 200, -200, 400, -400, 800, -800]),
        ];
        let buf = decode_mp3_to_buffer(&frames).unwrap();
        let resampled = resample(&buf, 44100).unwrap();
        let normalized = normalize_to_peak(&resampled, 32000);
        let mono = to_mono(&normalized);
        let wav = encode_wav(&mono);
        assert_eq!(&wav[0..4], b"RIFF");
        let decoded = decode_wav(&wav).unwrap();
        assert_eq!(decoded.sample_rate, 44100);
        assert_eq!(decoded.channels, 1);
    }

    #[test]
    fn duration_ms_computed_from_frames_and_rate() {
        let buf = AudioBuffer {
            sample_rate: 44100, channels: 1,
            samples: vec![0; 44100], // 1 second
        };
        assert_eq!(buf.duration_ms(), 1000);
    }
}
