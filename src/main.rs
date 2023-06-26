/////////////////////////////////////////////////////////
/// Todo:
///  - Add quirks
///  - Keypad support
///  - Document
///  - Check for errors and test
/////////////////////////////////////////////////////////
mod chip8;
mod emulator;
mod framebuffer;
mod instruction;

use chip8::PROGRAM_START;
use clap::{value_parser, Parser};
use emulator::{Emulator, Options};
use instruction::Instruction;
use std::{fs::read, path::PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the binary CHIP-8 program to run
    program: PathBuf,
    /// Display the disassembled binary CHIP-8 program to standard output before running it.  
    /// This is useful for debugging.
    #[arg(long)]
    disasm: bool,
    /// Target frames per second
    #[arg(short, long, default_value_t = 60, value_parser = value_parser!(u16).range(1..))]
    fps: u16,
    /// Target instructions per frame
    #[arg(short, long, default_value_t = 10, value_parser = value_parser!(u16).range(1..))]
    ipf: u16,
    /// Window scale factor
    #[arg(short, long, default_value_t = 10, value_parser = value_parser!(u32).range(1..))]
    scale: u32,
    /// Foreground color in RGBA8888 format (e.g., #FF0A2B1D or 0xFF0A2B1D)
    #[arg(short, long, default_value_t = String::from("0xFFFFFFFF"), value_parser=verify_color)]
    color: String,
    /// Background color in RGBA8888 format (e.g., #FF0A2B1D or 0xFF0A2B1D)
    #[arg(short, long, default_value_t = String::from("0x000000"), value_parser=verify_color)]
    background: String,
    /// Pitch of the buzzer (in Hz)
    #[arg(short, long, default_value_t = 440, value_parser = value_parser!(u16).range(20..=10_000))]
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
                "\'{}\': file could not be opened: {}",
                cli.program.display(),
                err
            );
            return;
        }
    };

    if rom.is_empty() {
        eprintln!(
            "\'{}\': not a valid CHIP-8 program: file is empty",
            cli.program.display()
        );
        return;
    }

    if cli.disasm {
        disassemble(&rom);
    }

    // Clap has already checked that `parse_color` will not return `Err` for these values,
    // so it is safe to unwrap; there is no possibility of panicking.
    let fg = parse_color(&cli.color).unwrap();
    let bg = parse_color(&cli.background).unwrap();

    let options = Options {
        fps: cli.fps,
        ipf: cli.ipf,
        scale: cli.scale,
        fg,
        bg,
        pitch: cli.pitch,
        display_wait: cli.display_wait,
    };
    let mut emu = match Emulator::new(&rom, options) {
        Ok(emu) => emu,
        Err(e) => {
            eprintln!(
                "\'{}\': not a valid CHIP-8 program: {}",
                cli.program.display(),
                e
            );
            return;
        }
    };
    if let Err(e) = emu.run() {
        eprintln!("an unexpected error occurred: {}", e);
    }
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

/// Verifies if the function `parse_color` will succeed
fn verify_color(s: &str) -> Result<String, String> {
    parse_color(s)?;
    Ok(String::from(s))
}

/// Parses input as RGBA8888 (hex) format.  Both "#" and "0x" are allowed as optional
/// prefixes. If parsing as a base 16 value fails, also tries base 10.  Returns `Err`
/// if both parsing attempts fail.
fn parse_color(s: &str) -> Result<u32, String> {
    let stripped = s.strip_prefix('#').unwrap_or(s);
    let stripped = s.strip_prefix("0x").unwrap_or(stripped);

    match u32::from_str_radix(stripped, 16) {
        Ok(value) => Ok(value),
        Err(_) => stripped
            .parse()
            .map_err(|_| format!("{} is not a valid color in RGBA8888 format", s)),
    }
}
