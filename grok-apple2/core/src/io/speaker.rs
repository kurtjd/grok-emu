use crate::settings;

const CYCLES_PER_SAMPLE: u32 = settings::CPU_CLK_SPEED / settings::SAMPLE_RATE;
const SAMPLES_PER_FRAME: usize = (settings::CYCLES_PER_FRAME / CYCLES_PER_SAMPLE) as usize;

pub(crate) struct Speaker {
    prev_polarity: bool,
    polarity: bool,
    cycles: u32,
    samples_idx: usize,
    samples: [bool; SAMPLES_PER_FRAME],
}

impl Speaker {
    pub(crate) fn new() -> Self {
        Speaker {
            prev_polarity: false,
            polarity: false,
            cycles: 0,
            samples_idx: 0,
            samples: [false; SAMPLES_PER_FRAME],
        }
    }

    pub(crate) fn begin_frame(&mut self) {
        self.cycles = 0;
        self.samples_idx = 0;
        self.samples = [false; SAMPLES_PER_FRAME];
        self.prev_polarity = self.polarity;
    }

    pub(crate) fn tick(&mut self) {
        self.cycles += 1;

        if self.cycles >= CYCLES_PER_SAMPLE {
            self.samples[self.samples_idx] = self.polarity;
            self.samples_idx += 1;
            self.cycles = 0;
        }
    }

    pub(crate) fn decode(&mut self) {
        self.polarity = !self.polarity;
    }

    pub(crate) fn samples(&mut self) -> &[bool] {
        &self.samples
    }
}
