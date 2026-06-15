use grok_apple2::peripheral::{disk, language, serial};
use grok_apple2::{Apple2, settings};
use sdl2::EventPump;
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use serialport::{SerialPort, TTYPort};
use std::io::{Read, Write};
use std::process::Command;
use std::time::{Duration, Instant};

// The correct ROM files must be placed at the paths below
// They are not included since they are technically still under copyright
// So make sure you only use ROMs that you have the legal right to use (lol)
const FW_ROM: [u8; 0x3000] = *include_bytes!("../roms/apple2_plus.rom");
const CHAR_ROM: [u8; 0x800] = *include_bytes!("../roms/char_set.rom");
const DISK2_ROM: [u8; 0x100] = *include_bytes!("../roms/disk2.rom");
const SSC_ROM: [u8; 0x800] = *include_bytes!("../roms/ssc.rom");

const FRAME_RATE: u32 = 60;
const US_PER_FRAME: u64 = 1000000 / FRAME_RATE as u64;
const SAMPLE_BUF_SZ: usize = 1024;
const SAMPLE_VOLUME: f32 = 0.5;

// TODO: Make this more flexible by also allowing user to pass in existing port etc
struct StdSerialPort(TTYPort);
impl StdSerialPort {
    fn new() -> Self {
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

struct SdlDisplay {
    canvas: Canvas<Window>,
    texture: Texture,
}

impl SdlDisplay {
    fn draw(&mut self, frame: &[u32]) {
        self.texture
            .with_lock(None, |buf, pitch| {
                for (src, dst) in frame
                    .chunks_exact(settings::DISP_WIDTH as usize)
                    .zip(buf.chunks_exact_mut(pitch))
                {
                    for (px, slot) in src.iter().zip(dst.chunks_exact_mut(4)) {
                        slot.copy_from_slice(&px.to_ne_bytes());
                    }
                }
            })
            .unwrap();
        self.canvas.copy(&self.texture, None, None).unwrap();
        self.canvas.present();
    }
}

pub struct SquareWave {
    buffer: [f32; SAMPLE_BUF_SZ],
    sample_idx: usize,
    buf_idx: usize,
}

impl SquareWave {
    pub fn insert_sample(&mut self, sample: f32) {
        self.buffer[self.buf_idx] = sample;
        self.buf_idx += 1;
        self.buf_idx %= SAMPLE_BUF_SZ;
    }
}

struct SdlAudio {
    device: AudioDevice<SquareWave>,
}

impl SdlAudio {
    fn new(sdl_context: &sdl2::Sdl) -> Self {
        let audio_subsystem = sdl_context.audio().unwrap();

        let audio_spec = AudioSpecDesired {
            freq: Some(settings::SAMPLE_RATE as i32),
            channels: Some(1),
            samples: Some(512),
        };

        let wave = SquareWave {
            buffer: [0.0; SAMPLE_BUF_SZ],
            sample_idx: 0,
            buf_idx: 0,
        };

        let device = audio_subsystem
            .open_playback(None, &audio_spec, |_| wave)
            .unwrap();
        device.resume();

        SdlAudio { device }
    }

    fn insert_samples(&mut self, samples: &[bool]) {
        let mut lock = self.device.lock();
        for s in samples {
            lock.insert_sample(match s {
                true => SAMPLE_VOLUME,
                false => -SAMPLE_VOLUME,
            });
        }
    }
}

impl grok_apple2::Audio for SdlAudio {
    fn feed_samples(&mut self, samples: &[bool]) {
        self.insert_samples(samples);
    }
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            if self.sample_idx == self.buf_idx {
                *x = 0.0;
            } else {
                *x = self.buffer[self.sample_idx];
                self.sample_idx += 1;
                self.sample_idx %= SAMPLE_BUF_SZ;
            }
        }
    }
}

fn handle_input(apple2: &mut Apple2<SdlAudio>, event_pump: &mut EventPump) -> bool {
    // TODO: Escape keys, and will need to change key for reset()

    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } => {
                return false;
            }
            Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => {
                apple2.reset();
            }
            Event::KeyDown {
                keycode: Some(keycode),
                keymod,
                ..
            } => {
                // Special case for arrow keys because they don't have an ASCII code
                if keycode == Keycode::Right {
                    apple2.input_arrow(true);
                } else if keycode == Keycode::Left {
                    apple2.input_arrow(false);
                } else {
                    let shift = keymod.contains(Mod::LSHIFTMOD) || keymod.contains(Mod::RSHIFTMOD);
                    let ctrl = keymod.contains(Mod::LCTRLMOD) || keymod.contains(Mod::RCTRLMOD);
                    apple2.input(keycode as u8, shift, ctrl);
                }
            }
            _ => {}
        }
    }

    true
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Initialize SDL
    let sdl_context = sdl2::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    // Initialize video
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(
            "Apple ][+",
            settings::DISP_WIDTH * settings::DISP_SCALE,
            settings::DISP_HEIGHT * settings::DISP_SCALE,
        )
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator
        .create_texture_streaming(
            PixelFormatEnum::RGB888,
            settings::DISP_WIDTH,
            settings::DISP_HEIGHT,
        )
        .unwrap();
    let mut display = SdlDisplay { canvas, texture };

    // Initialize audio
    let audio = SdlAudio::new(&sdl_context);

    // Initialize peripherals
    let mut language_card = language::LanguageCard::new();

    // Setup for ADTPro
    let sw1 = 0b1111001;
    let sw2 = 0b0011011;
    let mut serial_card = serial::SuperSerial::new(StdSerialPort::new(), SSC_ROM, sw1, sw2);

    let mut disk_card =
        disk::ControllerCard::new(DISK2_ROM, grok_apple2::settings::CPU_CLK_SPEED as usize);

    // Insert disk
    if args.len() > 1 {
        let disk_file = &args[1];
        let buffer = std::fs::read(disk_file).unwrap();
        let ext = std::path::Path::new(disk_file)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        match ext {
            "woz" => disk_card.insert_woz(&buffer),
            "dsk" => disk_card.insert_dsk(&buffer),
            "po" => disk_card.insert_po(&buffer),
            _ => panic!("Unsupported disk format: .{}", ext),
        }
    }

    // Initialize Apple 2
    let mut apple2 = Apple2::new(FW_ROM, CHAR_ROM, audio);
    apple2.insert_peripheral(&mut language_card, 0);
    apple2.insert_peripheral(&mut serial_card, 2);
    apple2.insert_peripheral(&mut disk_card, 6);
    apple2.init();

    // Main loop
    while handle_input(&mut apple2, &mut event_pump) {
        let start_time = Instant::now();
        let frame = apple2.run_frame();
        display.draw(frame);
        let elapsed = Duration::from_micros(start_time.elapsed().as_micros() as u64);

        // Sleep for rest of frame period
        let frame = Duration::from_micros(US_PER_FRAME);
        if frame > elapsed {
            std::thread::sleep(frame - elapsed);
        } else {
            eprintln!("Missed frame!");
        }
    }
}
