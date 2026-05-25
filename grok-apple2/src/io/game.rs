//! TODO. Just here as a stub to drive pushbutton and controller input low for now
//! so games don't get confused and think they are pressed.
use crate::mem_map;
use grok_6502::bus::Bus;

#[derive(Default)]
pub(crate) struct Game {}

impl Game {
    pub(crate) fn decode(&mut self, bus: &mut dyn Bus) {
        #[allow(clippy::single_match)]
        match bus.addr() {
            mem_map::PUSHBTN_IN..mem_map::TIMER_TRIGGER => bus.set_data(0),
            _ => (),
        }
    }
}
