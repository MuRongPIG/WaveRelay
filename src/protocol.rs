use std::io::Read;

use anyhow::{Context, Result, bail, ensure};
use opus_decoder::OpusDecoder;
use serde::Deserialize;

pub const SAMPLE_RATE: u32 = 48_000;
const MAX_PACKET: usize = 64 * 1024;
const PCM_FRAME_BYTES: usize = 4;
const OPUS_MAX_FRAMES: usize = 5760; // 120 ms at 48 kHz.

#[derive(Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AudioQuality {
    #[default]
    Pcm,
    Opus96,
    Opus192,
}

impl AudioQuality {
    pub fn server_args(self) -> &'static [&'static str] {
        match self {
            Self::Pcm => &["audio_codec=raw"],
            Self::Opus96 => &["audio_codec=opus", "audio_bit_rate=96000"],
            Self::Opus192 => &["audio_codec=opus", "audio_bit_rate=192000"],
        }
    }

    fn codec_id(self) -> [u8; 4] {
        match self {
            Self::Pcm => *b"\0raw",
            Self::Opus96 | Self::Opus192 => *b"opus",
        }
    }
}

pub struct AudioReader<R> {
    input: R,
    quality: AudioQuality,
    payload: Vec<u8>,
    opus: Option<OpusStream>,
}

impl<R: Read> AudioReader<R> {
    pub fn new(mut input: R, quality: AudioQuality) -> Result<Self> {
        let mut codec = [0; 4];
        input.read_exact(&mut codec)?;
        match codec {
            [0, 0, 0, 0] => bail!(
                "Android could not capture audio. On Android 11, unlock the screen before starting."
            ),
            [0, 0, 0, 1] => bail!("The Android server rejected the audio configuration"),
            id => ensure!(
                id == quality.codec_id(),
                "Unexpected scrcpy audio codec: {id:02x?}"
            ),
        }
        Ok(Self {
            input,
            quality,
            payload: Vec::new(),
            opus: None,
        })
    }

    pub fn next_frames(&mut self, frames: &mut Vec<[f32; 2]>) -> Result<bool> {
        frames.clear();
        loop {
            let mut header = [0; 12];
            // EOF between packets is distinct from EOF in a header or payload.
            if self.input.read(&mut header[..1])? == 0 {
                return Ok(false);
            }
            self.input.read_exact(&mut header[1..])?;
            let size = u32::from_be_bytes(header[8..].try_into().unwrap()) as usize;
            ensure!(
                size > 0 && size <= MAX_PACKET,
                "Invalid scrcpy audio packet size"
            );
            let config = header[0] & 0x80 != 0;
            if matches!(self.quality, AudioQuality::Pcm) {
                ensure!(
                    !config && size.is_multiple_of(PCM_FRAME_BYTES),
                    "Invalid scrcpy PCM packet"
                );
            }
            self.payload.resize(size, 0);
            self.input.read_exact(&mut self.payload)?;
            if matches!(self.quality, AudioQuality::Pcm) {
                frames.extend(
                    self.payload
                        .as_chunks::<PCM_FRAME_BYTES>()
                        .0
                        .iter()
                        .map(|frame| {
                            let left = i16::from_le_bytes([frame[0], frame[1]]);
                            let right = i16::from_le_bytes([frame[2], frame[3]]);
                            [f32::from(left) / 32768.0, f32::from(right) / 32768.0]
                        }),
                );
                return Ok(true);
            }
            if config {
                self.opus = Some(OpusStream::new(&self.payload)?);
                continue;
            }
            let opus = self
                .opus
                .as_mut()
                .context("Missing scrcpy Opus configuration")?;
            opus.decode(&self.payload, frames)?;
            if !frames.is_empty() {
                return Ok(true);
            }
        }
    }
}

struct OpusStream {
    decoder: OpusDecoder,
    pcm: Vec<f32>,
    skip: usize,
    gain: f32,
}

