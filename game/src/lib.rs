mod ui;

use std::{collections::HashMap, path::PathBuf};

use engine::{
    audio::MusicID,
    geometry::Rect,
    graphics::{
        animation::{self, AnimationID},
        rendering::Renderer,
        sprites::{self, SpriteSheetID},
    },
    imgui::dock,
    imgui::ImGui,
    input::config::ProgramConfig,
    Engine,
};
use sdl2::keyboard::Keycode;
use ui::GameUi;

pub struct GameState {
    ui: GameUi,
    show_editor_ui: bool,
    show_debug_ui: bool,
    editor_layout_init: bool,
    editor_zoom_amount: u8,
    music_playing: bool,
    music_id: MusicID,
    smiley_sprite_sheet_id: SpriteSheetID,
    smiley_animations: HashMap<Direction, AnimationID>,
    smiley_direction: Direction,
    smiley_is_animating: bool,
    smiley_input_mappings: Vec<(Keycode, Direction)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    Right,
    Up,
    Left,
    Down,
}

/// Keep DLL global variables up to date with main globals
#[no_mangle]
pub fn set_global_contexts(
    logger: &'static dyn log::Log,
    level: log::LevelFilter,
    ctx: *mut imgui::sys::ImGuiContext,
) {
    log::set_max_level(level);
    log::set_logger(logger).unwrap();
    unsafe {
        imgui::sys::igSetCurrentContext(ctx);
    }
}

#[no_mangle]
pub fn init(
    logger: &'static dyn log::Log,
    level: log::LevelFilter,
    ctx: *mut imgui::sys::ImGuiContext,
    engine: &mut Engine,
    config: &ProgramConfig,
) -> GameState {
    set_global_contexts(logger, level, ctx);

    // Init smiley
    let smiley_json_path = &PathBuf::from(r"resources/smiley.json");
    let smiley_texture_id = *engine.textures.get("resources/smiley.png").unwrap();
    let smiley_sprite_sheet_data = sprites::load_aseprite_sprite_sheet(&smiley_json_path).unwrap();
    let smiley_sprite_sheet_frames =
        sprites::aseprite_sprite_sheet_frames(&smiley_sprite_sheet_data);
    let smiley_sprite_sheet_id =
        engine
            .sprites
            .add_spritesheet(smiley_texture_id, &smiley_sprite_sheet_frames, None);
    let mut smiley_animations = HashMap::<Direction, AnimationID>::new();
    let smiley_input_mappings = vec![
        (Keycode::Right, Direction::Right),
        (Keycode::Up, Direction::Up),
        (Keycode::Left, Direction::Left),
        (Keycode::Down, Direction::Down),
    ];
    let smiley_animation_mappings = [
        (Direction::Right, "Right"),
        (Direction::Up, "Up"),
        (Direction::Left, "Left"),
        (Direction::Down, "Down"),
    ];
    for (direction, frame_tag_name) in smiley_animation_mappings {
        smiley_animations.entry(direction).or_insert(
            animation::add_asperite_sprite_sheet_animation(
                &mut engine.animation,
                &smiley_sprite_sheet_data,
                frame_tag_name,
            ),
        );
    }
    let smiley_direction = Direction::Down;
    let smiley_is_animating = false;

    if smiley_is_animating {
        engine
            .animation
            .start_animation(smiley_animations[&smiley_direction]);
    }

    let music_id = engine
        .audio
        .add_music(&PathBuf::from("./resources/audio/music.wav"));

    GameState {
        ui: GameUi::new(engine),
        show_debug_ui: config.show_debug_ui,
        show_editor_ui: true,
        editor_layout_init: false,
        editor_zoom_amount: 2,
        music_playing: false,
        music_id,
        smiley_sprite_sheet_id,
        smiley_animations,
        smiley_direction,
        smiley_is_animating,
        smiley_input_mappings,
    }
}

#[no_mangle]
pub fn update(game: &mut GameState, engine: &mut Engine, imgui: &mut ImGui) {
    if engine.input.keyboard.is_pressed_now(Keycode::F3) {
        game.show_debug_ui = !game.show_debug_ui;
    }
    if engine.input.keyboard.is_pressed_now(Keycode::F2) {
        game.show_editor_ui = !game.show_editor_ui;
    }

    if engine.input.keyboard.is_pressed(Keycode::Escape) {
        engine.request_quit();
    }

    game.ui.draw_centered();
    game.ui.set_cursor(
        (engine.renderer.canvas().size.width / 2) as i32,
        (engine.renderer.canvas().size.height / 2) as i32,
    );

    for (keycode, direction) in &game.smiley_input_mappings {
        if engine.input.keyboard.is_pressed_now(*keycode) {
            game.smiley_direction = *direction;
            if game.smiley_is_animating {
                let animation_id = game.smiley_animations[direction];
                engine.animation.start_animation(animation_id);
            }
        }
    }

    let play_label = if engine.audio.music_is_paused() || !game.music_playing {
        "Play"
    } else {
        "Pause"
    };
    if game.ui.button(play_label) {
        game.smiley_is_animating = !game.smiley_is_animating;

        let animation_id = game.smiley_animations[&game.smiley_direction];
        if game.smiley_is_animating {
            engine.animation.start_animation(animation_id);
        } else {
            engine.animation.stop_animation(animation_id);
        }

        if engine.audio.music_is_paused() {
            engine.audio.resume_music();
        } else {
            engine.audio.pause_music();
        }

        if !game.music_playing {
            game.music_playing = true;
            engine.audio.play_music(game.music_id);
        }
    }

    if let Some(ui) = begin_imgui_frame(imgui, engine) {
        if game.show_editor_ui {
            draw_editor_ui(game, engine, ui);
        }
        if game.show_debug_ui {
            debug::draw_debug_ui(ui);
        }
    }

    game.ui.update(engine);
}

