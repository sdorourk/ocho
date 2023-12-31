use std::time::{Duration, Instant};

use sdl2::{
    audio::{AudioCallback, AudioSpecDesired},
    event::Event,
    keyboard::{Keycode, Scancode},
    pixels::PixelFormatEnum,
};

use crate::chip8::{Chip8, Quirks, DISPLAY_HEIGHT, DISPLAY_WIDTH};

pub struct Emulator {
    chip: Chip8,
    options: Options,
}

#[derive(Debug)]
pub struct Options {
    // Frames per second
    pub fps: u16,
    /// Instructions executed per frame
    pub ipf: u16,
    /// Window scale factor
    pub scale: u32,
    /// Foreground color (RGBA8888)
    pub fg: u32,
    /// Background color (RGBA8888)
    pub bg: u32,
    /// Pitch of the buzzer (in Hz)
    pub pitch: u16,
    /// Limit only one draw operation per frame
    pub display_wait: bool,
}

impl Emulator {
    pub fn new(rom: &[u8], options: Options, quirks: Quirks) -> Result<Self, String> {
        let chip = Chip8::new(rom, quirks)?;
        Ok(Self { chip, options })
    }

    pub fn run(&mut self) -> Result<(), String> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let audio_subsystem = sdl_context.audio()?;

        // Required to avoid excessive conversions
        const HEIGHT: u32 = DISPLAY_HEIGHT as u32;
        const WIDTH: u32 = DISPLAY_WIDTH as u32;

        // Initialize the window
        let window = video_subsystem
            .window(
                "CHIP-8 Emulator",
                WIDTH * self.options.scale,
                HEIGHT * self.options.scale,
            )
            .position_centered()
            .resizable()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        let texture_creator = canvas.texture_creator();
        canvas
            .set_logical_size(WIDTH, HEIGHT)
            .map_err(|e| e.to_string())?;
        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGBA32, WIDTH, HEIGHT)
            .map_err(|e| e.to_string())?;

        // Initialize the audio
        let desired_audio_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: None,
        };
        let audio_device = audio_subsystem.open_playback(None, &desired_audio_spec, |spec| {
            let freq = if spec.freq < 0 {
                i64::from(-spec.freq)
            } else {
                i64::from(spec.freq)
            };
            let pitch = i64::from(self.options.pitch);
            SquareWave {
                channels: usize::from(spec.channels),
                half_period: freq / (2 * pitch),
                volume: 0.25,
                index: 0,
            }
        })?;

        // Colors as RGBA values
        let fg = self.options.fg.to_be_bytes();
        let bg = self.options.bg.to_be_bytes();

        let mut event_pump = sdl_context.event_pump()?;
        let nanos_per_frame: u128 =
            Duration::from_secs(1).as_nanos() / u128::from(self.options.fps);

        'running: loop {
            let start = Instant::now();
            for _ in 0..self.options.ipf {
                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit { .. }
                        | Event::KeyDown {
                            keycode: Some(Keycode::Escape),
                            ..
                        } => break 'running,
                        Event::KeyDown {
                            scancode: Some(scancode),
                            ..
                        } => {
                            if let Some(key) = self.keymap(scancode) {
                                self.chip.keypad.key_pressed(key);
                            }
                        }
                        Event::KeyUp {
                            scancode: Some(scancode),
                            ..
                        } => {
                            if let Some(key) = self.keymap(scancode) {
                                self.chip.keypad.key_released(key);
                            }
                        }
                        _ => {}
                    }
                }
                self.chip.step();

                if self.chip.st > 0 {
                    audio_device.resume();
                } else {
                    audio_device.pause();
                }
                if self.chip.fb.updated {
                    let pixels = self.chip.fb.to_color_model(&fg, &bg);
                    texture.with_lock(None, |buffer: &mut [u8], _: usize| {
                        buffer.copy_from_slice(&pixels);
                    })?;
                    self.chip.fb.updated = false;
                    if self.options.display_wait {
                        break;
                    }
                }
            }
            canvas.clear();
            canvas.copy(&texture, None, None)?;
            canvas.present();

            if self.chip.st > 0 {
                self.chip.st -= 1;
            }
            if self.chip.dt > 0 {
                self.chip.dt -= 1;
            }

            let elapsed_nanos = start.elapsed().as_nanos();
            if elapsed_nanos < nanos_per_frame {
                let sleep_duration = u64::try_from(nanos_per_frame - elapsed_nanos).unwrap_or(0);
                std::thread::sleep(Duration::from_nanos(sleep_duration));
            }
        }
        Ok(())
    }

    fn keymap(&self, scancode: Scancode) -> Option<u8> {
        match scancode {
            Scancode::Num1 => Some(0x1),
            Scancode::Num2 => Some(0x2),
            Scancode::Num3 => Some(0x3),
            Scancode::Num4 => Some(0xC),
            Scancode::Q => Some(0x4),
            Scancode::W => Some(0x5),
            Scancode::E => Some(0x6),
            Scancode::R => Some(0xD),
            Scancode::A => Some(0x7),
            Scancode::S => Some(0x8),
            Scancode::D => Some(0x9),
            Scancode::F => Some(0xE),
            Scancode::Z => Some(0xA),
            Scancode::X => Some(0x0),
            Scancode::C => Some(0xB),
            Scancode::V => Some(0xF),
            _ => None,
        }
    }
}

struct SquareWave {
    channels: usize,
    half_period: i64,
    volume: f32,
    index: i64,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        for x in out.chunks_mut(self.channels) {
            if self.index / self.half_period >= 2 {
                self.index = 0;
            }
            for vol in x {
                *vol = if self.index / self.half_period == 0 {
                    self.volume
                } else {
                    -self.volume
                };
            }
            self.index += 1;
        }
    }
}
