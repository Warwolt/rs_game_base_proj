use crate::GameState;
use engine::{imgui::dock, Engine};

pub struct Editor {
    layout_initialized: bool,
    zoom_amount: u8,
}

impl Editor {
    pub fn new() -> Self {
        Editor {
            layout_initialized: false,
            zoom_amount: 3,
        }
    }
}

pub fn draw_ui(game: &mut GameState, engine: &mut Engine, ui: &imgui::Ui) {
    let _dockspace = dock::dockspace("DockSpace", ui, &mut game.editor.layout_initialized)
        .split_node("Right", imgui::Direction::Right, 0.25)
        .split_node("Bottom", imgui::Direction::Down, 0.15)
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
        ui.radio_button("1x", &mut game.editor.zoom_amount, 1);
        ui.radio_button("2x", &mut game.editor.zoom_amount, 2);
        ui.radio_button("3x", &mut game.editor.zoom_amount, 3);
        ui.radio_button("4x", &mut game.editor.zoom_amount, 4);
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
        let scale = game.editor.zoom_amount as f32;
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
