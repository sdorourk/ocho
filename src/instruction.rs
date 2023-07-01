use std::fmt::Display;

use Instruction::*;
/// Chip-8 instruction set.
///
/// Doc-comments use the following variables:
///  - nnn - a 12-bit value, the lowester 12 bits of the instruction
///  - n - a 4-bit value, the lowest 4 bits of the instruction
///  - x - a 4-bit value, the lower 4 bits of the high byte of the instruction
///  - y - a 4-bit value, the upper 4 bits of the low byte of the instruction
///  - nn - an 8-bit value, the lowest 8 bits of the instruction
#[derive(Debug)]
pub enum Instruction {
    /// 0nnn - SYS nnn. Jump to machine code routine at nnn (ignored in modern interpreters).
    Sys(usize),
    /// 00E0 - CLS. Clear the display.
    Cls,
    /// 00EE - RET.  Return from a subroutine.
    Ret,
    /// 1nnn - JMP nnn.  Jump to address nnn.   
    Jmp(usize),
    /// 2nnn - CALL nnn.  Call subroutine at nnn.  
    Call(usize),
    /// 3xnn - SKEB Vx, nn.  Skip next instruction if Vx == nn.  
    Skeb(usize, u8),
    /// 4xnn - SKNEB Vx, nn.  Skip next instruction if Vx != nn.
    Skneb(usize, u8),
    /// 5xy0 - SKE Vx, Vy.  Skip next instruction if Vx == Vy.
    Ske(usize, usize),
    /// 6xnn - LDB Vx, nn.  Set Vx = nn.  
    Ldb(usize, u8),
    /// 7xnn - ADDB Vx, nn.  Set Vx = Vx + nn.  
    Addb(usize, u8),
    /// 8xy0 - LD Vx, Vy.  Set Vx = Vy.  
    Ld(usize, usize),
    /// 8xy1 - OR Vx, Vy.  Set Vx = Vx OR Vy.  
    Or(usize, usize),
    /// 8xy2 - AND Vx, Vy.  Set Vx = Vx AND Vy.
    And(usize, usize),
    /// 8xy3 - XOR Vx, Vy.  Set Vx = Vx XOR Vy.  
    Xor(usize, usize),
    /// 8xy4 - ADD Vx, Vy.  Set Vx = Vx + Vy.  VF is set to 1 if there is a carry, otherwise 0.
    Add(usize, usize),
    /// 8xy5 - SUB Vx, Vy.  Set Vx = Vx - Vy.  If Vx > Vy, then VF is set to 1, otherwise 0.
    Sub(usize, usize),
    /// 8xy6 - SHR Vx, Vy.  Set VF to the least-significant bit of VX, then set Vx = Vx >> 1.
    Shr(usize, usize),
    /// 8xy7 - SUBR Vx, Vy.  Set Vx = Vy - Vx.  If Vy > Vx, then VF is set to 1, otherwise 0.
    Subr(usize, usize),
    /// 8xyE - SHL Vx, Vy.  Set VF to the most-significant bit of Vx, then set Vx = Vx << 1.
    Shl(usize, usize),
    /// 9xy0 - SKNE Vx, Vy.  Skip next instruction if Vx != Vy.
    Skne(usize, usize),
    /// Annn - LDI nnn.  Set I (the index register) to nnn.  
    Ldi(usize),
    /// Bnnn - JMPZ nnn.  Jump to address nnn + V0.
    Jmpz(usize),
    /// Cxnn - RND Vx, nn.  Set Vx = random byte AND nn).
    Rnd(usize, u8),
    /// Dxyn - DRAW Vx, Vy, n.  Draw a sprite of height n to the framebuffer, starting at
    /// coordinate (Vx, Vy).  Sprite data is stored in memory, starting at I.  
    Draw(usize, usize, u8),
    /// Ex9E - SKP Vx.  Skip next instruction if key with the value Vx is pressed.
    Skp(usize),
    /// ExA1 - SKNP Vx.  Skip next instruction if key with the value of Vx is not pressed.
    Sknp(usize),
    /// Fx07 - LDDT Vx.  Set Vx to the value of the delay timer.  
    Ldft(usize),
    /// Fx0A - LDK Vx.  Wait for a key release and store the value of the key in Vx.
    Ldk(usize),
    /// Fx15 - LDDT Vx.  Set the delay timer to Vx.  
    Lddt(usize),
    /// Fx18 - LDST Vx.  Set the sound timer to Vx.  
    Ldst(usize),
    /// Fx1E - ADDI Vx.  Set I = I + Vx.  
    Addi(usize),
    /// Fx29 - FONT Vx.  Set I to the location of font data for digit Vx.  
    Font(usize),
    /// Fx33 - BCD Vx.  Store the binary-coded decimal representation of Vx into memory with
    /// the hundreds digit at location I, the tens digit at location I+1, and the ones digit
    /// at location I+2.  
    Bcd(usize),
    /// Fx55 - SREG Vx.  Store registers V0 through Vx in memory starting at location I.
    Sreg(usize),
    /// Fx65 - LREG Vx.  Read registers V0 through Vx from memory starting at location I.
    Lreg(usize),
    /// Unrecognized instruction.  
    Err(u16),
}

