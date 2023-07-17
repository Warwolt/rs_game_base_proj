use std::ffi::{c_void, CString};
use std::mem::size_of;

use gl::types::{GLfloat, GLint, GLsizeiptr};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::{GLContext, GLProfile, Window};
use sdl2::VideoSubsystem;

struct Monitor(i32);

struct ShaderProgram {
    vertex_shader: u32,
    frag_shader: u32,
    program: u32,
}

const VERTEX_SHADER_SRC: &str = include_str!("shader.vert");
const FRAGMENT_SHADER_SRC: &str = include_str!("shader.frag");
const CANVAS_VERTEX_SRC: &str = include_str!("canvas.vert");
const CANVAS_FRAGMENT_SRC: &str = include_str!("canvas.frag");

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

impl ShaderProgram {
    fn new(vert_path: &str, frag_path: &str) -> Self {
        let vertex_shader;
        let frag_shader;
        let shader_program;
        unsafe {
            // Vertex shader
            vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
            let vertex_c_str = CString::new(vert_path.as_bytes()).unwrap();
            gl::ShaderSource(vertex_shader, 1, &vertex_c_str.as_ptr(), std::ptr::null());
            gl::CompileShader(vertex_shader);

            let mut vertex_status = gl::FALSE as GLint;
            gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut vertex_status);
            if vertex_status != (gl::TRUE as GLint) {
                panic!("couldn't compile vertex shader");
            }

            // Fragment shader
            frag_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            let frag_c_str = CString::new(frag_path.as_bytes()).unwrap();
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
        }

        ShaderProgram {
            vertex_shader,
            frag_shader,
            program: shader_program,
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.vertex_shader);
            gl::DeleteShader(self.frag_shader);
        }
    }
}

fn set_attribute_pointers(vao: u32, vbo: u32) {
    unsafe {
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

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
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let resolution_width = 200;
    let resolution_height = 150;

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
    let shader = ShaderProgram::new(VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC);
    let mut triangle_vbo = 0;
    let mut triangle_vao = 0;
    let mut white_texture = 0;
    unsafe {
        // VAO
        gl::GenVertexArrays(1, &mut triangle_vao);

        // VBO
        #[rustfmt::skip]
        let triangle_vertices: [GLfloat; (3 + 4 + 2) * 3] = [
            // pos              // color               // texture
            -0.5, -0.5, 0.0,    1.0, 0.0, 0.0, 1.0,    0.0, 0.0,
            0.0,  0.5, 0.0,     0.0, 1.0, 0.0, 1.0,    0.0, 0.0,
            0.5, -0.5, 0.0,     0.0, 0.0, 1.0, 1.0,    0.0, 0.0,
        ];
        gl::GenBuffers(1, &mut triangle_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, triangle_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (triangle_vertices.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
            triangle_vertices.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );
        assert_no_gl_error!();

        set_attribute_pointers(triangle_vao, triangle_vbo);

        // create a blank white texture
        gl::GenTextures(1, &mut white_texture);
        gl::BindTexture(gl::TEXTURE_2D, white_texture);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

        let white_rgba: [u8; 4] = [255, 255, 255, 255];
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            1, // 1 pixel wide
            1, // 1 pixel high
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            white_rgba.as_ptr() as *const c_void,
        );
        assert_no_gl_error!();
    }

    let canvas_shader = ShaderProgram::new(CANVAS_VERTEX_SRC, CANVAS_FRAGMENT_SRC);
    let mut canvas_fbo = 0;
    let mut canvas_vao = 0;
    let mut canvas_texture = 0;
    unsafe {
        // VAO
        gl::GenVertexArrays(1, &mut canvas_vao);

        // VBO
        #[rustfmt::skip]
        let canvas_rect: [GLfloat; (3 + 4 + 2) * 6] = [
            // pos              // color                  // texture
            // first triangle
            -1.0, -1.0, 0.0,    1.0, 1.0, 1.0, 1.0,    0.0, 0.0,  // bottom left
            -1.0, 1.0, 0.0,     1.0, 1.0, 1.0, 1.0,    0.0, 1.0,  // top left
            1.0, 1.0, 0.0,      1.0, 1.0, 1.0, 1.0,    1.0, 1.0,  // top right
            // second triangle
            -1.0, -1.0, 0.0,    1.0, 1.0, 1.0, 1.0,    0.0, 0.0,  // bottom left
            1.0, 1.0, 0.0,      1.0, 1.0, 1.0, 1.0,    1.0, 1.0,  // top right
            1.0, -1.0, 0.0,     1.0, 1.0, 1.0, 1.0,    1.0, 0.0,  // bottom right
        ];
        let mut canvas_vbo = 0;
        gl::GenBuffers(1, &mut canvas_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, canvas_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (canvas_rect.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
            canvas_rect.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );
        assert_no_gl_error!();

        set_attribute_pointers(canvas_vao, canvas_vbo);

        gl::GenFramebuffers(1, &mut canvas_fbo);
        gl::BindFramebuffer(gl::FRAMEBUFFER, canvas_fbo);

        gl::GenTextures(1, &mut canvas_texture);
        gl::BindTexture(gl::TEXTURE_2D, canvas_texture);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            resolution_width as i32,
            resolution_height as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            std::ptr::null(),
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, canvas_texture, 0);
        assert_no_gl_error!();

        let fbo_status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
        if fbo_status != gl::FRAMEBUFFER_COMPLETE {
            panic!(
                "CheckFramebufferStatus failed with fbo_status = {}",
                fbo_status
            );
        }

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
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
            // draw on canvas
            gl::BindFramebuffer(gl::FRAMEBUFFER, canvas_fbo);
            gl::Viewport(0, 0, resolution_width as i32, resolution_height as i32);
            gl::ClearColor(0.0, 0.5, 0.5, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(shader.program);
            gl::BindTexture(gl::TEXTURE_2D, white_texture);
            gl::BindVertexArray(triangle_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);

            // render canvas to screen
            let (window_width, window_height) = window.size();
            let (canvas_width, canvas_height) = {
                let scale = f32::min(
                    window_width as f32 / resolution_width as f32,
                    window_height as f32 / resolution_height as f32,
                );
                (
                    f32::round(scale * resolution_width as f32) as i32,
                    f32::round(scale * resolution_height as f32) as i32,
                )
            };
            let (canvas_x, canvas_y) = (
                (window_width as i32 - canvas_width) / 2,
                (window_height as i32 - canvas_height) / 2,
            );

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Viewport(canvas_x, canvas_y, canvas_width, canvas_height);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::UseProgram(canvas_shader.program);
            gl::BindVertexArray(canvas_vao);
            gl::BindTexture(gl::TEXTURE_2D, canvas_texture);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }

        window.gl_swap_window();
    }
}
