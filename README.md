# CHIP-8 in Rust

This project started as a way for me to learn:

- Low-level stuff;
- Rust;
- Hardware/Console emulation.

(I'm new to all of the above)

I've started the main implementation but decide to start over and document my progress here.

## Dependencies

To be able to build and run the project you will first need to have Rust installed. It's also necessary to have SDL2 lib installed on your system. For ubuntu based systems run the following command:

```bash
sudo apt-get install libsdl2-dev
```

## Running

To test out the project, use the following command:

```bash
cargo run -- <path> <ipf>
```

- path: Path to ROM file.
- ipf: Instructions per frame.

## Current State

The following checklist shows a bit of the progress and current state of the emulator.

- [x] Memory, stack, registers and PC;
- [x] ROM reading;
- [x] Fonts;
- [x] Cycle accurate loop;
- [x] OpCode matching;
- [x] Display to screen;
- [x] Basic OpCodes (for IBM logo e.g.);
- [ ] All OpCodes;
- [ ] Quirk configurability.
