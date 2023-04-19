use crate::midpoint;
use gl::types::*;
use glam::Mat4;
use itertools::Itertools;
use std::{
    ffi::{c_void, CString},
    mem::size_of,
};

const VERTEX_SHADER_SRC: &str = include_str!("vertex.shader");
const FRAGMENT_SHADER_SRC: &str = include_str!("fragment.shader");

#[derive(Debug)]
pub struct Renderer {
    shader: ShaderData,
    draw: DrawData,
}

#[derive(Debug)]
struct ShaderData {
    program: u32,
    vertex_shader: u32,
    fragment_shader: u32,
    primitives_vbo: u32,
    primitives_vao: u32,
}

#[derive(Debug)]
enum PrimitiveType {
    Line,
    Triangle,
    Point,
}

/// Metadata for a sequence of vertices in the vertex buffer
#[derive(Debug)]
struct VertexSection {
    length: usize,            // The number of vertices in the section
    primitive: PrimitiveType, // The primitive to draw the vertices as
}

#[derive(Debug)]
struct DrawData {
    active_color: ColorRGBA,
    vertices: Vec<Vertex>,
    sections: Vec<VertexSection>,
}

/// xyz
#[derive(Debug, Copy, Clone)]
struct Position(f32, f32, f32);
/// rgb
#[derive(Debug, Copy, Clone)]
struct ColorRGBA(u8, u8, u8, u8);

#[allow(dead_code)]
#[derive(Debug)]
struct Vertex {
    x: GLfloat,
    y: GLfloat,
    z: GLfloat,
    r: GLfloat,
    g: GLfloat,
    b: GLfloat,
    a: GLfloat,
}

#[no_mangle]
extern "system" fn on_opengl_debug_message(
    _source: u32,
    _type: u32,
    _id: u32,
    severity: u32,
    _length: i32,
    message: *const i8,
    _user_param: *mut c_void,
) {
    unsafe {
        let formatted_message = format!(
            "OpenGL: {}",
            std::ffi::CStr::from_ptr(message).to_str().unwrap()
        );
        match severity {
            gl::DEBUG_SEVERITY_HIGH => log::error!("{}", formatted_message),
            gl::DEBUG_SEVERITY_MEDIUM => log::error!("{}", formatted_message),
            gl::DEBUG_SEVERITY_LOW => log::warn!("{}", formatted_message),
            _ => (),
        }
    }
}
impl Renderer {
    pub fn new() -> Self {
        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::DebugMessageCallback(on_opengl_debug_message, std::ptr::null());
        }

        let vertex_shader = compile_shader(VERTEX_SHADER_SRC, gl::VERTEX_SHADER);
        let fragment_shader = compile_shader(FRAGMENT_SHADER_SRC, gl::FRAGMENT_SHADER);
        let program = link_program(vertex_shader, fragment_shader);

        let primitives_vbo = new_vbo(1000, gl::DYNAMIC_DRAW);
        let primitives_vao = new_vao();
        Vertex::set_vao_attr_ptrs(primitives_vao, primitives_vbo);

