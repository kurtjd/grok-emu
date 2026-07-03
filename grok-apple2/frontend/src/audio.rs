use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use grok_apple2_core::{Apple2, settings};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// The fully-assembled emulator, parameterized over our cpal audio backend.
pub type Machine = Apple2<'static, CpalAudio>;

const SAMPLE_VOLUME: f32 = 0.5;
/// Cap the audio ring buffer so it can't grow unbounded if the output stream stalls.
const AUDIO_BUF_MAX: usize = settings::SAMPLE_RATE as usize / 4; // ~250ms of samples

/// Implements the core `Audio` trait by pushing square-wave samples into a ring
/// buffer that the cpal output stream drains on its own thread.
pub struct CpalAudio {
    ring: Arc<Mutex<VecDeque<f32>>>,
}

impl grok_apple2_core::Audio for CpalAudio {
    fn feed_samples(&mut self, samples: &[bool]) {
        let mut ring = self.ring.lock().unwrap();
        let room = AUDIO_BUF_MAX.saturating_sub(ring.len());
        ring.extend(
            samples
                .iter()
                .take(room)
                .map(|&high| if high { SAMPLE_VOLUME } else { -SAMPLE_VOLUME }),
        );
    }
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    ring: Arc<Mutex<VecDeque<f32>>>,
) -> cpal::Stream
where
    T: cpal::SizedSample + cpal::FromSample<f32>,
{
    let channels = config.channels as usize;
    device
        .build_output_stream(
            config,
            move |out: &mut [T], _: &cpal::OutputCallbackInfo| {
                let mut ring = ring.lock().unwrap();
                for frame in out.chunks_mut(channels) {
                    // Mono source: replicate the sample across all channels.
                    let sample = T::from_sample(ring.pop_front().unwrap_or(0.0));
                    frame.fill(sample);
                }
            },
            |err| eprintln!("audio stream error: {err}"),
            None,
        )
        .expect("failed to build audio output stream")
}

/// Open the default output device and start streaming from the returned
/// `CpalAudio` sink. The `cpal::Stream` must be kept alive to keep playing.
pub fn init_audio() -> (CpalAudio, cpal::Stream) {
    let ring = Arc::new(Mutex::new(VecDeque::new()));

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no output audio device available");

    let default_cfg = device
        .default_output_config()
        .expect("no default output audio config");

    // Request the core's native sample rate. The device's default channel count
    // is kept so we don't fight the backend over channel layout.
    let config = cpal::StreamConfig {
        channels: default_cfg.channels(),
        sample_rate: cpal::SampleRate(settings::SAMPLE_RATE),
        buffer_size: cpal::BufferSize::Default,
    };

    let stream = match default_cfg.sample_format() {
        cpal::SampleFormat::F32 => build_stream::<f32>(&device, &config, ring.clone()),
        cpal::SampleFormat::I16 => build_stream::<i16>(&device, &config, ring.clone()),
        cpal::SampleFormat::U16 => build_stream::<u16>(&device, &config, ring.clone()),
        other => panic!("unsupported audio sample format: {other:?}"),
    };
    stream.play().expect("failed to start audio stream");

    (CpalAudio { ring }, stream)
}
