//! Apple II Super Serial Card.

use crate::peripheral::Peripheral;
use bitfield_struct::bitfield;
use grok_6502::bus::{Bus, Op};

/// SSC ROM Size.
pub const ROM_SIZE: usize = 0x800;

mod reg_addr {
    pub const DIPSW1: u8 = 0x1;
    pub const DIPSW2: u8 = 0x2;
    pub const XDREG: u8 = 0x8;
    pub const STATUS: u8 = 0x9;
    pub const COMMAND: u8 = 0xA;
    pub const CONTROL: u8 = 0xB;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TxControl {
    tie: bool,
    rts_assert: bool,
    brk_assert: bool,
}

impl TxControl {
    const fn from_bits(bits: u8) -> Self {
        TxControl {
            tie: bits == 0b01,
            rts_assert: bits != 0b00,
            brk_assert: bits == 0b11,
        }
    }

    const fn into_bits(self) -> u8 {
        match (self.tie, self.rts_assert, self.brk_assert) {
            (false, false, false) => 0b00,
            (true, true, false) => 0b01,
            (false, true, false) => 0b10,
            (false, true, true) => 0b11,
            _ => unreachable!(),
        }
    }
}

/// Serial word length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordLength {
    /// 5 bits.
    _5,
    /// 6 bits.
    _6,
    /// 7 bits.
    _7,
    /// 8 bits.
    _8,
}

impl WordLength {
    const fn from_bits(bits: u8) -> Self {
        match bits {
            0b00 => WordLength::_8,
            0b01 => WordLength::_7,
            0b10 => WordLength::_6,
            0b11 => WordLength::_5,
            _ => unreachable!(),
        }
    }

    const fn into_bits(self) -> u8 {
        match self {
            WordLength::_8 => 0b00,
            WordLength::_7 => 0b01,
            WordLength::_6 => 0b10,
            WordLength::_5 => 0b11,
        }
    }
}

impl From<WordLength> for u32 {
    fn from(word_length: WordLength) -> Self {
        match word_length {
            WordLength::_5 => 5,
            WordLength::_6 => 6,
            WordLength::_7 => 7,
            WordLength::_8 => 8,
        }
    }
}

/// Serial parity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parity {
    /// No parity.
    None,
    /// Odd parity.
    Odd,
    /// Even parity.
    Even,
    /// Mark parity.
    Mark,
    /// Space parity.
    Space,
}

impl Parity {
    const fn from_bits(bits: u8) -> Self {
        match bits {
            0b000 | 0b010 | 0b100 | 0b110 => Parity::None,
            0b001 => Parity::Odd,
            0b011 => Parity::Even,
            0b101 => Parity::Mark,
            0b111 => Parity::Space,
            _ => unreachable!(),
        }
    }

    const fn into_bits(self) -> u8 {
        match self {
            Parity::None => 0b000,
            Parity::Odd => 0b001,
            Parity::Even => 0b011,
            Parity::Mark => 0b101,
            Parity::Space => 0b111,
        }
    }
}

/// Serial stop bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopBits {
    /// 1 stop bit.
    _1,
    /// 2 stop bits.
    _2,
}

impl StopBits {
    const fn from_bits(bits: u8) -> Self {
        match bits {
            0b0 => StopBits::_1,
            0b1 => StopBits::_2,
            _ => unreachable!(),
        }
    }

    const fn into_bits(self) -> u8 {
        match self {
            StopBits::_1 => 0b0,
            StopBits::_2 => 0b1,
        }
    }
}

impl From<StopBits> for u32 {
    fn from(stop_bits: StopBits) -> Self {
        match stop_bits {
            StopBits::_1 => 1,
            StopBits::_2 => 2,
        }
    }
}

/// Serial baud rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Baud {
    /// 50 bps.
    _50,
    /// 75 bps.
    _75,
    /// 109.92 bps.
    _109_92,
    /// 134.58 bps.
    _134_58,
    /// 150 bps.
    _150,
    /// 300 bps.
    _300,
    /// 600 bps.
    _600,
    /// 1200 bps.
    _1200,
    /// 1800 bps.
    _1800,
    /// 2400 bps.
    _2400,
    /// 3600 bps.
    _3600,
    /// 4800 bps.
    _4800,
    /// 7200 bps.
    _7200,
    /// 9600 bps.
    _9600,
    /// 19200 bps.
    _19200,
    /// 115200 bps (external clock).
    _115200,
}

