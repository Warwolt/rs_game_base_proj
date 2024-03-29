use crate::GameState;
use engine::{imgui::dock, Engine};
use imgui::StyleColor;

pub struct Editor {
    layout_initialized: bool,
    zoom_amount: u8,
    show_demo: bool,
}

impl Editor {
    pub fn new() -> Self {
        Editor {
            layout_initialized: false,
            show_demo: false,
            zoom_amount: 3,
        }
    }
}

pub fn draw_ui(game: &mut GameState, engine: &mut Engine, ui: &imgui::Ui) {
    let scene_view_label = "SceneView";
    let log_label = "Log";
    let _dockspace = dock::dockspace("DockSpace", ui, &mut game.editor.layout_initialized)
        .split_node("Right", imgui::Direction::Right, 0.25)
        .split_node("Bottom", imgui::Direction::Down, 0.15)
        .dock_window(scene_view_label, "DockSpace")
        .dock_window(log_label, "Bottom")
        .dock_window("SceneEditor", "Right")
        .begin();

    if game.editor.show_demo {
        ui.show_demo_window(&mut game.editor.show_demo);
    }

    if let Some(_menu_bar) = ui.begin_main_menu_bar() {
        if let Some(_file_menu) = ui.begin_menu("File") {
            if ui
                .menu_item_config("Show Demo")
                .selected(game.editor.show_demo)
                .build()
            {
                game.editor.show_demo = !game.editor.show_demo;
            }

            if ui.menu_item("Exit") {
                engine.request_quit();
            }
        }
    }

    if let Some(_window) = ui.window(log_label).begin() {
        for entry in engine.captured_log {
            // Print log
            ui.text(format!("{}", entry.time));
            ui.same_line();
            let log_level_color = match entry.level {
                log::Level::Error => [255.0, 0.0, 0.0, 255.0],   // Red
                log::Level::Warn => [255.0, 242.0, 0.0, 255.0],  // Yellow
                log::Level::Info => [0.0, 255.0, 0.0, 255.0],    // Green
                log::Level::Debug => [255.0, 0.0, 255.0, 255.0], // Purple
                log::Level::Trace => [0.0, 255.0, 255.0, 255.0], // Cyan
            };
            let color_style = ui.push_style_color(StyleColor::Text, log_level_color);
            ui.text(format!("{}", entry.level));
            color_style.end();
            ui.same_line();
            ui.text(format!("[{}] {}", entry.module, entry.text));
        }

        // Auto scroll
        if ui.scroll_y() >= ui.scroll_max_y() {
            ui.set_scroll_here_y_with_ratio(1.0);
        }
    }

    if let Some(_window) = ui.window("SceneEditor").begin() {
        ui.text("Zoom:");
        ui.radio_button("1x", &mut game.editor.zoom_amount, 1);
        ui.radio_button("2x", &mut game.editor.zoom_amount, 2);
        ui.radio_button("3x", &mut game.editor.zoom_amount, 3);
        ui.radio_button("4x", &mut game.editor.zoom_amount, 4);

        ui.text(format!(
            "window relative mouse {:?}",
            engine.input.mouse.window_pos
        ));
        ui.text(format!(
            "canvas relative mouse {:?}",
            engine.input.mouse.pos
        ));
    }

    // FIXME-1: Need to figure out how to keep the canvas position up to date
    // relative to a scrolled position internally in the window.
    //
    // FIXME-2: Probably we should only be setting up the layout if the ini file
    // doesn't exist? or something. It would be nice to be able to keep the
    // layout used when closing the program.
    if let Some(_window) = ui.window(scene_view_label).begin() {
        let _canvas_window = ui.child_window("Canvas").begin();
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

        let avail_size = ui.content_region_avail();
        let cursor_pos = ui.cursor_screen_pos();
        let image_pos = [
            cursor_pos[0] + f32::ceil(f32::max(0.0, (avail_size[0] - image_size[0]) * 0.5)),
            cursor_pos[1] + f32::ceil(f32::max(0.0, (avail_size[1] - image_size[1]) * 0.5)),
        ];

        // need to keep canvas size and pos up to date for mouse to work
        let canvas = engine.renderer.canvas_mut();
        canvas.scale = scale;
        canvas.pos.x = image_pos[0] as i32;
        canvas.pos.y = image_pos[1] as i32;
        canvas.scaled_size.width = image_size[0] as u32;
        canvas.scaled_size.height = image_size[1] as u32;

        ui.set_cursor_screen_pos(image_pos);
        let _ = imgui::Image::new(imgui::TextureId::new(canvas_texture), image_size)
            .uv0([0.0, 1.0])
            .uv1([1.0, 0.0])
            .build(&ui);
    }
}
