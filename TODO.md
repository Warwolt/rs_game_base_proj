# TODO list

- [x] rendering
  - [x] draw lines
  - [x] draw rectangles
  - [x] draw circles
  - [x] draw with alpha value
  - [x] draw sprites
    - [x] Aseprite JSON sprite sheet integration
    - [x] animation
  - [x] draw fonts
    - [ ] load fonts in background thread
  - [x] render at different resolutions (reference: https://stackoverflow.com/questions/7071090/low-resolution-in-opengl-to-mimic-older-games)
  - [ ] tilemaps (for levels)
  - [x] toggle fullscreen with F11

- [x] input
  - [x] update loop with delta time
  - [x] keyboard
  - [x] mouse
  - [ ] gamepad

- [x] audio
  - [x] play sounds
  - [x] play music
  - [x] hot reload audio files

- [ ] physics
  - [ ] geometry
    - [ ] point line intersection
    - [ ] line line intersection
    - [ ] line circle intersection
    - [ ] line polygon intersection
    - [ ] circle circle intersection
    - [ ] polygon polygon intersection
  - [ ] collision system

- [ ] dev tools
  - [ ] add profiler (https://crates.io/crates/microprofile)
  - [ ] source code hot reloading (https://robert.kra.hn/posts/hot-reloading-rust/)
  - [x] resource hot reloading
    - [x] sprites
    - [x] sounds
  - [ ] deserialize ini file into a struct
  - [ ] measure frame count

- [ ] documentation
  - [ ] setup rust docs for project
  - [ ] document the reload system
  - [ ] document rendering

- [x] migrate resources to Git LFS