impl From<u16> for Instruction {
    fn from(value: u16) -> Self {
        let i = (value & 0xF000) >> 12;
        let x = usize::from((value & 0x0F00) >> 8);
        let y = usize::from((value & 0x00F0) >> 4);
        let n = u8::try_from(value & 0x000F).unwrap();
        let nnn = usize::from(value & 0x0FFF);
        let nn = u8::try_from(value & 0x00FF).unwrap();

        match i {
            0 => match nnn {
                0x0E0 => Cls,
                0x0EE => Ret,
                _ => Sys(nnn),
            },
            1 => Jmp(nnn),
            2 => Call(nnn),
            3 => Skeb(x, nn),
            4 => Skneb(x, nn),
            5 => match n {
                0 => Ske(x, y),
                _ => Err(value),
            },
            6 => Ldb(x, nn),
            7 => Addb(x, nn),
            8 => match n {
                0 => Ld(x, y),
                1 => Or(x, y),
                2 => And(x, y),
                3 => Xor(x, y),
                4 => Add(x, y),
                5 => Sub(x, y),
                6 => Shr(x, y),
                7 => Subr(x, y),
                0xE => Shl(x, y),
                _ => Err(value),
            },
            9 => match n {
                0 => Skne(x, y),
                _ => Err(value),
            },
            0xA => Ldi(nnn),
            0xB => Jmpz(nnn),
            0xC => Rnd(x, nn),
            0xD => Draw(x, y, n),
            0xE => match nn {
                0x9E => Skp(x),
                0xA1 => Sknp(x),
                _ => Err(value),
            },
            0xF => match nn {
                0x07 => Ldft(x),
                0x0A => Ldk(x),
                0x15 => Lddt(x),
                0x18 => Ldst(x),
                0x1E => Addi(x),
                0x29 => Font(x),
                0x33 => Bcd(x),
                0x55 => Sreg(x),
                0x65 => Lreg(x),
                _ => Err(value),
            },
            _ => Err(value),
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Sys(nnn) => write!(f, "{:<5} {:#05X}", "SYS", nnn),
            Cls => write!(f, "{:<5}", "CLS"),
            Ret => write!(f, "{:<5}", "RET"),
            Jmp(nnn) => write!(f, "{:<5} {:#05X}", "JMP", nnn),
            Call(nnn) => write!(f, "{:<5} {:#05X}", "CALL", nnn),
            Skeb(x, nn) => write!(f, "{:<5} V{:X}, {:#04X}", "SKEB", x, nn),
            Skneb(x, nn) => write!(f, "{:<5} V{:X}, {:#04X}", "SKNEB", x, nn),
            Ske(x, y) => write!(f, "{:<5} V{:X}, V{:X}", "SKE", x, y),
            Ldb(x, nn) => write!(f, "{:<5} V{:X}, {:#04X}", "LDB", x, nn),
            Addb(x, nn) => write!(f, "{:<5} V{:X}, {:#04X}", "ADDB", x, nn),
            Ld(x, y) => write!(f, "{:<5} V{:X}, V{:X}", "LD", x, y),
            Or(x, y) => write!(f, "{:<5} V{:X}, V{:X}", "OR", x, y),
            And(x, y) => write!(f, "{:<5} V{:X}, V{:X}", "AND", x, y),
            Xor(x, y) => write!(f, "{:<5} V{:X}, V{:X}", "XOR", x, y),
            Add(x, y) => write!(f, "{:<5} V{:X}, V{:X}", "ADD", x, y),
            Sub(x, y) => write!(f, "{:<5} V{:X}, V{:X}", "SUB", x, y),
            Shr(x, y) => write!(f, "{:<5} V{:X}, V{:X}", "SHR", x, y),
            Subr(x, y) => write!(f, "{:<5} V{:X}, V{:X}", "SUBR", x, y),
            Shl(x, y) => write!(f, "{:<5} V{:X}, V{:X}", "SHL", x, y),
            Skne(x, y) => write!(f, "{:<5} V{:X}, V{:X}", "SKNE", x, y),
            Ldi(nnn) => write!(f, "{:<5} {:#05X}", "LDI", nnn),
            Jmpz(nnn) => write!(f, "{:<5} {:#05X}", "JMPZ", nnn),
            Rnd(x, nn) => write!(f, "{:<5} V{:X}, {:#04X}", "RND", x, nn),
            Draw(x, y, n) => write!(f, "{:<5} V{:X}, V{:X}, {:#03X}", "DRAW", x, y, n),
            Skp(x) => write!(f, "{:<5} V{:X}", "SKP", x),
            Sknp(x) => write!(f, "{:<5} V{:X}", "SKNP", x),
            Ldft(x) => write!(f, "{:<5} V{:X}", "LDFT", x),
            Ldk(x) => write!(f, "{:<5} V{:X}", "LDK", x),
            Lddt(x) => write!(f, "{:<5} V{:X}", "LDDT", x),
            Ldst(x) => write!(f, "{:<5} V{:X}", "LDST", x),
            Addi(x) => write!(f, "{:<5} V{:X}", "ADDI", x),
            Font(x) => write!(f, "{:<5} V{:X}", "FONT", x),
            Bcd(x) => write!(f, "{:<5} V{:X}", "BCD", x),
            Sreg(x) => write!(f, "{:<5} V{:X}", "SREG", x),
            Lreg(x) => write!(f, "{:<5} V{:X}", "LREG", x),
            Err(instr) => write!(f, "{:<5} {:#06X}", "ERR", instr),
        }
    }
}