impl Baud {
    const fn from_bits(bits: u8) -> Self {
        match bits {
            // This technically tells 6551 to use an external clock,
            // but the SSC contains a 1.8432 MHz external clock,
            // so we just skip the foreplay and say 115200
            0b0001 => Baud::_50,
            0b0010 => Baud::_75,
            0b0011 => Baud::_109_92,
            0b0100 => Baud::_134_58,
            0b0101 => Baud::_150,
            0b0110 => Baud::_300,
            0b0111 => Baud::_600,
            0b1000 => Baud::_1200,
            0b1001 => Baud::_1800,
            0b1010 => Baud::_2400,
            0b1011 => Baud::_3600,
            0b1100 => Baud::_4800,
            0b1101 => Baud::_7200,
            0b1110 => Baud::_9600,
            0b1111 => Baud::_19200,
            0b0000 => Baud::_115200,
            _ => unreachable!(),
        }
    }

    const fn into_bits(self) -> u8 {
        match self {
            Baud::_50 => 0b0001,
            Baud::_75 => 0b0010,
            Baud::_109_92 => 0b0011,
            Baud::_134_58 => 0b0100,
            Baud::_150 => 0b0101,
            Baud::_300 => 0b0110,
            Baud::_600 => 0b0111,
            Baud::_1200 => 0b1000,
            Baud::_1800 => 0b1001,
            Baud::_2400 => 0b1010,
            Baud::_3600 => 0b1011,
            Baud::_4800 => 0b1100,
            Baud::_7200 => 0b1101,
            Baud::_9600 => 0b1110,
            Baud::_19200 => 0b1111,
            Baud::_115200 => 0b0000,
        }
    }
}

impl From<Baud> for u32 {
    fn from(baud: Baud) -> Self {
        match baud {
            Baud::_50 => 50,
            Baud::_75 => 75,
            Baud::_109_92 => 109,
            Baud::_134_58 => 134,
            Baud::_150 => 150,
            Baud::_300 => 300,
            Baud::_600 => 600,
            Baud::_1200 => 1200,
            Baud::_1800 => 1800,
            Baud::_2400 => 2400,
            Baud::_3600 => 3600,
            Baud::_4800 => 4800,
            Baud::_7200 => 7200,
            Baud::_9600 => 9600,
            Baud::_19200 => 19200,
            Baud::_115200 => 115200,
        }
    }
}

#[bitfield(u8)]
struct Status {
    parity_err: bool,
    framing_err: bool,
    overrun: bool,
    rdrf: bool,
    #[bits(default = true)]
    tdre: bool,
    ndcd: bool,
    ndsr: bool,
    irq: bool,
}

#[bitfield(u8)]
struct Control {
    #[bits(4)]
    baud: Baud,
    // Note: This is not supported according to manual so we don't emulate it
    rx_clock: bool,
    #[bits(2)]
    word_length: WordLength,
    #[bits(1)]
    stop_bits: StopBits,
}

#[bitfield(u8)]
struct Command {
    dtr: bool,
    nrie: bool,
    #[bits(2)]
    tx_control: TxControl,
    echo: bool,
    #[bits(3)]
    parity: Parity,
}

struct Acia6551<S: SerialPort> {
    status: Status,
    command: Command,
    control: Control,
    rdreg: u8,
    port: S,
    frame_ticks: u32,
    frame_ticks_max: u32,
}

impl<S: SerialPort> Acia6551<S> {
    fn new(port: S) -> Self {
        let mut acia = Acia6551 {
            control: Control::default(),
            command: Command::default(),
            status: Status::default(),
            rdreg: 0,
            port,
            frame_ticks: 0,
            frame_ticks_max: 0,
        };
        acia.hard_reset();
        acia
    }

    fn hard_reset(&mut self) {
        self.set_control(Control::default(), true);
        // Note: I found one datasheet that said bit 1 is set on hardware reset:
        // https://www.princeton.edu/~mae412/HANDOUTS/Datasheets/6551_acia.pdf
        //
        // But after hours of debugging it turns out the SSC firmware expects it to be clear,
        // and another copy of the datasheet I found also states bit 1 is clear:
        // https://www.devili.iki.fi/pub/Commodore/docs/datasheets/CSG/6551-8511.pdf
        //
        // So the first datasheet must be some incorrect older revision or something...
        self.set_command(Command::default(), true);
        self.status = Status::default();
    }

    fn soft_reset(&mut self) {
        self.set_command(
            self.command
                .with_echo(false)
                .with_tx_control(TxControl::from_bits(0b00))
                .with_nrie(false)
                .with_dtr(false),
            false,
        );
        self.status = self.status.with_overrun(false);
    }

