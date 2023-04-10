use gl::types::*;
use glam::Mat4;
use std::{
    ffi::{c_void, CString},
    mem::size_of,
};

use crate::game_state::GameState;

const VERTEX_SHADER_SRC: &str = include_str!("vertex.shader");
const FRAGMENT_SHADER_SRC: &str = include_str!("fragment.shader");

#[derive(Debug)]
pub struct Renderer {
    shader_program: ShaderProgram,
    triangle_vao: u32,
    triangle_ebo: u32,
    line_vao: u32,
}

#[derive(Debug)]
struct ShaderProgram {
    id: u32,
    vertex_shader: u32,
    fragment_shader: u32,
}

/// xyz
#[derive(Debug, Copy, Clone)]
struct Position(f32, f32, f32);
/// rgb
#[derive(Debug, Copy, Clone)]
struct RGBColor(u8, u8, u8);

#[allow(dead_code)]
struct VertexData {
    x: GLfloat,
    y: GLfloat,
    z: GLfloat,
    r: GLfloat,
    g: GLfloat,
    b: GLfloat,
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {
            shader_program: ShaderProgram::new(VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC),
            triangle_vao: u32::default(),
            triangle_ebo: u32::default(),
            line_vao: u32::default(),
        }
    }

    pub fn set_window_size(&self, width: u32, height: u32) {
        let projection = Mat4::orthographic_lh(0.0, width as f32, height as f32, 0.0, -1.0, 1.0);
        unsafe {
            let projection_name = CString::new("projection").unwrap();
            gl::UseProgram(self.shader_program.id);
            let location = gl::GetUniformLocation(self.shader_program.id, projection_name.as_ptr());
            gl::UniformMatrix4fv(location, 1, gl::FALSE, &projection.to_cols_array()[0]);
        }
    }

    pub fn render(&self, _game_state: &GameState) {
        unsafe {
            gl::ClearColor(0.0, 129.0 / 255.0, 129.0 / 255.0, 1.0); // set background
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(self.shader_program.id);
            gl::BindVertexArray(self.triangle_vao);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const _);
            gl::BindVertexArray(self.line_vao);
            gl::DrawArrays(gl::LINES, 0, 12);
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.triangle_vao);
            gl::DeleteVertexArrays(1, &self.triangle_vao);
        }
    }
}

impl ShaderProgram {
    fn new(vertex_shader_src: &str, fragment_shader_src: &str) -> Self {
        let vertex_shader = compile_shader(vertex_shader_src, gl::VERTEX_SHADER);
        let fragment_shader = compile_shader(fragment_shader_src, gl::FRAGMENT_SHADER);
        let id = link_program(vertex_shader, fragment_shader);
        Self {
            vertex_shader,
            fragment_shader,
            id,
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
            gl::DeleteShader(self.fragment_shader);
            gl::DeleteShader(self.vertex_shader);
        }
    }
}

impl VertexData {
    fn new(Position(x, y, z): Position, RGBColor(r, g, b): RGBColor) -> Self {
        VertexData {
            x: x as f32,
            y: y as f32,
            z: z as f32,
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
        }
    }

    fn bind_vbo_to_vao(vbo: u32, vao: u32) {
        unsafe {
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            let position_location = 0;
            let color_location = 1;

            gl::VertexAttribPointer(
                position_location,
                3,
                gl::FLOAT,
                gl::FALSE,
                size_of::<VertexData>().try_into().unwrap(),
                (0 * size_of::<GLfloat>()) as *const _,
            );

            gl::VertexAttribPointer(
                color_location,
                3,
                gl::FLOAT,
                gl::FALSE,
                6 * size_of::<GLfloat>() as i32,
                (3 * size_of::<GLfloat>()) as *const _,
            );

            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
        }
    }
}

