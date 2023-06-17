use std::{
    cmp::{max, min},
    ops::{Index, IndexMut},
};

use crate::chip8::DISPLAY_HEIGHT as HEIGHT;
use crate::chip8::DISPLAY_WIDTH as WIDTH;

#[derive(Debug)]
pub struct Framebuffer {
    /// Pixel buffer
    buffer: [bool; HEIGHT * WIDTH],
    /// Display has been updated.  Set this to false after redrawing the screen.  
    pub updated: bool,
}

impl Framebuffer {
    pub const fn new() -> Self {
        Self {
            buffer: [false; HEIGHT * WIDTH],
            updated: false,
        }
    }

    /// Unset all pixels
    pub fn clear(&mut self) {
        self.buffer.copy_from_slice(&[false; HEIGHT * WIDTH]);
        self.updated = true;
    }

    /// Draws a sprite at `(x,y)` that has a width of 8 pixels and height of `n` pixels.
    /// `sprite` contains the sprite data.  Sprites drawn at the edge of the screen will be
    /// clipped if `wrap` is false; otherwise, sprites will get drawn at the right coordinates
    /// on the other side of the screen.  Returns true if any pixels are flipped from set
    /// to unset.
    pub fn draw(&mut self, x: u8, y: u8, n: u8, sprite: &[u8], wrap: bool) -> bool {
        let n = usize::from(n);
        assert_eq!(sprite.len(), n);

        let x = usize::from(x) % WIDTH;
        let y = usize::from(y) % HEIGHT;
        let max_x = if wrap { x + 8 } else { min(x + 8, WIDTH) };
        let max_y = if wrap { y + n } else { min(y + n, HEIGHT) };
        let mut ret = false;

        for i in x..max_x {
            for j in y..max_y {
                let sprite_pixel = ((sprite[j - y] >> (7 - (i - x))) & 0x1) == 1;
                if sprite_pixel && self[(i, j)] {
                    self[(i, j)] = false;
                    ret = true;
                } else if sprite_pixel && !self[(i, j)] {
                    self[(i, j)] = true;
                }
            }
        }

        ret
    }

    /// Convert the framebuffer into a color model (e.g., RGB888 or ARGB8888).  A set
    /// pixel is represented by `fg` and an unset pixel is represented by `bg`.
    pub fn to_color_model<T>(&self, fg: &[T], bg: &[T]) -> Vec<T>
    where
        T: Clone,
    {
        let max_cap = max(fg.len(), bg.len()) * HEIGHT * WIDTH;
        let mut ret = Vec::with_capacity(max_cap);
        for pixel in self.buffer {
            if pixel {
                ret.extend_from_slice(fg);
            } else {
                ret.extend_from_slice(bg);
            }
        }

        ret
    }
}

impl Index<(usize, usize)> for Framebuffer {
    type Output = bool;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let x = index.0 % WIDTH;
        let y = index.1 % HEIGHT;
        &self.buffer[y * WIDTH + x]
    }
}

impl IndexMut<(usize, usize)> for Framebuffer {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        self.updated = true;
        let x = index.0 % WIDTH;
        let y = index.1 % HEIGHT;
        &mut self.buffer[y * WIDTH + x]
    }
}
