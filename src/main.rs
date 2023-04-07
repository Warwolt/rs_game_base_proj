use std::path::Path;

use configparser::ini::Ini;
use glow::HasContext;
use sdl2::keyboard::Keycode;
use sdl2::VideoSubsystem;
use sdl2::{event::Event, video::Window};
use simple_logger::SimpleLogger;

const CONFIG_FILE: &str = "config.ini";

fn init_logging() {
    SimpleLogger::new().init().unwrap();
}

fn init_window(sdl_video: &VideoSubsystem, title: &str, width: u32, height: u32) -> Window {
    let mut window = sdl_video
        .window(title, width, height)
        .allow_highdpi()
        .opengl()
        .position_centered()
        .resizable()
        .hidden()
        .build()
        .unwrap();

    // hack: move window to 2nd monitor because my laptop is stupid
    {
        let bounds = sdl_video.display_bounds(1).unwrap();
        window.set_position(
            sdl2::video::WindowPos::Positioned(bounds.x + (bounds.w - width as i32) / 2),
            sdl2::video::WindowPos::Positioned(bounds.y + (bounds.h - height as i32) / 2),
        );
        window.show();
    }

    return window;
}

fn main() {
    /* Initialize logging */
    init_logging();

    /* Initialize configuration */
    let mut config = Ini::new();
    if Path::new("./config.ini").exists() {
        config.load("config.ini").unwrap();
    }
    config.set("numbers", "first", Some(String::from("11")));

    /* Initialize SDL */
    let sdl = sdl2::init().unwrap();
    let sdl_video = sdl.video().unwrap();
    let window = init_window(&sdl_video, "Base Project", 800, 600);

    /* Initialize OpenGL */
    let gl_attr = sdl_video.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 0);
    let _gl_context = window.gl_create_context().unwrap();
    let gl = unsafe {
        glow::Context::from_loader_function(|s| sdl_video.gl_get_proc_address(s) as *const _)
    };

    let (shader_program, vertex_array) = unsafe {
        let vertex_array = gl
            .create_vertex_array()
            .expect("Cannot create vertex array");
        gl.bind_vertex_array(Some(vertex_array));

        let shader_program = gl.create_program().expect("Cannot create shader program");

        let vertex_shader_source = r#"const vec2 verts[3] = vec2[3](
                vec2(0.5f, 1.0f),
                vec2(0.0f, 0.0f),
                vec2(1.0f, 0.0f)
            );
            out vec2 vert;
            void main() {
                vert = verts[gl_VertexID];
                gl_Position = vec4(vert - 0.5, 0.0, 1.0);
            }"#;
        let fragment_shader_source = r#"precision mediump float;
            in vec2 vert;
            out vec4 color;
            void main() {
                color = vec4(vert, 0.5, 1.0);
            }"#;

        let shader_version = "#version 130";

        let shader_sources = [
            (glow::VERTEX_SHADER, vertex_shader_source),
            (glow::FRAGMENT_SHADER, fragment_shader_source),
        ];

        let mut shaders = Vec::with_capacity(shader_sources.len());

        for (shader_type, shader_source) in shader_sources.iter() {
            let shader = gl
                .create_shader(*shader_type)
                .expect("Cannot create shader");
            gl.shader_source(shader, &format!("{}\n{}", shader_version, shader_source));
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                panic!("{}", gl.get_shader_info_log(shader));
            }
            gl.attach_shader(shader_program, shader);
            shaders.push(shader);
        }

        gl.link_program(shader_program);
        if !gl.get_program_link_status(shader_program) {
            panic!("{}", gl.get_program_info_log(shader_program));
        }

        for shader in shaders {
            gl.detach_shader(shader_program, shader);
            gl.delete_shader(shader);
        }

        gl.use_program(Some(shader_program));

        (shader_program, vertex_array)
    };

    unsafe {
        gl.clear_color(0.0, 0.5, 0.5, 1.0);
    }

    let mut event_pump = sdl.event_pump().unwrap();
    'main_loop: loop {
        /* Input */
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main_loop,
                Event::KeyDown { keycode, .. } => match keycode {
                    Some(Keycode::Escape) => {
                        break 'main_loop;
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        /* Game update */
        // ..

        /* ImGui update */

        /* Game render */
        unsafe {
            gl.clear(glow::COLOR_BUFFER_BIT);
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
        }

        /* ImGui render */

        window.gl_swap_window();
    }

    unsafe {
        gl.delete_program(shader_program);
        gl.delete_vertex_array(vertex_array);
    }

    config.write(CONFIG_FILE).unwrap();
}
