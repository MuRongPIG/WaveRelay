use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU32, Ordering},
};

use anyhow::{Context, Result, bail, ensure};
use cpal::{
    Device, FromSample, SampleFormat, SizedSample, Stream, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use ringbuf::{HeapCons, HeapProd, HeapRb, traits::*};

use crate::protocol::SAMPLE_RATE;

type Frame = [f32; 2];

pub struct Playback {
    _stream: Stream,
    queue: HeapProd<Frame>,
    converter: Resampler,
    failed: Arc<AtomicBool>,
}

impl Playback {
    pub fn start(buffer_ms: u32, volume: Arc<AtomicU32>, stop: Arc<AtomicBool>) -> Result<Self> {
        let device = cpal::default_host()
            .default_output_device()
            .context("No default audio output device")?;
        let supported = device
            .default_output_config()
            .context("Could not open default audio output")?;
        let config = supported.config();
        ensure!(
            config.channels > 0 && config.sample_rate.0 > 0,
            "Invalid default audio output format"
        );

        let target =
            (u64::from(config.sample_rate.0) * u64::from(buffer_ms) / 1000).max(1) as usize;
        let (queue, consumer) = HeapRb::<Frame>::new(target * 4).split();
        let failed = Arc::new(AtomicBool::new(false));

        macro_rules! build {
            ($sample:ty) => {
                build_stream::<$sample>(
                    &device,
                    &config,
                    consumer,
                    target,
                    volume,
                    failed.clone(),
                    stop,
                )
            };
        }
        let stream = match supported.sample_format() {
            SampleFormat::I8 => build!(i8),
            SampleFormat::I16 => build!(i16),
            SampleFormat::I32 => build!(i32),
            SampleFormat::I64 => build!(i64),
            SampleFormat::U8 => build!(u8),
            SampleFormat::U16 => build!(u16),
            SampleFormat::U32 => build!(u32),
            SampleFormat::U64 => build!(u64),
            SampleFormat::F32 => build!(f32),
            SampleFormat::F64 => build!(f64),
            format => bail!("Unsupported output sample format: {format}"),
        }?;
        stream.play().context("Could not start playback")?;
        Ok(Self {
            _stream: stream,
            queue,
            converter: Resampler::new(config.sample_rate.0),
            failed,
        })
    }

    pub fn push(&mut self, frames: impl IntoIterator<Item = Frame>) {
        let (converter, queue) = (&mut self.converter, &mut self.queue);
        for frame in frames {
            converter.push(frame, |sample| {
                let _ = queue.try_push(sample);
            });
        }
    }

    pub fn failed(&self) -> bool {
        self.failed.load(Ordering::Relaxed)
    }
}

fn build_stream<T: SizedSample + FromSample<f32>>(
    device: &Device,
    config: &StreamConfig,
    mut queue: HeapCons<Frame>,
    target: usize,
    volume: Arc<AtomicU32>,
    failed: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
) -> Result<Stream> {
    let channels = usize::from(config.channels);
    let mut playing = false;
    Ok(device.build_output_stream(
        config,
        move |output: &mut [T], _| {
            let volume = volume.load(Ordering::Relaxed).min(100) as f32 / 100.0;
            let queued = queue.occupied_len();
            let callback_frames = output.len() / channels;
            let prefill = target.max(callback_frames).min(queue.capacity().get());
            // Drop old frames when playback falls behind. The callback owns the read index.
            if queued > (prefill + callback_frames).max(prefill * 2) {
                queue.skip(queued - prefill);
            }
            playing |= queued >= prefill;

            for frame in output.chunks_exact_mut(channels) {
                let stereo = playing.then(|| queue.try_pop()).flatten();
                playing = stereo.is_some();
                let [left, right] = stereo.unwrap_or_default();
                frame.fill(T::from_sample(0.0));
                if channels == 1 {
                    frame[0] = T::from_sample((left + right) * 0.5 * volume);
                } else {
                    frame[0] = T::from_sample(left * volume);
                    frame[1] = T::from_sample(right * volume);
                }
            }
        },
        move |error| {
            eprintln!("Playback device error: {error}");
            failed.store(true, Ordering::Relaxed);
            stop.store(true, Ordering::Relaxed);
        },
        None,
    )?)
}

struct Resampler {
    rate: u32,
    phase: u64,
    previous: Option<Frame>,
}

impl Resampler {
    fn new(rate: u32) -> Self {
        Self {
            rate,
            phase: 0,
            previous: None,
        }
    }

    fn push(&mut self, current: Frame, mut output: impl FnMut(Frame)) {
        if self.rate == SAMPLE_RATE {
            output(current);
            return;
        }
        let Some(previous) = self.previous.replace(current) else {
            return;
        };
        let rate = u64::from(self.rate);
        // Phase carries the position of the next output frame across packet boundaries.
        while self.phase < rate {
            let mix = self.phase as f32 / rate as f32;
            output([
                previous[0] + (current[0] - previous[0]) * mix,
                previous[1] + (current[1] - previous[1]) * mix,
            ]);
            self.phase += u64::from(SAMPLE_RATE);
        }
        self.phase -= rate;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resampling_preserves_rate_and_channel_order() {
        for rate in [44_100, 48_000, 96_000] {
            let mut resampler = Resampler::new(rate);
            let mut count = 0;
            for _ in 0..=SAMPLE_RATE {
                resampler.push([0.25, -0.5], |frame| {
                    assert_eq!(frame, [0.25, -0.5]);
                    count += 1;
                });
            }
            assert_eq!(count, rate + u32::from(rate == SAMPLE_RATE));
        }
    }

    #[test]
    fn upsamples_with_interpolation() {
        let mut resampler = Resampler::new(96_000);
        let mut output = Vec::new();
        resampler.push([0.0, 1.0], |frame| output.push(frame));
        resampler.push([1.0, 0.0], |frame| output.push(frame));
        assert_eq!(output, [[0.0, 1.0], [0.5, 0.5]]);
    }
}
