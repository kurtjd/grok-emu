use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use grok_apple2_core::{Apple2, settings};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
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
    // Emulation speed factor (1 = normal, N = Nx fast-forward), shared with the
    // app. At Nx the emulator produces N samples for every one the output
    // consumes, so `feed_samples` keeps only every Nth: the ring fills at the
    // normal rate while spanning Nx the emulated time, giving sped-up "chipmunk"
    // playback instead of overrunning the ring.
    speed: Arc<AtomicU32>,
}

impl CpalAudio {
    /// Build a sink feeding the given ring buffer. Used to attach a freshly
    /// rebuilt emulator (e.g. after a power cycle) to the already-running output
    /// stream instead of tearing the audio device down and back up.
    pub fn new(ring: Arc<Mutex<VecDeque<f32>>>, speed: Arc<AtomicU32>) -> Self {
        Self { ring, speed }
    }

    /// A handle to the shared ring buffer, so another sink can later be built for
    /// the same output stream.
    pub fn ring(&self) -> Arc<Mutex<VecDeque<f32>>> {
        self.ring.clone()
    }
}

impl grok_apple2_core::Audio for CpalAudio {
    fn feed_samples(&mut self, samples: &[bool]) {
        // Keep every `stride`th sample: at 1x that's all of them; at Nx it drops
        // the extra samples fast-forward produces so playback speeds up in pitch
        // rather than flooding the ring. A square-wave beeper doesn't care about
        // the sub-sample jitter this adds at frame boundaries.
        let stride = self.speed.load(Ordering::Relaxed).max(1) as usize;
        let mut ring = self.ring.lock().unwrap();
        let room = AUDIO_BUF_MAX.saturating_sub(ring.len());
        ring.extend(
            samples
                .iter()
                .step_by(stride)
                .take(room)
                .map(|&high| if high { SAMPLE_VOLUME } else { -SAMPLE_VOLUME }),
        );
    }
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    ring: Arc<Mutex<VecDeque<f32>>>,
    muted: Arc<AtomicBool>,
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
                let muted = muted.load(Ordering::Relaxed);
                for frame in out.chunks_mut(channels) {
                    // Drain the ring even when muted so it stays in sync with the
                    // emulator; just emit silence instead of the popped sample.
                    let popped = ring.pop_front().unwrap_or(0.0);
                    let sample = T::from_sample(if muted { 0.0 } else { popped });
                    frame.fill(sample);
                }
            },
            |err| eprintln!("audio stream error: {err}"),
            None,
        )
        .expect("failed to build audio output stream")
}

/// Open the default output device and start streaming from the returned
/// `CpalAudio` sink. The `cpal::Stream` must be kept alive to keep playing. The
/// returned `Arc<AtomicBool>` mutes the output when set; it lives in the output
/// stream (built once) so muting persists across emulator power cycles. The
/// returned `Arc<AtomicU32>` is the emulation speed factor (1 = normal); it lives
/// in the sink, so a power cycle must reattach it to the rebuilt `CpalAudio`.
pub fn init_audio() -> (CpalAudio, cpal::Stream, Arc<AtomicBool>, Arc<AtomicU32>) {
    let ring = Arc::new(Mutex::new(VecDeque::new()));
    let muted = Arc::new(AtomicBool::new(false));
    let speed = Arc::new(AtomicU32::new(1));

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
        cpal::SampleFormat::F32 => {
            build_stream::<f32>(&device, &config, ring.clone(), muted.clone())
        }
        cpal::SampleFormat::I16 => {
            build_stream::<i16>(&device, &config, ring.clone(), muted.clone())
        }
        cpal::SampleFormat::U16 => {
            build_stream::<u16>(&device, &config, ring.clone(), muted.clone())
        }
        other => panic!("unsupported audio sample format: {other:?}"),
    };
    stream.play().expect("failed to start audio stream");

    (
        CpalAudio {
            ring,
            speed: speed.clone(),
        },
        stream,
        muted,
        speed,
    )
}
