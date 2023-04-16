use std::ffi::{c_void, CString};
use std::mem::size_of;

use gl::types::{GLfloat, GLint, GLsizeiptr};
use image::GenericImageView;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::{GLContext, GLProfile, Window};
use sdl2::VideoSubsystem;

struct Monitor(i32);

const VERTEX_SHADER_SRC: &str = include_str!("textures.vert");
const FRAGMENT_SHADER_SRC: &str = include_str!("textures.frag");

fn init_video(sdl: &sdl2::Sdl) -> sdl2::VideoSubsystem {
    let sdl_video = sdl.video().unwrap();

    // hint that we'll use the "330 core" OpenGL profile
    let gl_attr = sdl_video.gl_attr();
    gl_attr.set_context_version(3, 3);
    gl_attr.set_context_profile(GLProfile::Core);

    let debug_gl = true;
    if debug_gl {
        gl_attr.set_context_flags().debug().set();
    }

    sdl_video
}

fn init_window(
    sdl_video: &VideoSubsystem,
    title: &str,
    width: u32,
    height: u32,
    monitor: Monitor,
) -> Window {
    let mut window = sdl_video
        .window(title, width, height)
        .allow_highdpi()
        .opengl()
        .position_centered()
        .resizable()
        .hidden()
        .build()
        .unwrap();

    if let Ok(bounds) = sdl_video.display_bounds(monitor.0) {
        window.set_position(
            sdl2::video::WindowPos::Positioned(bounds.x + (bounds.w - width as i32) / 2),
            sdl2::video::WindowPos::Positioned(bounds.y + (bounds.h - height as i32) / 2),
        );
    }
    window.show();

    return window;
}

fn init_opengl(window: &Window) -> GLContext {
    let gl_context = window.gl_create_context().unwrap();
    window.gl_make_current(&gl_context).unwrap();
    window.subsystem().gl_set_swap_interval(1).unwrap();
    return gl_context;
}

fn gl_error_to_string(err: gl::types::GLenum) -> &'static str {
    match err {
        gl::INVALID_ENUM => "GL_INVALID_ENUM",
        gl::INVALID_VALUE => "GL_INVALID_VALUE",
        gl::INVALID_OPERATION => "GL_INVALID_OPERATION",
        gl::STACK_OVERFLOW => "GL_STACK_OVERFLOW",
        gl::OUT_OF_MEMORY => "GL_OUT_OF_MEMORY",
        gl::INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
        gl::CONTEXT_LOST => "GL_CONTEXT_LOST",
        _ => "(unknown error)",
    }
}

macro_rules! assert_no_gl_error {
    () => {
        let gl_error = gl::GetError();
        if gl_error != gl::NO_ERROR {
            panic!(
                "OpenGL produced error code {}",
                gl_error_to_string(gl_error)
            );
        }
    };
}

