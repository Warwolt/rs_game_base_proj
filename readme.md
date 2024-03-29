# Base Project
A rust crate using SDL+OpenGL to provide the basis for game development.
Games made with this should be forked from this repo, probably.

## Setup
To run SDL, you need the following library files:

- SDL2
- SDL2_Mixer

### Windows
For Windows, MSVC is used. Download the relevant libraries from:
- https://github.com/libsdl-org/SDL/releases
- https://github.com/libsdl-org/SDL_mixer/releases

And then add them to:
```
C:\Users\{Your Username}\.rustup\toolchains\{current toolchain}\lib\rustlib\{current toolchain}\lib
```

(For further instructions see https://github.com/Rust-SDL2/rust-sdl2#windows-msvc)

## Tooling
This project assumes Aseprite is used for pixel art

## Hot reloading
All the game code is under the `game` crate, and this code can be reloaded by
pressing F5 while running the program.
