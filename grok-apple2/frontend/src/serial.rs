use grok_apple2_core::peripheral::serial;
use serialport::{SerialPort, TTYPort};
use std::io::{Read, Write};
use std::process::Command;
use std::time::Duration;

// TODO: Make this more flexible by also allowing user to pass in existing port etc
pub struct StdSerialPort(TTYPort);
impl StdSerialPort {
    pub fn new() -> Self {
        const VIRTUAL_PORT_LINK: &str = "/dev/ttyUSB9";

        let (mut master, slave) = TTYPort::pair().expect("Failed to create virtual serial port");

        master
            .set_timeout(Duration::from_millis(0))
            .expect("Failed to configure virtual serial port");

        if let Some(name) = slave.name() {
            // Revisit: Get rid of this
            //
            // Specifically here because ADTPro is annoying and doesn't let you enter port manually,
            // and only scans /dev/ttyUSB*, etc (but never /dev/pts/n) so we have to do this
            // nasty hack
            let linked = Command::new("sudo")
                .args(["ln", "-sf", &name, VIRTUAL_PORT_LINK])
                .status()
                .is_ok_and(|status| status.success());

            if linked {
                println!("Super Serial Card available at {VIRTUAL_PORT_LINK} (-> {name})");
            } else {
                println!(
                    "Super Serial Card available at {name} \
                     (couldn't sudo-link {VIRTUAL_PORT_LINK})"
                );
            }
        }

        Self(master)
    }
}

impl serial::SerialPort for StdSerialPort {
    fn read(&mut self) -> Option<u8> {
        let mut buf = [0];
        match self.0.read(&mut buf) {
            Ok(1) => Some(buf[0]),
            _ => None,
        }
    }

    fn write(&mut self, data: u8) {
        let _ = self.0.write(&[data]);
    }

    fn set_baud(&mut self, baud: serial::Baud) {
        let _ = self.0.set_baud_rate(baud.into());
    }

    fn set_word_length(&mut self, word_length: serial::WordLength) {
        let d = match word_length {
            serial::WordLength::_5 => serialport::DataBits::Five,
            serial::WordLength::_6 => serialport::DataBits::Six,
            serial::WordLength::_7 => serialport::DataBits::Seven,
            serial::WordLength::_8 => serialport::DataBits::Eight,
        };
        let _ = self.0.set_data_bits(d);
    }

    fn set_stop_bits(&mut self, stop_bits: serial::StopBits) {
        let s = match stop_bits {
            serial::StopBits::_1 => serialport::StopBits::One,
            serial::StopBits::_2 => serialport::StopBits::Two,
        };
        let _ = self.0.set_stop_bits(s);
    }

    fn set_parity(&mut self, parity: serial::Parity) {
        let p = match parity {
            serial::Parity::None | serial::Parity::Mark | serial::Parity::Space => {
                serialport::Parity::None
            }
            serial::Parity::Odd => serialport::Parity::Odd,
            serial::Parity::Even => serialport::Parity::Even,
        };
        let _ = self.0.set_parity(p);
    }

    fn rts_assert(&mut self, _assert: bool) {
        // Do nothing
    }

    fn brk_assert(&mut self, _assert: bool) {
        // Do nothing
    }

    fn cts_asserted(&mut self) -> bool {
        true
    }

    fn dtr_assert(&mut self, _assert: bool) {
        // Do nothing
    }

    fn dsr_asserted(&mut self) -> bool {
        true
    }

    fn dcd_asserted(&mut self) -> bool {
        true
    }

    fn parity_err(&mut self) -> bool {
        false
    }

    fn framing_err(&mut self) -> bool {
        false
    }

    fn overrun(&mut self) -> bool {
        false
    }
}
