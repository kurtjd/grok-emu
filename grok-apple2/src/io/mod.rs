pub(crate) mod game;
pub(crate) mod graphics;
pub(crate) mod keyboard;
pub(crate) mod speaker;

use crate::mem_map;
use grok_6502::bus::Bus;

pub trait Video {
    fn draw_frame(&mut self, frame_buf: &[u8]);
}

pub trait Audio {
    fn feed_samples(&mut self, samples: &[bool]);
}

pub(crate) struct Io<V: Video, A: Audio> {
    pub(crate) keyboard: keyboard::Keyboard,
    pub(crate) graphics: graphics::Graphics<V>,
    pub(crate) speaker: speaker::Speaker<A>,
    pub(crate) game: game::Game,
}

impl<V: Video, A: Audio> Io<V, A> {
    pub(crate) fn decode(&mut self, bus: &mut dyn Bus) {
        match bus.addr() {
            mem_map::KEYBOARD_EN..mem_map::KEYBOARD_CLR => self.keyboard.decode(bus),
            mem_map::KEYBOARD_CLR..mem_map::CASSETTE_TOGGLE => self.keyboard.decode(bus),
            mem_map::CASSETTE_TOGGLE..mem_map::SPEAKER => (),
            mem_map::SPEAKER..mem_map::UTIL_STROBE => self.speaker.decode(),
            mem_map::UTIL_STROBE..mem_map::SCREEN_MODE => (),
            mem_map::SCREEN_MODE..mem_map::ANNUNCIATOR => self.graphics.decode(bus),
            mem_map::ANNUNCIATOR..mem_map::CASSETTE_IN => self.game.decode(bus),
            mem_map::CASSETTE_IN..mem_map::PUSHBTN_IN => (),
            mem_map::PUSHBTN_IN..mem_map::CONTROLLER_IN => self.game.decode(bus),
            mem_map::CONTROLLER_IN..mem_map::CASSETTE_IN_ALT => self.game.decode(bus),
            mem_map::CASSETTE_IN_ALT..mem_map::PUSHBTN_IN_ALT => (),
            mem_map::PUSHBTN_IN_ALT..mem_map::CONTROLLER_IN_ALT => self.game.decode(bus),
            mem_map::CONTROLLER_IN_ALT..mem_map::TIMER_TRIGGER => self.game.decode(bus),
            mem_map::TIMER_TRIGGER.. => self.game.decode(bus),
            _ => unreachable!(),
        }
    }
}
