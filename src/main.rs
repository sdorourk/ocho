/////////////////////////////////////////////////////////
/// Todo:
///  - Add quirks.
///  - Add options
///  - Implement SDL
///  - Check for errors and test
/////////////////////////////////////////////////////////
mod chip8;
mod emulator;
mod framebuffer;
mod instruction;

use chip8::{Chip8, MEMORY_SIZE, PROGRAM_START};
use clap::Parser;
use emulator::Emulator;
use instruction::Instruction;
use std::{fs::read, path::PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the binary CHIP-8 program to run
    program: PathBuf,
}

fn main() {
    let args = Cli::parse();

    let rom = match read(&args.program) {
        Ok(rom) => rom,
        Err(err) => {
            eprintln!(
                "The file \'{}\' could not be opened: {}",
                args.program.display(),
                err
            );
            return;
        }
    };

    if rom.is_empty() {
        eprintln!(
            "The file \'{}\' is not a valid CHIP-8 program",
            args.program.display()
        );
        return;
    } else if rom.len() > MEMORY_SIZE - PROGRAM_START {
        eprintln!(
            "The file \'{}\' is too large to fit in memory",
            args.program.display()
        );
        return;
    }
    disassemble(&rom);

    let mut emu = Emulator::new(&rom).unwrap();
    emu.run();
}

fn disassemble(rom: &[u8]) {
    let rom: Vec<u16> = rom
        .chunks(2)
        .map(|x| {
            if x.len() == 2 {
                u16::from_be_bytes([x[0], x[1]])
            } else {
                u16::from_be_bytes([x[0], 0])
            }
        })
        .collect();
    let mut addr = PROGRAM_START;
    for instr in rom {
        println!("{:#06X}: {}", addr, Instruction::from(instr));
        addr += 2;
    }
}

fn interactive_debugger() {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}

fn print_display(chip: &Chip8) {
    let disp = chip.fb.to_color_model(&[true], &[false]);
    for y in 0..chip8::DISPLAY_HEIGHT {
        for x in 0..chip8::DISPLAY_WIDTH {
            if disp[y * chip8::DISPLAY_WIDTH + x] {
                print!("X");
            } else {
                print!(" ");
            }
        }
        println!("");
    }
}
