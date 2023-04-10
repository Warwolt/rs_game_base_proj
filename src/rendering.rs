use gl::types::*;
use glam::Mat4;
use std::{ffi::CString, mem::size_of};

use crate::game_state::GameState;

const VERTEX_SHADER_SRC: &str = include_str!("vertex.shader");
const FRAGMENT_SHADER_SRC: &str = include_str!("fragment.shader");

pub struct Renderer {
    shader_programs: [ShaderProgram; 1],
    vertex_array_objects: [u32; 1],
}

struct ShaderProgram {
    id: u32,
    vertex_shader: u32,
    fragment_shader: u32,
}

/// xyz
struct Position(f32, f32, f32);
/// rgb
struct Color(f32, f32, f32);

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
        let shader_programs = [ShaderProgram::new(VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC)];
        let mut vertex_array_objects = [u32::default(); 1];

        unsafe {
            gl::GenVertexArrays(
                vertex_array_objects.len() as i32,
                vertex_array_objects.as_mut_ptr(),
            );
        }

        Renderer {
            shader_programs,
            vertex_array_objects,
        }
    }

    pub fn set_window_size(&self, width: u32, height: u32) {
        let projection = Mat4::orthographic_lh(0.0, width as f32, 0.0, height as f32, -1.0, 1.0);
        unsafe {
            let shader = self.shader_programs[0].id;
            let projection_name = CString::new("projection").unwrap();
            gl::UseProgram(shader);
            let location = gl::GetUniformLocation(shader, projection_name.as_ptr());
            gl::UniformMatrix4fv(location, 1, gl::FALSE, &projection.to_cols_array()[0]);
        }
    }

    pub fn render(&self, _game_state: &GameState) {
        unsafe {
            gl::ClearColor(0.0, 0.5, 0.5, 1.0); // set background
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(self.shader_programs[0].id);
            gl::BindVertexArray(self.vertex_array_objects[0]);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }
    }

    fn add_vertex_buffer_object(&self, vao_index: usize, vertices: &[VertexData]) -> u32 {
        unsafe {
            // create buffer object
            gl::BindVertexArray(self.vertex_array_objects[vao_index]);
            let mut vertex_buffer_object = u32::default();
            gl::GenBuffers(1, &mut vertex_buffer_object);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer_object);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * size_of::<VertexData>()) as GLsizeiptr,
                std::mem::transmute(&vertices[0]),
                gl::STATIC_DRAW,
            );

            //
            // Note: This config is assuming the location of the position and
            // color attribute, but if we're assuming that we rely on a specific
            // shader, so shouldn't this function be part of the ShaderProgram
            // struct?
            //

            // configure position attribute
            {
                let position_location = 0;
                gl::VertexAttribPointer(
                    position_location,
                    3,
                    gl::FLOAT,
                    gl::FALSE,
                    size_of::<VertexData>().try_into().unwrap(),
                    0 as *const _,
                );
                gl::EnableVertexAttribArray(0);
            }

            // configure color attribute
            {
                let color_location = 1;
                gl::VertexAttribPointer(
                    color_location,
                    3,
                    gl::FLOAT,
                    gl::FALSE,
                    6 * size_of::<GLfloat>() as i32,
                    (3 * size_of::<GLfloat>()) as *const _,
                );
                gl::EnableVertexAttribArray(1);
            }

            vertex_buffer_object
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(
                self.vertex_array_objects.len() as i32,
                self.vertex_array_objects.as_ptr(),
            );
            gl::DeleteVertexArrays(
                self.vertex_array_objects.len() as i32,
                self.vertex_array_objects.as_ptr(),
            );
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
    fn new(Position(x, y, z): Position, Color(r, g, b): Color) -> Self {
        VertexData { x, y, z, r, g, b }
    }
}

pub fn setup_triangle_program(game_renderer: &mut Renderer) {
    unsafe {
        let triangle_1: [VertexData; 3] = [
            VertexData::new(Position(400.0, 450.0, 0.0), Color(1.0, 0.0, 0.0)),
            VertexData::new(Position(600.0, 150.0, 0.0), Color(0.0, 1.0, 0.0)),
            VertexData::new(Position(200.0, 150.0, 0.0), Color(0.0, 0.0, 1.0)),
        ];

        game_renderer.add_vertex_buffer_object(0, &triangle_1);

        // Setup fragment output
        let frag_data_name = CString::new("frag_color").unwrap();
        gl::BindFragDataLocation(
            game_renderer.shader_programs[0].id,
            0,
            frag_data_name.as_ptr(),
        );
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