        // TODO: consider only enabling blending when drawing vertices with an alpha value < 1.0?
        // we could store this as meta-data in the VertexSection
        // for now, just always have this enabled to keep things simple
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        Renderer {
            shader: ShaderData {
                program,
                vertex_shader,
                fragment_shader,
                primitives_vbo,
                primitives_vao,
            },
            draw: DrawData {
                active_color: ColorRGBA(0, 0, 0, 255),
                vertices: Vec::new(),
                sections: Vec::new(),
            },
        }
    }

    pub fn render(&self) {
        unsafe {
            // upload data to VBO
            gl::BindVertexArray(self.shader.primitives_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.shader.primitives_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                size_of_buf(&self.draw.vertices) as _,
                self.draw.vertices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            // clear
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(self.shader.program);
            gl::BindVertexArray(self.shader.primitives_vao);
            let mut buffer_offset = 0;
            for section in &self.draw.sections {
                let mode = match section.primitive {
                    PrimitiveType::Triangle => gl::TRIANGLES,
                    PrimitiveType::Line => gl::LINES,
                    PrimitiveType::Point => gl::POINTS,
                };
                gl::DrawArrays(mode, buffer_offset, section.length as i32);
                buffer_offset += section.length as i32;
            }
        }
    }

    pub fn on_window_resize(&self, width: u32, height: u32) {
        let projection = Mat4::orthographic_lh(0.0, width as f32, height as f32, 0.0, -1.0, 1.0);
        unsafe {
            gl::UseProgram(self.shader.program);
            let projection_name = CString::new("projection").unwrap();
            let location = gl::GetUniformLocation(self.shader.program, projection_name.as_ptr());
            gl::UniformMatrix4fv(location, 1, gl::FALSE, &projection.to_cols_array()[0]);
        }
    }

    pub fn clear(&mut self) {
        self.draw.active_color = ColorRGBA(0, 0, 0, 255);
        self.draw.vertices.clear();
        self.draw.sections.clear();
    }

    pub fn set_draw_color(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.draw.active_color = ColorRGBA(r, g, b, a);
    }

    #[allow(dead_code)]
    pub fn draw_point(&mut self, x: i32, y: i32) {
        self.draw.vertices.push(Vertex::new(
            Position(x as f32, y as f32, 0.0),
            self.draw.active_color,
        ));
        self.draw.sections.push(VertexSection {
            length: 1,
            primitive: PrimitiveType::Point,
        })
    }

    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
        let (x0, y0, x1, y1) = (x0 as f32, y0 as f32, x1 as f32, y1 as f32);
        let color = self.draw.active_color;

        // offset slightly to get around weird missing pixels
        let start = Position(x0 - 0.5, y0 - 0.5, 0.0);
        let end = Position(x1, y1, 0.0);

        self.draw.vertices.push(Vertex::new(start, color));
        self.draw.vertices.push(Vertex::new(end, color));

        self.draw.sections.push(VertexSection {
            length: 2,
            primitive: PrimitiveType::Line,
        })
    }

    pub fn draw_rect(&mut self, x: i32, y: i32, w: i32, h: i32) {
        let (x, y, w, h) = (x as f32, y as f32, w as f32, h as f32);
        let lines = [
            (x, y, x + w, y),         // top line
            (x, y, x, y + h),         // left line
            (x + w, y, x + w, y + h), // right line
            (x, y + h, x + w, y + h), // bottom line
        ];
        for (x0, y0, x1, y1) in lines {
            self.draw.vertices.push(Vertex::new(
                Position(x0 as f32, y0 as f32, 0.0),
                self.draw.active_color,
            ));
            self.draw.vertices.push(Vertex::new(
                Position(x1 as f32, y1 as f32, 0.0),
                self.draw.active_color,
            ));
        }
        self.draw.sections.push(VertexSection {
            length: 8,
            primitive: PrimitiveType::Line,
        })
    }

    pub fn draw_rect_fill(&mut self, x: i32, y: i32, w: i32, h: i32) {
        let (x, y, w, h) = (x as f32, y as f32, w as f32, h as f32);
        let color = self.draw.active_color;

        let top_left = Position(x, y, 0.0);
        let top_right = Position(x + w, y, 0.0);
        let bottom_left = Position(x, y + h, 0.0);
        let bottom_right = Position(x + w, y + h, 0.0);

        // first triangle
        self.draw.vertices.push(Vertex::new(top_left, color));
        self.draw.vertices.push(Vertex::new(top_right, color));
        self.draw.vertices.push(Vertex::new(bottom_left, color));

        // second triangle
        self.draw.vertices.push(Vertex::new(top_right, color));
        self.draw.vertices.push(Vertex::new(bottom_left, color));
        self.draw.vertices.push(Vertex::new(bottom_right, color));

        self.draw.sections.push(VertexSection {
            length: 6,
            primitive: PrimitiveType::Triangle,
        })
    }

    #[allow(dead_code)]
    pub fn draw_circle(&mut self, center_x: i32, center_y: i32, radius: u32) {
        let circle_vertices = midpoint::circle_points(radius).into_iter().map(|(x, y)| {
            Vertex::new(
                Position((center_x + x) as f32, (center_y + y) as f32, 0.0),
                self.draw.active_color,
            )
        });

        let prev_vertices_len = self.draw.vertices.len();
        self.draw.vertices.extend(circle_vertices);

        self.draw.sections.push(VertexSection {
            length: self.draw.vertices.len() - prev_vertices_len,
            primitive: PrimitiveType::Point,
        })
    }

    pub fn draw_fill_circle(&mut self, center_x: i32, center_y: i32, radius: u32) {
        let half_circle_points = midpoint::circle_points(radius)
            .into_iter()
            .filter(|(_, y)| *y >= 0) // grab upper half of circle
            .unique_by(|(x, _)| *x); // make sure we don't overlap any lines (messes with transparency)
        let line_vertices = half_circle_points.flat_map(|(x, y)| {
            [
                // start the line on upper half circle
                Vertex::new(
                    Position((center_x + x) as f32, (center_y + y) as f32, 0.0),
                    self.draw.active_color,
                ),
                // end the line on lower half circle
                Vertex::new(
                    Position((center_x + x) as f32, (center_y - y) as f32, 0.0),
                    self.draw.active_color,
                ),
            ]
        });

        let prev_vertices_len = self.draw.vertices.len();
        self.draw.vertices.extend(line_vertices);

        self.draw.sections.push(VertexSection {
            length: self.draw.vertices.len() - prev_vertices_len,
            primitive: PrimitiveType::Line,
        })
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.shader.program);
            gl::DeleteShader(self.shader.fragment_shader);
            gl::DeleteShader(self.shader.vertex_shader);
            gl::DeleteBuffers(1, &self.shader.primitives_vao);
            gl::DeleteVertexArrays(1, &self.shader.primitives_vao);
        }
    }
}

impl Vertex {
    fn new(Position(x, y, z): Position, ColorRGBA(r, g, b, a): ColorRGBA) -> Self {
        Vertex {
            x: x as f32,
            y: y as f32,
            z: z as f32,
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    /// Setup attribute pointers for binding `Vertex` data to a VAO
    fn set_vao_attr_ptrs(vao: u32, vbo: u32) {
        unsafe {
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            let position_location = 0;
            let position_size = 3;
            let position_stride = size_of::<Vertex>();
            let position_offset = 0 * size_of::<GLfloat>();

            gl::VertexAttribPointer(
                position_location,
                position_size,
                gl::FLOAT,
                gl::FALSE,
                position_stride as i32,
                position_offset as *const _,
            );

            let color_location = 1;
            let color_size = 4;
            let color_stride = size_of::<Vertex>();
            let color_offset = 3 * size_of::<GLfloat>();

            gl::VertexAttribPointer(
                color_location,
                color_size,
                gl::FLOAT,
                gl::FALSE,
                color_stride as i32,
                color_offset as *const _,
            );

            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
        }
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

fn new_vbo(size: usize, usage: GLenum) -> u32 {
    let mut vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut vbo);
        gl::BufferData(gl::ARRAY_BUFFER, size as isize, std::ptr::null(), usage);
    }
    vbo
}

fn new_vao() -> u32 {
    let mut vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
    }
    vao
}