pub fn setup_shader_program(game_renderer: &mut Renderer) {
    unsafe {
        let (screen_w, screen_h) = (800.0, 600.0);
        let (rect_w, rect_h) = (92.0, 24.0);
        let rect_x = (screen_w - rect_w) / 2.0;
        let rect_y = (screen_h - rect_h) / 2.0;

        // button triangles
        let grey = RGBColor(194, 194, 194);
        let rectangle_vertices: [VertexData; 4] = [
            VertexData::new(Position(rect_x, rect_y, 0.0), grey),
            VertexData::new(Position(rect_x + rect_w, rect_y, 0.0), grey),
            VertexData::new(Position(rect_x, rect_y + rect_h, 0.0), grey),
            VertexData::new(Position(rect_x + rect_w, rect_y + rect_h, 0.0), grey),
        ];
        let rectangle_indices: [(GLuint, GLuint, GLuint); 2] = [(0, 1, 2), (1, 2, 3)];

        // FIXME: We have to offset the coordinates in order to not get missing pixels
        // this needs to be kept in mind when a "draw_line" function is written.
        // This might be fixable with "gl::MatrixMode"
        // http://factor-language.blogspot.com/2008/11/some-recent-ui-rendering-fixes.html
        // https://www.khronos.org/opengl/wiki/Viewing_and_Transformations#How_do_I_draw_2D_controls_over_my_3D_rendering?
        let white = RGBColor(255, 255, 255);
        let dark_grey = RGBColor(129, 129, 129);
        let black = RGBColor(0, 0, 0);
        #[rustfmt::skip]
        let line_vertices: [VertexData; 12] = [
            // horizontal white line
            VertexData::new(Position(rect_x - 0.5, rect_y - 0.5, 0.0), white),
            VertexData::new(Position(rect_x + rect_w, rect_y, 0.0), white),
            // vertical white line
            VertexData::new(Position(rect_x, rect_y - 0.5, 0.0), white),
            VertexData::new(Position(rect_x, rect_y + rect_h, 0.0), white),

            // horizontal grey line
            VertexData::new(Position(rect_x + 0.5, rect_y + rect_h - 1.0, 0.0), dark_grey),
            VertexData::new(Position(rect_x + rect_w - 1.0, rect_y + rect_h - 1.0, 0.0), dark_grey),
            // vertical grey line
            VertexData::new(Position(rect_x + rect_w - 1.0, rect_y - 0.5 + 1.0, 0.0), dark_grey),
            VertexData::new(Position(rect_x + rect_w - 1.0, rect_y - 0.5 + rect_h - 1.0, 0.0), dark_grey),

            // horizontal black line
            VertexData::new(Position(rect_x, rect_y + rect_h, 0.0), black),
            VertexData::new(Position(rect_x + rect_w, rect_y + rect_h, 0.0), black),
            // vertical black line
            VertexData::new(Position(rect_x + rect_w, rect_y, 0.0), black),
            VertexData::new(Position(rect_x + rect_w, rect_y + rect_h, 0.0), black),
        ];

        // bind rectangle
        {
            let vao = new_vao();
            let vbo = new_vbo(&rectangle_vertices);
            let ebo = new_ebo(vao);
            VertexData::bind_vbo_to_vao(vbo, vao);
            bind_ebo_to_vao(ebo, vao, &rectangle_indices);
            game_renderer.triangle_vao = vao;
            game_renderer.triangle_ebo = ebo;
        }

        // bind lines
        {
            let vao = new_vao();
            let vbo = new_vbo(&line_vertices);
            VertexData::bind_vbo_to_vao(vbo, vao);
            game_renderer.line_vao = vao;
        }

        // Setup fragment output
        let frag_data_name = CString::new("frag_color").unwrap();
        gl::BindFragDataLocation(game_renderer.shader_program.id, 0, frag_data_name.as_ptr());
    }
}

fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    let shader;
    unsafe {
        shader = gl::CreateShader(ty);
        // Attempt to compile the shader
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);

        // Get the compile status
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(
                shader,
                len,
                std::ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );
            panic!(
                "{}",
                std::str::from_utf8(&buf)
                    .ok()
                    .expect("ShaderInfoLog not valid utf8")
            );
        }
    }
    shader
}

fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);
        // Get the link status
        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len: GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetProgramInfoLog(
                program,
                len,
                std::ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );
            panic!(
                "{}",
                std::str::from_utf8(&buf)
                    .ok()
                    .expect("ProgramInfoLog not valid utf8")
            );
        }
        program
    }
}

fn size_of_buf<T>(buf: &[T]) -> usize {
    buf.len() * size_of::<T>()
}

fn new_vbo<T>(data: &[T]) -> u32 {
    let mut vbo = u32::default();
    unsafe {
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            size_of_buf(data) as _,
            data as *const _ as *const c_void,
            gl::STATIC_DRAW,
        )
    }
    vbo
}

fn new_vao() -> u32 {
    let mut vao = u32::default();
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
    }
    vao
}

fn new_ebo(vao: u32) -> u32 {
    let mut ebo = u32::default();
    unsafe {
        gl::BindVertexArray(vao);
        gl::GenBuffers(1, &mut ebo);
    }
    ebo
}

fn bind_ebo_to_vao<T>(ebo: u32, vao: u32, indices: &[T]) {
    unsafe {
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            size_of_buf(indices) as GLsizeiptr,
            indices as *const _ as *const c_void,
            gl::STATIC_DRAW,
        );
    }
}