#[no_mangle]
pub fn render(game: &mut GameState, engine: &mut Engine) {
    engine.renderer.clear();
    draw_background(&mut engine.renderer);
    game.ui.render(engine);

    // draw smiley
    {
        let smiley_frame = engine
            .animation
            .current_frame(game.smiley_animations[&game.smiley_direction]);
        let canvas = engine.renderer.canvas().size;
        let (smiley_width, smiley_height) = (16, 16);
        let (smiley_x, smiley_y) = (
            (canvas.width - smiley_width) / 2,
            (canvas.height - smiley_height) / 2 - (canvas.height as f32 * 0.1) as u32,
        );
        engine.sprites.draw_sprite(
            &mut engine.renderer,
            game.smiley_sprite_sheet_id,
            smiley_frame,
            smiley_x as _,
            smiley_y as _,
        );
    }
}

#[no_mangle]
pub fn write_to_config(config: &mut ProgramConfig, game: &GameState) {
    config.show_debug_ui = game.show_debug_ui;
}

fn draw_background(renderer: &mut Renderer) {
    renderer.set_draw_color(0, 128, 128, 255);
    renderer.draw_rect_fill(Rect {
        x: 0,
        y: 0,
        w: renderer.canvas().size.width,
        h: renderer.canvas().size.height,
    });
}

fn begin_imgui_frame<'a>(imgui: &'a mut ImGui, engine: &Engine<'a>) -> Option<&'a mut imgui::Ui> {
    // The ImGui context can be momentarily lost when reloading the game DLL
    let context_exists = unsafe { imgui::sys::igGetCurrentContext() != std::ptr::null_mut() };
    if context_exists {
        Some(imgui.begin_frame(engine))
    } else {
        None
    }
}

fn draw_editor_ui(game: &mut GameState, engine: &mut Engine, ui: &imgui::Ui) {
    let _dockspace = dock::dockspace("DockSpace", ui, &mut game.editor_layout_init)
        .split_node("Right", imgui::Direction::Right, 0.25)
        .split_node("Bottom", imgui::Direction::Down, 0.25)
        .dock_window("SceneView", "DockSpace")
        .dock_window("Log", "Bottom")
        .dock_window("SceneEditor", "Right")
        .begin();

    let menu_bar_padding = ui.push_style_var(imgui::StyleVar::FramePadding([0.0, 8.0]));
    if let Some(_menu_bar) = ui.begin_main_menu_bar() {
        menu_bar_padding.end();
        if let Some(_file_menu) = ui.begin_menu("File") {
            if ui.menu_item("Exit") {
                engine.request_quit();
            }
        }
    }

    if let Some(_window) = ui.window("Log").begin() {
        ui.text(format!(
            "window relative mouse {:?}",
            engine.input.mouse.window_pos
        ));
        ui.text(format!(
            "canvas relative mouse {:?}",
            engine.input.mouse.pos
        ));
    }

    if let Some(_window) = ui.window("SceneEditor").begin() {
        ui.text("Zoom:");
        ui.radio_button("1x", &mut game.editor_zoom_amount, 1);
        ui.radio_button("2x", &mut game.editor_zoom_amount, 2);
        ui.radio_button("3x", &mut game.editor_zoom_amount, 3);
        ui.radio_button("4x", &mut game.editor_zoom_amount, 4);
    }

    // FIXME-1: Need to figure out how to keep the canvas position up to date
    // relative to a scrolled position internally in the window.
    //
    // FIXME-2: Probably we should only be setting up the layout if the ini file
    // doesn't exist? or something. It would be nice to be able to keep the
    // layout used when closing the program.
    if let Some(_window) = ui.window("SceneView").horizontal_scrollbar(true).begin() {
        let _canvas_window = ui.child_window("Canvas").begin();
        let window_size = ui.window_size();
        let window_pos = ui.window_pos();
        let canvas_texture = engine.renderer.canvas().texture as usize;

        // FIXME: the canvas should be kept up to date in another function
        // probably? Doing it raw like this seems super error prone, need a
        // better abstraction for reifying the idea of a "canvas".

        // setup canvas size here
        let scale = game.editor_zoom_amount as f32;
        let image_size = [
            scale * engine.renderer.canvas().size.width as f32,
            scale * engine.renderer.canvas().size.height as f32,
        ];
        let image_pos = [
            f32::round(window_pos[0] + (window_size[0] - image_size[0]) * 0.5),
            f32::round(window_pos[1] + (window_size[1] - image_size[1]) * 0.5),
        ];

        // need to keep canvas size and pos up to date for mouse to work
        let canvas = engine.renderer.canvas_mut();
        canvas.scale = scale;
        canvas.pos.x = image_pos[0] as i32;
        canvas.pos.y = image_pos[1] as i32;
        canvas.scaled_size.width = image_size[0] as u32;
        canvas.scaled_size.height = image_size[1] as u32;

        // FIXME: this somehow completely messes up scrolling, but centering the
        // image is required in order to make the canvas view look nice
        ui.set_cursor_screen_pos(image_pos);
        let _ = imgui::Image::new(imgui::TextureId::new(canvas_texture), image_size)
            .uv0([0.0, 1.0])
            .uv1([1.0, 0.0])
            .build(&ui);
    }
}

mod debug {
    pub fn draw_debug_ui(ui: &imgui::Ui) {
        if let Some(window) = ui.window("Debug Window").begin() {
            if ui.button("Press me!") {
                log::debug!("hello world!");
            }
            window.end();
        }
    }
}
