use sdl2::{
    audio::{AudioCallback, AudioSpecDesired},
    event::Event,
    keyboard::Keycode,
    pixels::PixelFormatEnum,
};

use crate::chip8::{Chip8, DISPLAY_HEIGHT, DISPLAY_WIDTH};

pub struct Emulator {
    chip: Chip8,
}

impl Emulator {
    pub fn new(rom: &[u8]) -> Result<Self, String> {
        let chip = Chip8::new(rom)?;

        Ok(Self { chip })
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
            .window("CHIP-8 Emulator", width * 10, height * 10)
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
            .create_texture_streaming(PixelFormatEnum::ARGB8888, width, height)
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

        'running: loop {
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
            }
            canvas.clear();
            canvas.copy(&texture, None, None)?;
            canvas.present();

            std::thread::sleep(std::time::Duration::from_millis(10));
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
