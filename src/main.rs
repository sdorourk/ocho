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
use clap::{Parser, value_parser};
use emulator::{Emulator, Options};
use instruction::Instruction;
use std::{fs::read, path::PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the binary CHIP-8 program to run
    program: PathBuf,
    /// Target frames per second 
    #[arg(short, long, default_value_t = 60, value_parser = value_parser!(u16).range(1..))]
    fps: u16,
    /// Target instructions per frame
    #[arg(short, long, default_value_t = 10, value_parser = value_parser!(u16).range(1..))]
    ipf: u16,
    /// Window scale factor
    #[arg(short, long, default_value_t = 10, value_parser = value_parser!(u32).range(1..))]
    scale: u32,
    /// Pitch of the buzzer (in Hz)
    #[arg(short, long, default_value_t = 440, value_parser = value_parser!(u16).range(1..))]
    pitch: u16,
    /// Limit only one draw operation per frame
    #[arg(short, long)] 
    display_wait: bool,
}

fn main() {
    let cli = Cli::parse();

    let rom = match read(&cli.program) {
        Ok(rom) => rom,
        Err(err) => {
            eprintln!(
                "The file \'{}\' could not be opened: {}",
                cli.program.display(),
                err
            );
            return;
        }
    };

    if rom.is_empty() {
        eprintln!(
            "The file \'{}\' is not a valid CHIP-8 program",
            cli.program.display()
        );
        return;
    } else if rom.len() > MEMORY_SIZE - PROGRAM_START {
        eprintln!(
            "The file \'{}\' is too large to fit in memory",
            cli.program.display()
        );
        return;
    }
    disassemble(&rom);

    let options = Options {
        fps: cli.fps,
        ipf: cli.ipf,
        scale: cli.scale,
        fg: 0xffffff,
        bg: 0,
        pitch: cli.pitch,
        display_wait: cli.display_wait,
    }; 

    let mut emu = Emulator::new(&rom, options).unwrap();
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
