use rand::Rng;

use crate::Instruction::*;
use crate::{framebuffer::Framebuffer, instruction::Instruction};

/// Memory size in bytes
pub const MEMORY_SIZE: usize = 4096;
/// Program start address
pub const PROGRAM_START: usize = 0x200;
/// Display height in pixels
pub const DISPLAY_HEIGHT: usize = 32;
/// Display width in pixels
pub const DISPLAY_WIDTH: usize = 64;
/// Stack size
const STACK_SIZE: usize = 16;
/// Number of 8-bit general purpose registers
const NUMBER_OF_REGISTERS: usize = 16;
/// Number of keys on the keypad
pub const KEYPAD_SIZE: usize = 16;
/// Number of glyphs in the default font
const GLYPH_COUNT: usize = 16;
/// Size (in bytes) of the glyphs in the default font
const GLYPH_SIZE: usize = 5;
/// Default font
const FONT_DATA: [u8; GLYPH_SIZE * GLYPH_COUNT] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

/// CHIP-8 virtual machine
#[derive(Debug)]
pub struct Chip8 {
    /// RAM
    mem: [u8; MEMORY_SIZE],
    /// Display framebuffer
    pub fb: Framebuffer,
    /// 8-bit general purpose registers
    v: [u8; NUMBER_OF_REGISTERS],
    /// Index (address) register
    i: usize,
    /// Program counter
    pc: usize,
    /// Delay timer
    pub dt: u8,
    /// Sound timer
    pub st: u8,
    /// Address stack
    stack: [usize; STACK_SIZE],
    /// Stack pointer
    sp: usize,
    /// Keypad
    pub keypad: [bool; KEYPAD_SIZE],
}

impl Chip8 {
    pub fn new(rom: &[u8]) -> Result<Self, String> {
        if rom.len() >= MEMORY_SIZE - PROGRAM_START {
            return Result::Err("Program is too large to hold in CHIP-8 memory".into());
        }

        let mut mem: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];
        mem[0..FONT_DATA.len()].copy_from_slice(&FONT_DATA);
        mem[PROGRAM_START..PROGRAM_START + rom.len()].copy_from_slice(rom);

