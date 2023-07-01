# ocho
A simple CHIP-8 emulator and disassembler written in Rust. Uses [SDL](https://www.libsdl.org/) for graphics, audio, and keyboard support. If you want to compile it from source, you can run `cargo build --release` or `cargo run --release`. See the [command line interface](#command-line-interface) section below for details. 

This is a hobby project made with the intention of learning more about Rust and emulation. 

## Command line interface
Modern CHIP-8 interpreters often behave slightly different than the original COSMAC VIP version.  This emulator attempts to use modern default behaviors whenever possible. However, you can control this behavior using the command line interface. 
```
Usage: ocho [OPTIONS] <PROGRAM>

Arguments:
  <PROGRAM>  Path to the binary CHIP-8 program

Options:
      --disasm                   Display disassembly code before running the binary CHIP-8 program
  -f, --fps <FPS>                Target frames per second [default: 60]
  -i, --ipf <IPF>                Target instructions per frame [default: 10]
  -s, --scale <SCALE>            Window scale factor [default: 10]
  -c, --color <COLOR>            Foreground color in RGBA8888 format (e.g., #FF0A2B1D or 0xFF0A2B1D) [default: 0xFFFFFFFF]
  -b, --background <BACKGROUND>  Background color in RGBA8888 format (e.g., #FF0A2B1D or 0xFF0A2B1D) [default: 0x000000]
  -p, --pitch <PITCH>            Pitch of the buzzer (in Hz) [default: 440]
  -d, --display-wait             Limit one draw operation per frame
      --quirk-vf-reset           Bitwise operations reset the flags register
      --quirk-memory             Save and load instructions increment the index register
      --quirk-wrap               Sprites drawn to the screen wrap, instead of clip
      --quirk-shift              Bitwise shifting operations use two registers, instead of only one
      --quirk-jump               Jump with offset instruction uses specified register, instead of V0
  -h, --help                     Print help
  -V, --version                  Print version
```

## Keypad
At any time you can press *Esc* to close the emulator. The CHIP-8 featured a hexadecimal keypad with keys labelled `0` through `F`. These keys are mapped using the left-hand side of the keyboard:
```
Keyboard   CHIP-8 Keypad
1 2 3 4      1 2 3 C
Q W E R      4 5 6 D
A S D F      7 8 9 E
Z X C V      A 0 B F
```

## Status
Passes all of [Timendus' tests](https://github.com/Timendus/chip8-test-suite). In order to pass the [quirk's test](https://github.com/Timendus/chip8-test-suite#quirks-test), you must enable the quirks from the [command line interface](#command-line-interface):
```
cargo run --release -- 5-quirks.ch8 --quirk-vf-reset --quirk-memory -d --quirk-shift
```
Sound, timers, and random number generation can be tested using [Matthew Mikolay's tests](https://github.com/mattmikolay/chip-8). In particular, see the [heart monitor demo](https://github.com/mattmikolay/chip-8/tree/master/heartmonitor), [morse code demo](https://github.com/mattmikolay/chip-8/tree/master/morsecode), [delay timer test](https://github.com/mattmikolay/chip-8/tree/master/delaytimer), and [random number test](https://github.com/mattmikolay/chip-8/tree/master/randomnumber). 

## References
 - [https://en.wikipedia.org/wiki/CHIP-8](https://en.wikipedia.org/wiki/CHIP-8)
 - [https://tobiasvl.github.io/blog/write-a-chip-8-emulator/](https://tobiasvl.github.io/blog/write-a-chip-8-emulator/)
 - [https://github.com/Timendus/chip8-test-suite#quirks-test](https://github.com/Timendus/chip8-test-suite#quirks-test)
 - [https://chip8.gulrak.net/](https://chip8.gulrak.net/)
 - [http://devernay.free.fr/hacks/chip8/C8TECH10.HTM](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
 - [https://chip-8.github.io/database/](https://chip-8.github.io/database/)