impl OpusStream {
    fn new(header: &[u8]) -> Result<Self> {
        // scrcpy strips the Android CSD wrapper and sends an OpusHead packet.
        ensure!(
            header.len() >= 19
                && header.starts_with(b"OpusHead")
                && header[8] == 1
                && matches!(header[9], 1 | 2)
                && header[18] == 0,
            "Unsupported scrcpy Opus header (expected mono or stereo with mapping family 0)"
        );
        let skip = u16::from_le_bytes([header[10], header[11]]) as usize;
        let gain_db = f32::from(i16::from_le_bytes([header[16], header[17]])) / 256.0;
        Ok(Self {
            decoder: OpusDecoder::new(SAMPLE_RATE, 2)?,
            pcm: vec![0.0; OPUS_MAX_FRAMES * 2],
            skip,
            gain: 10.0_f32.powf(gain_db / 20.0),
        })
    }

    fn decode(&mut self, payload: &[u8], frames: &mut Vec<[f32; 2]>) -> Result<()> {
        let decoded = self.decoder.decode_float(payload, &mut self.pcm, false)?;
        let skip = self.skip.min(decoded);
        self.skip -= skip;
        frames.extend(
            self.pcm[skip * 2..decoded * 2]
                .as_chunks::<2>()
                .0
                .iter()
                .map(|[left, right]| {
                    [
                        (left * self.gain).clamp(-1.0, 1.0),
                        (right * self.gain).clamp(-1.0, 1.0),
                    ]
                }),
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_packet_and_decodes_signed_stereo() {
        let bytes = b"\0raw\0\0\0\0\0\0\0\x01\0\0\0\x04\0\x80\0\x40";
        let mut reader = AudioReader::new(bytes.as_slice(), AudioQuality::Pcm).unwrap();
        let mut frames = Vec::new();
        assert!(reader.next_frames(&mut frames).unwrap());
        assert_eq!(frames, [[-1.0, 0.5]]);
        assert!(!reader.next_frames(&mut frames).unwrap());
    }

    #[test]
    fn rejects_disabled_capture_and_unexpected_codec() {
        for codec in [b"\0\0\0\0", b"\0\0\0\x01", b"opus"] {
            assert!(AudioReader::new(codec.as_slice(), AudioQuality::Pcm).is_err());
        }
    }

    #[test]
    fn rejects_truncated_and_invalid_packets() {
        for bytes in [
            b"\0raw\0".as_slice(),
            b"\0raw\0\0\0\0\0\0\0\0\0\0\0\x04\0".as_slice(),
            b"\0raw\0\0\0\0\0\0\0\0\0\0\0\x03".as_slice(),
            b"\0raw\0\0\0\0\0\0\0\0\0\0\0\x02\0\0".as_slice(),
        ] {
            let mut reader = AudioReader::new(bytes, AudioQuality::Pcm).unwrap();
            assert!(reader.next_frames(&mut Vec::new()).is_err());
        }
    }

    #[test]
    fn decodes_opus_packet_and_applies_preskip() {
        let mut bytes = b"opus".to_vec();
        let header = b"OpusHead\x01\x02\x78\0\x80\xbb\0\0\0\0\0";
        bytes.extend_from_slice(&(1_u64 << 63).to_be_bytes());
        bytes.extend_from_slice(&(header.len() as u32).to_be_bytes());
        bytes.extend_from_slice(header);
        bytes.extend_from_slice(&0_u64.to_be_bytes());
        bytes.extend_from_slice(&3_u32.to_be_bytes());
        bytes.extend_from_slice(&[0xf8, 0xff, 0xfe]);
        let mut reader = AudioReader::new(bytes.as_slice(), AudioQuality::Opus96).unwrap();
        let mut frames = Vec::new();
        assert!(reader.next_frames(&mut frames).unwrap());
        assert_eq!(frames.len(), 960 - 120);
        assert!(frames.iter().flatten().all(|sample| sample.is_finite()));
        assert!(!reader.next_frames(&mut frames).unwrap());
    }
}
