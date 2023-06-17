use crate::Instruction::*;
use crate::{framebuffer::Framebuffer, instruction::Instruction};

pub const MEMORY_SIZE: usize = 4096;
pub const PROGRAM_START: usize = 0x200;

pub const DISPLAY_HEIGHT: usize = 32;
pub const DISPLAY_WIDTH: usize = 64;

pub const STACK_SIZE: usize = 16;

pub const NUMBER_OF_REGISTERS: usize = 16;

pub const FONT_DATA: [u8; 5 * 16] = [
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
}

impl Chip8 {
    pub fn new(rom: &[u8]) -> Self {
        assert!(rom.len() < MEMORY_SIZE - PROGRAM_START);

        let mut mem: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];
        mem[0..FONT_DATA.len()].copy_from_slice(&FONT_DATA);
        mem[PROGRAM_START..PROGRAM_START + rom.len()].copy_from_slice(rom);

        Self {
            mem,
            fb: Framebuffer::new(),
            v: [0; NUMBER_OF_REGISTERS],
            i: 0,
            pc: PROGRAM_START,
            dt: 0,
            st: 0,
            stack: [0; STACK_SIZE],
            sp: 0,
        }
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
            "Program counter has exceeded memory size"
        );
        u16::from_be_bytes([self.mem[self.pc], self.mem[self.pc + 1]])
    }

    fn execute(&mut self, instr: Instruction) {
        // Increment program counter as this is the default for most instructions
        self.pc += 2;

        match instr {
            Sys(_) => {},
            Cls => {
                self.fb.clear();
            },
            Ret => todo!(),
            Jmp(nnn) => {
                self.pc = nnn;
            },
            Call(_) => todo!(),
            Skeb(_, _) => todo!(),
            Skneb(_, _) => todo!(),
            Ske(_, _) => todo!(),
            Ldb(x, nn) => {
                self.v[x] = nn;
            },
            Addb(_, _) => todo!(),
            Ld(_, _) => todo!(),
            Or(_, _) => todo!(),
            And(_, _) => todo!(),
            Xor(_, _) => todo!(),
            Add(_, _) => todo!(),
            Sub(_, _) => todo!(),
            Shr(_, _) => todo!(),
            Subr(_, _) => todo!(),
            Shl(_, _) => todo!(),
            Skne(_, _) => todo!(),
            Ldi(nnn) => {
                self.i = nnn;
            },
            Jmpz(_) => todo!(),
            Rnd(_, _) => todo!(),
            Draw(x, y, n) => {
                if self.fb.draw(self.v[x], self.v[y], n, &self.mem[self.i..self.i+usize::from(n)], false) {
                    self.v[0xF] = 1;
                } else {
                    self.v[0xF] = 0;
                }
            },
            Skp(_) => todo!(),
            Sknp(_) => todo!(),
            Ldft(_) => todo!(),
            Ldk(_) => todo!(),
            Lddt(_) => todo!(),
            Ldst(_) => todo!(),
            Addi(_) => todo!(),
            Font(_) => todo!(),
            Bcd(_) => todo!(),
            Sreg(_) => todo!(),
            Lreg(_) => todo!(),
            Err(_) => {},
        }
    }
}
