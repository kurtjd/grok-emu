pub(crate) mod game;
pub(crate) mod keyboard;
pub(crate) mod speaker;
pub(crate) mod video;

use crate::mem_map;
use grok_6502::bus::Bus;

pub trait Audio {
    fn feed_samples(&mut self, samples: &[bool]);
}

pub(crate) struct Io<A: Audio> {
    pub(crate) keyboard: keyboard::Keyboard,
    pub(crate) video: video::Video,
    pub(crate) speaker: speaker::Speaker<A>,
    pub(crate) game: game::Game,
}

impl<A: Audio> Io<A> {
    pub(crate) fn decode(&mut self, bus: &mut dyn Bus) {
        match bus.addr() {
            mem_map::KEYBOARD_EN..mem_map::KEYBOARD_CLR => self.keyboard.decode(bus),
            mem_map::KEYBOARD_CLR..mem_map::CASSETTE_TOGGLE => self.keyboard.decode(bus),
            mem_map::CASSETTE_TOGGLE..mem_map::SPEAKER => (),
            mem_map::SPEAKER..mem_map::UTIL_STROBE => self.speaker.decode(),
            mem_map::UTIL_STROBE..mem_map::SCREEN_MODE => (),
            mem_map::SCREEN_MODE..mem_map::ANNUNCIATOR => self.video.decode(bus),
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