        Ok(Self {
            mem,
            fb: Framebuffer::new(),
            v: [0; NUMBER_OF_REGISTERS],
            i: 0,
            pc: PROGRAM_START,
            dt: 0,
            st: 0,
            stack: [0; STACK_SIZE],
            sp: 0,
            keypad: [false; KEYPAD_SIZE],
        })
    }

    pub fn step(&mut self) {
        // Fetch and decode
        let instr = Instruction::from(self.fetch());
        // Execute the instruction
        self.execute(instr);
    }

    fn fetch(&self) -> u16 {
        assert!(
            self.pc + 1 < MEMORY_SIZE,
            "Attempted to read outside of memory bounds"
        );
        u16::from_be_bytes([self.mem[self.pc], self.mem[self.pc + 1]])
    }

    fn execute(&mut self, instr: Instruction) {
        // Increment program counter as this is the default for most instructions
        self.pc += 2;

        match instr {
            Sys(_) => {}
            Cls => {
                self.fb.clear();
            }
            Ret => {
                assert_ne!(self.sp, 0, "Stack underflow");
                self.sp -= 1;
                self.pc = self.stack[self.sp] + 2;
            }
            Jmp(nnn) => {
                self.pc = nnn;
            }
            Call(nnn) => {
                assert_ne!(self.sp, STACK_SIZE, "Stack overflow");
                self.stack[self.sp] = self.pc - 2;
                self.sp += 1;
                self.pc = nnn;
            }
            Skeb(x, nn) => {
                if self.v[x] == nn {
                    self.pc += 2;
                }
            }
            Skneb(x, nn) => {
                if self.v[x] != nn {
                    self.pc += 2;
                }
            }
            Ske(x, y) => {
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            }
            Ldb(x, nn) => {
                self.v[x] = nn;
            }
            Addb(x, nn) => {
                self.v[x] = self.v[x].wrapping_add(nn);
            }
            Ld(x, y) => {
                self.v[x] = self.v[y];
            }
            Or(x, y) => {
                self.v[x] |= self.v[y];
            }
            And(x, y) => {
                self.v[x] &= self.v[y];
            }
            Xor(x, y) => {
                self.v[x] ^= self.v[y];
            }
            Add(x, y) => {
                let (value, overflow) = self.v[x].overflowing_add(self.v[y]);
                self.v[x] = value;
                if overflow {
                    self.v[0xF] = 1;
                } else {
                    self.v[0xF] = 0;
                }
            }
            Sub(x, y) => {
                let (value, overflow) = self.v[x].overflowing_sub(self.v[y]);
                self.v[x] = value;
                if overflow {
                    self.v[0xF] = 0;
                } else {
                    self.v[0xF] = 1;
                }
            }
            Shr(x, _) => {
                self.v[0xF] = self.v[x] & 0x1;
                self.v[x] >>= 1;
            }
            Subr(x, y) => {
                let (value, overflow) = self.v[y].overflowing_sub(self.v[x]);
                self.v[x] = value;
                if overflow {
                    self.v[0xF] = 0;
                } else {
                    self.v[0xF] = 1;
                }
            }
            Shl(x, _) => {
                self.v[0xF] = (self.v[x] & 0b1000_0000) >> 7;
                self.v[x] <<= 1;
            }
            Skne(x, y) => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
            Ldi(nnn) => {
                self.i = nnn;
            }
            Jmpz(nnn) => {
                self.pc = nnn + usize::from(self.v[0]);
            }
            Rnd(x, nn) => {
                self.v[x] = rand::thread_rng().gen::<u8>() & nn;
            }
            Draw(x, y, n) => {
                if self.fb.draw(
                    self.v[x],
                    self.v[y],
                    n,
                    &self.mem[self.i..self.i + usize::from(n)],
                    false,
                ) {
                    self.v[0xF] = 1;
                } else {
                    self.v[0xF] = 0;
                }
            }
            Skp(x) => {
                let key = usize::from(self.v[x]);
                assert!(key < KEYPAD_SIZE, "{:#X} is not a valid key", key);
                if self.keypad[key] {
                    self.pc += 2;
                }
            }
            Sknp(x) => {
                let key = usize::from(self.v[x]);
                assert!(key < KEYPAD_SIZE, "{:#X} is not a valid key", key);
                if !self.keypad[key] {
                    self.pc += 2;
                }
            }
            Ldft(x) => {
                self.v[x] = self.dt;
            }
            Ldk(x) => {
                let pressed_keys: Vec<u8> = self
                    .keypad
                    .into_iter()
                    .enumerate()
                    .filter_map(|(i, val)| {
                        if val {
                            Some(u8::try_from(i).unwrap())
                        } else {
                            None
                        }
                    })
                    .collect();
                if pressed_keys.is_empty() {
                    self.pc -= 2;
                } else {
                    self.v[x] = pressed_keys[0];
                }
            }
            Lddt(x) => {
                self.dt = self.v[x];
            }
            Ldst(x) => {
                self.st = self.v[x];
            }
            Addi(x) => {
                self.i += usize::from(self.v[x]);
            }
            Font(x) => {
                let digit = usize::from(self.v[x]);
                assert!(
                    digit < GLYPH_COUNT,
                    "{:#X} is not a valid glyph in the default font",
                    digit
                );
                self.i = GLYPH_SIZE * digit;
            }
            Bcd(x) => {
                assert!(
                    self.i + 2 < MEMORY_SIZE,
                    "Attempted to write outside of memory bounds"
                );
                self.mem[self.i] = self.v[x] / 100;
                self.mem[self.i + 1] = (self.v[x] / 10) % 10;
                self.mem[self.i + 2] = self.v[x] % 10;
            }
            Sreg(x) => {
                assert!(
                    self.i + x < MEMORY_SIZE,
                    "Attempted to write outside of memory bounds"
                );
                for offset in 0..=x {
                    self.mem[self.i + offset] = self.v[offset];
                }
            }
            Lreg(x) => {
                assert!(
                    self.i + x < MEMORY_SIZE,
                    "Attempted to read outside of memory bounds"
                );
                for offset in 0..=x {
                    self.v[offset] = self.mem[self.i + offset];
                }
            }
            Err(_) => {}
        }
    }
}
