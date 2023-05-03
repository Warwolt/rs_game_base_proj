# TODO list

- [ ] rendering
  - [x] draw lines
  - [x] draw rectangles
  - [x] draw circles
  - [x] draw with alpha value
  - [x] draw sprites
    - [x] Aseprite JSON sprite sheet integration
    - [x] animation
  - [ ] draw fonts
    - [ ] load fonts in background thread
  - [ ] render at different resolutions (reference: https://stackoverflow.com/questions/7071090/low-resolution-in-opengl-to-mimic-older-games)
  - [ ] tilemaps (for levels)

- [ ] input
  - [x] update loop with delta time
  - [x] keyboard
  - [x] mouse
  - [ ] gamepad

- [ ] audio
  - [ ] play sounds
  - [ ] play music

- [ ] physics
  - [ ] point line intersection
  - [ ] line line intersection
  - [ ] line circle intersection
  - [ ] line polygon intersection
  - [ ] circle circle intersection
  - [ ] polygon polygon intersection

- [ ] dev tools
  - [ ] add profiler (https://crates.io/crates/microprofile)
  - [ ] soure code hot reloading (https://robert.kra.hn/posts/hot-reloading-rust/)
  - [ ] resource hot reloading
    - [x] sprites
    - [ ] sounds
  - [ ] deserialize ini file into a struct
  - [ ] measure frame count
