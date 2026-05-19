use grok_6502::bus::{Bus, SimpleBus};

pub const SAMPLE_RATE: u32 = 44100;

const CYCLES_PER_SAMPLE: u32 = crate::settings::CPU_CLK_SPEED / SAMPLE_RATE;

mod soft_switch {
    pub const SPEAKER: usize = 0xC030; // Whole page
}

pub(crate) struct Sound<A: Audio> {
    audio: A,
    prev_polarity: bool,
    polarity: bool,
    polarity_change: bool,
    cycles: u32,
    // TODO: Remove std dep
    samples: Vec<bool>,
}

impl<A: Audio> Sound<A> {
    pub(crate) fn new(audio: A) -> Self {
        Sound {
            audio,
            prev_polarity: false,
            polarity: false,
            polarity_change: false,
            cycles: 0,
            samples: Vec::new(),
        }
    }

    pub(crate) fn reset(&mut self) {
        self.cycles = 0;
        self.samples.clear();
        self.polarity_change = false;
        self.prev_polarity = self.polarity;
    }

    pub(crate) fn tick(&mut self, bus: &SimpleBus) {
        self.cycles += 1;

        if self.cycles >= CYCLES_PER_SAMPLE {
            self.samples.push(self.polarity);
            self.cycles = 0;
            if self.polarity != self.prev_polarity {
                self.polarity_change = true;
            }
        }

        if bus.addr() as usize == soft_switch::SPEAKER {
            self.polarity = !self.polarity;
        }
    }

    pub(crate) fn feed_samples(&mut self) {
        // Note: Only feeding on poalrity change I think is more of an SDL issue and
        // should likely be removed from here once that is cleaned up
        if self.polarity_change {
            self.audio.feed_samples(&self.samples);
        }
        self.reset();
    }
}

pub trait Audio {
    fn feed_samples(&mut self, samples: &[bool]);
}