    fn frame_reset(&mut self) {
        self.frame_ticks = 0;

        // TODO: Also generate TX rdy interrupt (if enabled in Command reg)
        self.status.set_tdre(true);

        if self.command.dtr()
            && !self.status.rdrf()
            && let Some(data) = self.port.read()
        {
            // TODO: Also generate RX rdy interrupt (if enabled in Command reg)
            self.rdreg = data;
            self.status.set_rdrf(true);

            // Update error bits in status
            self.status.set_parity_err(self.port.parity_err());
            self.status.set_framing_err(self.port.framing_err());
            self.status.set_overrun(self.port.overrun());

            // If echo is enabled and transmitter is off,
            // we automatically TX back what we just RXed
            if self.command.echo() && self.command.tx_control().into_bits() == 0b00 {
                self.write(data);
            }
        }
    }

    fn adjust_frame(&mut self) {
        let word_len = u32::from(self.control.word_length());
        let parity = if self.command.parity() == Parity::None {
            0
        } else {
            1
        };
        let stop_bits = u32::from(self.control.stop_bits());
        let frame_len = 1 + word_len + parity + stop_bits;
        self.frame_ticks_max =
            crate::settings::CPU_CLK_SPEED * frame_len / u32::from(self.control.baud());
    }

    fn tick(&mut self) {
        // Note: This frame ticking stuff is really more just to prevent from constant syscalls
        // in std environments, not intended to very accurately represent time.
        //
        // Though I suppose software could theoretically try to use the baud clock as some sort of
        // time sync, so maybe there will be software that breaks with this?
        //
        // Well whatever, I'll cross that bridge when I get there.
        self.frame_ticks += 1;
        if self.frame_ticks >= self.frame_ticks_max {
            self.frame_reset();
        }
    }

    fn read(&mut self) -> u8 {
        self.status.set_rdrf(false);
        self.rdreg
    }

    fn write(&mut self, data: u8) {
        if self.command.tx_control().rts_assert && self.port.cts_asserted() {
            self.status.set_tdre(false);
            self.port.write(data);
        }
    }

    fn set_control(&mut self, control: Control, force: bool) {
        let old_control = self.control;

        if force || old_control.baud() != control.baud() {
            self.port.set_baud(control.baud());
        }

        if force || old_control.word_length() != control.word_length() {
            self.port.set_word_length(control.word_length());
        }

        if force || old_control.stop_bits() != control.stop_bits() {
            self.port.set_stop_bits(control.stop_bits());
        }

        self.control = control;
        self.adjust_frame();
    }

    fn set_command(&mut self, command: Command, force: bool) {
        let old_command = self.command;

        if force || old_command.dtr() != command.dtr() {
            self.port.dtr_assert(command.dtr());
        }

        if force || old_command.tx_control().rts_assert != command.tx_control().rts_assert {
            self.port.rts_assert(command.tx_control().rts_assert);
        }

        if force || old_command.tx_control().brk_assert != command.tx_control().brk_assert {
            self.port.brk_assert(command.tx_control().brk_assert);
        }

        if force || old_command.parity() != command.parity() {
            self.port.set_parity(command.parity());
        }

        self.command = command;
        self.adjust_frame();
    }

    fn status(&mut self) -> Status {
        let status = self
            .status
            .with_ndcd(!self.port.dcd_asserted())
            .with_ndsr(!self.port.dsr_asserted());

        if !self.command.tx_control().tie {
            self.status.set_irq(false);
        }

        status
    }

    fn set_status(&mut self, _status: Status) {
        self.soft_reset();
    }
}

/// Super Serial Card (SSC).
pub struct SuperSerial<S: SerialPort> {
    rom: [u8; ROM_SIZE],
    acia: Acia6551<S>,
    dipsw1: u8,
    dipsw2: u8,
}

impl<S: SerialPort> SuperSerial<S> {
    /// Create a new instance of a Super Serial Card with the given serial port.
    ///
    /// The ROM must be the 2KB SSC ROM.
    ///
    /// SW1 and SW2 must match the following binary format (ON = 1, OFF = 0):
    ///
    /// |   7   |   6   |   5   |   4   |   3   |   2   |   1   |   0   |
    /// |-------|-------|-------|-------|-------|-------|-------|-------|
    /// |   -   | SWX_7 | SWX_6 | SWX_5 | SWX_4 | SWX_3 | SWX_2 | SWX_1 |
    pub fn new(port: S, rom: [u8; ROM_SIZE], sw1: u8, sw2: u8) -> Self {
        // The below are all active low,
        // and yes, the dipswx registers
        // are jumbled in order like this
        let sw1 = !sw1;
        let sw1_1 = sw1 & 1;
        let sw1_2 = (sw1 >> 1) & 1;
        let sw1_3 = (sw1 >> 2) & 1;
        let sw1_4 = (sw1 >> 3) & 1;
        let sw1_5 = (sw1 >> 4) & 1;
        let sw1_6 = (sw1 >> 5) & 1;
        let dipsw1 =
            (sw1_1 << 7) | (sw1_2 << 6) | (sw1_3 << 5) | (sw1_4 << 4) | (sw1_5 << 1) | sw1_6;

        let sw2 = !sw2;
        let sw2_1 = sw2 & 1;
        let sw2_2 = (sw2 >> 1) & 1;
        let sw2_3 = (sw2 >> 2) & 1;
        let sw2_4 = (sw2 >> 3) & 1;
        let sw2_5 = (sw2 >> 4) & 1;
        let dipsw2 = (sw2_1 << 7) | (sw2_2 << 5) | (sw2_3 << 3) | (sw2_4 << 2) | (sw2_5 << 1);

        SuperSerial {
            rom,
            acia: Acia6551::new(port),
            dipsw1,
            dipsw2,
        }
    }
}

