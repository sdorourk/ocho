use std::time::Instant;

use sdl2::{
    audio::{AudioCallback, AudioSpecDesired},
    event::Event,
    keyboard::Keycode,
    pixels::PixelFormatEnum,
};

use crate::chip8::{Chip8, DISPLAY_HEIGHT, DISPLAY_WIDTH};

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
    /// Display scale factor
    pub scale: u32,
    /// Foreground color (ARGB8888)
    pub fg: u32,
    /// Background color (ARGB8888)
    pub bg: u32,
    /// Pitch of the buzzer (in Hz)
    pub pitch: u16,
    /// Limit only one draw operation per frame
    pub display_wait: bool,
}

impl Emulator {
    pub fn new(rom: &[u8], options: Options) -> Result<Self, String> {
        let chip = Chip8::new(rom)?;

        Ok(Self { chip, options })
    }

    pub fn run(&mut self) -> Result<(), String> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let audio_subsystem = sdl_context.audio()?;

        // We need the height and width as u32
        let height = u32::try_from(DISPLAY_HEIGHT).unwrap();
        let width = u32::try_from(DISPLAY_WIDTH).unwrap();

        // Initialize the window
        let window = video_subsystem
            .window("CHIP-8 Emulator", width * self.options.scale, height * self.options.scale)
            .position_centered()
            .resizable()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        let texture_creator = canvas.texture_creator();
        canvas
            .set_logical_size(width, height)
            .map_err(|e| e.to_string())?;
        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGBA32, width, height)
            .map_err(|e| e.to_string())?;

        // Initialize the audio
        let desired_audio_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: None,
        };
        let audio_device =
            audio_subsystem.open_playback(None, &desired_audio_spec, |spec| SquareWave {
                phase_inc: 440.0 / f64::try_from(spec.freq).unwrap_or(44100.0),
                phase: 0.0,
                volume: 0.25,
            })?;

        // Create the event pump
        let mut event_pump = sdl_context.event_pump()?;
        let nano_seconds_per_frame: u128 = 1_000_000_000 / u128::from(self.options.fps);
        
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
                    let pixels = self
                        .chip
                        .fb
                        .to_color_model(&[0xFF, 0xFF, 0xFF, 0xFF], &[0, 0, 0, 0]);
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

            let elapsed_nano_seconds = start.elapsed().as_nanos();
            if elapsed_nano_seconds < nano_seconds_per_frame {
                let sleep_duration = u64::try_from(nano_seconds_per_frame - elapsed_nano_seconds).unwrap_or(0);
                std::thread::sleep(std::time::Duration::from_nanos(sleep_duration));
            }
        }
        Ok(())
    }
}

struct SquareWave {
    phase_inc: f64,
    phase: f64,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}