fn main() {
    let window_width = 800;
    let window_height = 600;

    /* Create SDL window */
    let sdl = sdl2::init().unwrap();
    let sdl_video = init_video(&sdl);
    let window = init_window(
        &sdl_video,
        "Base Project",
        window_width,
        window_height,
        Monitor(1),
    );

    /* Initialize OpenGL */
    let _gl_context = init_opengl(&window); // closes on drop
    gl::load_with(|s| sdl_video.gl_get_proc_address(s) as _);

    /* Setup rendering */
    let mut vbo = 0;
    let mut vao = 0;
    let vertex_shader;
    let frag_shader;
    let shader_program;
    let mut texture_id = 0;
    unsafe {
        // Vertex shader
        vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        let vertex_c_str = CString::new(VERTEX_SHADER_SRC.as_bytes()).unwrap();
        gl::ShaderSource(vertex_shader, 1, &vertex_c_str.as_ptr(), std::ptr::null());
        gl::CompileShader(vertex_shader);

        let mut vertex_status = gl::FALSE as GLint;
        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut vertex_status);
        if vertex_status != (gl::TRUE as GLint) {
            panic!("couldn't compile vertex shader");
        }

        // Fragment shader
        frag_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        let frag_c_str = CString::new(FRAGMENT_SHADER_SRC.as_bytes()).unwrap();
        gl::ShaderSource(frag_shader, 1, &frag_c_str.as_ptr(), std::ptr::null());
        gl::CompileShader(frag_shader);

        let mut frag_status = gl::FALSE as GLint;
        gl::GetShaderiv(frag_shader, gl::COMPILE_STATUS, &mut frag_status);
        if frag_status != (gl::TRUE as GLint) {
            panic!("couldn't compile fragment shader");
        }

        // Shader program
        shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, frag_shader);
        gl::LinkProgram(shader_program);

        let mut program_status = gl::FALSE as GLint;
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut program_status);
        if program_status != (gl::TRUE as GLint) {
            panic!("couldn't link shader program");
        }

        // VAO
        gl::GenVertexArrays(1, &mut vao);

        // VBO
        #[rustfmt::skip]
        let vertices: [GLfloat; (3 + 4 + 2) * 6] = [
            // pos              // color               // texture uv

            // first triangle
            -0.5, -0.5, 0.0,    1.0, 1.0, 1.0, 1.0,    0.0, 0.0,  // bottom left
            -0.5, 0.5, 0.0,     1.0, 1.0, 1.0, 1.0,    0.0, 1.0,  // top left
            0.5, 0.5, 0.0,      1.0, 1.0, 1.0, 1.0,    1.0, 1.0,  // top right

            // second triangle
            -0.5, -0.5, 0.0,    1.0, 1.0, 1.0, 1.0,    0.0, 0.0,  // bottom left
            0.5, 0.5, 0.0,      1.0, 1.0, 1.0, 1.0,    1.0, 1.0,  // top right
            0.5, -0.5, 0.0,     1.0, 1.0, 1.0, 1.0,    1.0, 0.0,  // bottom right
        ];
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
            vertices.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );
        assert_no_gl_error!();

        // Vertex attribute pointers
        gl::BindVertexArray(vao);

        let position_location = 0;
        let position_size = 3;
        let position_stride = (3 + 4 + 2) * size_of::<GLfloat>();
        let position_offset = 0 * size_of::<GLfloat>();
        gl::VertexAttribPointer(
            position_location,
            position_size as i32,
            gl::FLOAT,
            gl::FALSE,
            position_stride as i32,
            position_offset as *const c_void,
        );
        gl::EnableVertexAttribArray(0);
        assert_no_gl_error!();

        let color_location = 1;
        let color_size = 4;
        let color_stride = (3 + 4 + 2) * size_of::<GLfloat>();
        let color_offset = 3 * size_of::<GLfloat>();
        gl::VertexAttribPointer(
            color_location,
            color_size as i32,
            gl::FLOAT,
            gl::FALSE,
            color_stride as i32,
            color_offset as *const c_void,
        );
        gl::EnableVertexAttribArray(1);
        assert_no_gl_error!();

        let texture_uv_location = 2;
        let texture_uv_size = 2;
        let texture_uv_stride = (3 + 4 + 2) * size_of::<GLfloat>();
        let texture_uv_offset = (3 + 4) * size_of::<GLfloat>();

        gl::VertexAttribPointer(
            texture_uv_location,
            texture_uv_size as i32,
            gl::FLOAT,
            gl::FALSE,
            texture_uv_stride as i32,
            texture_uv_offset as *const c_void,
        );
        gl::EnableVertexAttribArray(2);
        assert_no_gl_error!();

        // Texture data
        let texture_image = image::open("resources/container.jpg").unwrap();
        let (texture_width, texture_height) = texture_image.dimensions();
        let texture_data = texture_image
            .pixels()
            .map(|(_x, _y, pixel)| pixel.0)
            .collect::<Vec<[u8; 4]>>();
        gl::GenTextures(1, &mut texture_id);
        gl::BindTexture(gl::TEXTURE_2D, texture_id);
        #[rustfmt::skip]
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            texture_width as i32,
            texture_height as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            texture_data.as_ptr() as *const c_void,
        );
        assert_no_gl_error!();
    }

    /* Main loop */
    let mut event_pump = sdl.event_pump().unwrap();
    'main_loop: loop {
        /* Input */
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    break 'main_loop;
                }
                Event::KeyDown { keycode, .. } if keycode == Some(Keycode::Escape) => {
                    break 'main_loop;
                }
                _ => (),
            }
        }

        /* Update */
        // ...

        /* Draw */
        unsafe {
            gl::ClearColor(0.0, 0.5, 0.5, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(shader_program);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }

        window.gl_swap_window();
    }

    /* Clean up rendering */
    unsafe {
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(frag_shader);
    }
}