impl<S: SerialPort> Peripheral for SuperSerial<S> {
    fn tick(&mut self, bus: &mut dyn Bus, _pins: &mut super::Pins) {
        if bus.res() {
            self.acia.hard_reset();
        }

        self.acia.tick();
    }

    fn device_select(&mut self, bus: &mut dyn Bus, _pins: &mut super::Pins) {
        let (addr, rw) = ((bus.addr() & 0xF) as u8, bus.op());

        match (addr, rw) {
            (reg_addr::DIPSW1, Op::Read) => {
                // Bit 0 of this register is wired to CTS
                // (I guess to make it visible to user?)
                bus.set_data(self.dipsw1 | !self.acia.port.cts_asserted() as u8)
            }
            (reg_addr::DIPSW2, Op::Read) => bus.set_data(self.dipsw2),

            (reg_addr::XDREG, Op::Read) => bus.set_data(self.acia.read()),
            (reg_addr::XDREG, Op::Write) => self.acia.write(bus.data()),

            (reg_addr::STATUS, Op::Read) => bus.set_data(self.acia.status().into()),
            (reg_addr::STATUS, Op::Write) => self.acia.set_status(bus.data().into()),

            (reg_addr::COMMAND, Op::Read) => bus.set_data(self.acia.command.into()),
            (reg_addr::COMMAND, Op::Write) => self.acia.set_command(bus.data().into(), false),

            (reg_addr::CONTROL, Op::Read) => bus.set_data(self.acia.control.into()),
            (reg_addr::CONTROL, Op::Write) => self.acia.set_control(bus.data().into(), false),

            _ => (),
        }
    }

    fn io_select(&mut self, bus: &mut dyn Bus, _pins: &mut super::Pins) {
        // Note: The SSC maps its IO ROM space to the last page of extended ROM
        let addr = 0x700 | (bus.addr() & 0xFF) as usize;
        bus.set_data(self.rom[addr]);
    }

    fn io_strobe(&mut self, bus: &mut dyn Bus, _pins: &mut super::Pins) {
        let addr = (bus.addr() & 0x7FF) as usize;
        bus.set_data(self.rom[addr]);
    }
}

/// Represents the actual serial port on the host machine.
///
/// If certain methods aren't supported or applicable by the underlying serial port,
/// they can be stubbed out or perform no-ops.
pub trait SerialPort {
    /// Attempt to read a single byte.
    ///
    /// Returns `None` if no data is available.
    fn read(&mut self) -> Option<u8>;
    /// Write a single byte.
    fn write(&mut self, data: u8);
    /// Set the baud rate.
    fn set_baud(&mut self, baud: Baud);
    /// Set the word length.
    fn set_word_length(&mut self, word_length: WordLength);
    /// Set the parity.
    fn set_parity(&mut self, parity: Parity);
    /// Set the number of stop bits.
    fn set_stop_bits(&mut self, stop_bits: StopBits);
    /// Assert or deassert the RTS line.
    fn rts_assert(&mut self, assert: bool);
    /// Assert or deassert the BRK line.
    fn brk_assert(&mut self, assert: bool);
    /// Assert or deassert the DTR line.
    fn dtr_assert(&mut self, assert: bool);
    /// Returns true if the CTS line is asserted.
    fn cts_asserted(&mut self) -> bool;
    /// Returns true if the DSR line is asserted.
    fn dsr_asserted(&mut self) -> bool;
    /// Returns true if the DCD line is asserted.
    fn dcd_asserted(&mut self) -> bool;
    /// Parity error occurred.
    fn parity_err(&mut self) -> bool;
    /// Framing error occurred.
    fn framing_err(&mut self) -> bool;
    /// Buffer overrun occurred.
    fn overrun(&mut self) -> bool;
}
