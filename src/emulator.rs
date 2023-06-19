use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::PixelFormatEnum,
    render::{Canvas, TextureCreator},
    video::{Window, WindowContext},
    AudioSubsystem, Sdl, VideoSubsystem,
};

use crate::{
    chip8::{Chip8, DISPLAY_HEIGHT, DISPLAY_WIDTH},
    interactive_debugger, print_display,
};

pub struct Emulator {
    chip: Chip8,
    sdl_context: Sdl,
    video_subsystem: VideoSubsystem,
    audio_subsystem: AudioSubsystem,
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
    height: u32,
    width: u32,
}

impl Emulator {
    pub fn new(rom: &[u8]) -> Result<Self, String> {
        let chip = Chip8::new(rom)?;

        let height = u32::try_from(DISPLAY_HEIGHT).unwrap();
        let width = u32::try_from(DISPLAY_WIDTH).unwrap();

        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let audio_subsystem = sdl_context.audio()?;

        let window = video_subsystem
            .window("CHIP-8", width * 10, height * 10)
            .position_centered()
            .resizable()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        let texture_creator = canvas.texture_creator();

        Ok(Self {
            chip,
            sdl_context,
            video_subsystem,
            audio_subsystem,
            canvas,
            texture_creator,
            height,
            width,
        })
    }

    pub fn run(&mut self) -> Result<(), String> {
        self.canvas
            .set_logical_size(self.width, self.height)
            .map_err(|e| e.to_string())?;
        let mut texture = self
            .texture_creator
            .create_texture_streaming(PixelFormatEnum::ARGB8888, self.width, self.height)
            .map_err(|e| e.to_string())?;
        let mut event_pump = self.sdl_context.event_pump()?;

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
            self.canvas.clear();
            self.canvas.copy(&texture, None, None)?;
            self.canvas.present();

            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        Ok(())
    }
}
