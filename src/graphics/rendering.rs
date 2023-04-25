use crate::{geometry::Rect, graphics::midpoint};
use gl::types::*;
use glam::Mat4;
use image::GenericImageView;
use itertools::Itertools;
use std::{
    collections::HashMap,
    ffi::{c_void, CString},
    mem::size_of,
};

#[derive(Debug)]
pub struct Renderer {
    shader: ShaderData,
    draw: DrawData,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Hash)]
pub struct TextureID(u32);

#[derive(Debug, Clone, Copy)]
pub struct TextureData {
    pub id: TextureID,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
struct ShaderData {
    program: u32,
    vertex_shader: u32,
    fragment_shader: u32,
    vbo: u32,
    vao: u32,
    white_texture_id: u32,
    textures: HashMap<TextureID, TextureData>,
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
    texture_id: u32,          // The texture to draw with
    color_key: ColorRGBA,     // The RGBA value to draw transparently
}

#[derive(Debug)]
struct DrawData {
    active_color: ColorRGBA,
    active_color_key: ColorRGBA,
    vertices: Vec<Vertex>,
    sections: Vec<VertexSection>,
}

/// xyz
#[derive(Debug, Copy, Clone)]
struct Position(f32, f32, f32);
/// rgb
#[derive(Debug, Copy, Clone)]
struct ColorRGBA(u8, u8, u8, u8);
// uv
#[derive(Debug, Copy, Clone)]
struct TextureUV(f32, f32);

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct Vertex {
    pos: VertexPosition,
    color: VertexColor,
    texture_uv: VertexTextureUV,
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct VertexPosition {
    x: GLfloat,
    y: GLfloat,
    z: GLfloat,
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct VertexColor {
    r: GLfloat,
    g: GLfloat,
    b: GLfloat,
    a: GLfloat,
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct VertexTextureUV {
    u: GLfloat,
    v: GLfloat,
}

const VERTEX_SHADER_SRC: &str = include_str!("shaders/vertex.shader");
const FRAGMENT_SHADER_SRC: &str = include_str!("shaders/fragment.shader");

pub fn texture_from_image_path(renderer: &mut Renderer, path: &str) -> TextureData {
    let image = image::open(path).unwrap().flipv();
    let (width, height) = image.dimensions();
    let data = image
        .pixels()
        .flat_map(|(_x, _y, pixel)| pixel.0)
        .collect::<Vec<u8>>();
    let id = renderer.add_texture(&data, width, height);

    log::info!("Loaded image \"{}\"", path);

    TextureData { id, width, height }
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

impl Renderer {
    // FIXME: should this have window dimensions as parameters? Right now client
    // has to call on_window_resize after constructing, which seems a little fucked.
    pub fn new() -> Self {
        // Enable OpenGL debug logging
        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::DebugMessageCallback(on_opengl_debug_message, std::ptr::null());
        }

        // Setup shader program
        let vertex_shader = compile_shader(VERTEX_SHADER_SRC, gl::VERTEX_SHADER);
        let fragment_shader = compile_shader(FRAGMENT_SHADER_SRC, gl::FRAGMENT_SHADER);
        let program = link_program(vertex_shader, fragment_shader);

        // Setup drawing buffer
        let primitives_vbo = new_vbo();
        let primitives_vao = new_vao();
        Vertex::set_attribute_pointers(primitives_vao, primitives_vbo);

        // Setup default texture when drawing primitves
        let white_texture = new_texture();
        upload_data_to_texture(white_texture, &[255, 255, 255, 255], 1, 1);

        // Enable alpha blending
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        Renderer {
            shader: ShaderData {
                program,
                vertex_shader,
                fragment_shader,
                vbo: primitives_vbo,
                vao: primitives_vao,
                white_texture_id: white_texture,
                textures: HashMap::new(),
            },
            draw: DrawData {
                active_color: ColorRGBA(0, 0, 0, 255),
                active_color_key: ColorRGBA(0, 0, 0, 0),
                vertices: Vec::new(),
                sections: Vec::new(),
            },
        }
    }

    pub fn render(&self) {
        unsafe {
            // upload data to VBO
            gl::BindVertexArray(self.shader.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.shader.vbo);
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
            gl::BindVertexArray(self.shader.vao);
            let mut buffer_offset = 0;
            for section in &self.draw.sections {
                let mode = match section.primitive {
                    PrimitiveType::Triangle => gl::TRIANGLES,
                    PrimitiveType::Line => gl::LINES,
                    PrimitiveType::Point => gl::POINTS,
                };

                self.set_color_key_uniform(section.color_key);
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, section.texture_id);
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

    #[allow(dead_code)]
    pub fn add_texture(&mut self, rgba_data: &[u8], width: u32, height: u32) -> TextureID {
        let id = TextureID(new_texture());
        self.shader
            .textures
            .insert(id, TextureData { id, width, height });
        upload_data_to_texture(id.0, rgba_data, width, height);
        id
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.draw.active_color = ColorRGBA(0, 0, 0, 255);
        self.draw.vertices.clear();
        self.draw.sections.clear();
    }

    #[allow(dead_code)]
    pub fn set_draw_color(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.draw.active_color = ColorRGBA(r, g, b, a);
    }

    #[allow(dead_code)]
    pub fn set_color_key(&mut self, r: u8, g: u8, b: u8) {
        self.draw.active_color_key = ColorRGBA(r, g, b, 255);
    }

    #[allow(dead_code)]
    pub fn disable_color_key(&mut self) {
        self.draw.active_color_key = ColorRGBA(0, 0, 0, 0);
    }

    #[allow(dead_code)]
    pub fn draw_point(&mut self, x: i32, y: i32) {
        self.draw.vertices.push(Vertex::with_color(
            Position(x as f32, y as f32, 0.0),
            self.draw.active_color,
        ));
        self.draw.sections.push(VertexSection {
            length: 1,
            primitive: PrimitiveType::Point,
            texture_id: self.shader.white_texture_id,
            color_key: self.draw.active_color_key,
        })
    }

    #[allow(dead_code)]
    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
        let (x0, y0, x1, y1) = (x0 as f32, y0 as f32, x1 as f32, y1 as f32);
        let color = self.draw.active_color;

        // offset slightly to get around weird missing pixels
        let start = Position(x0 - 0.5, y0 - 0.5, 0.0);
        let end = Position(x1, y1, 0.0);

        self.draw.vertices.push(Vertex::with_color(start, color));
        self.draw.vertices.push(Vertex::with_color(end, color));

        self.draw.sections.push(VertexSection {
            length: 2,
            primitive: PrimitiveType::Line,
            texture_id: self.shader.white_texture_id,
            color_key: self.draw.active_color_key,
        })
    }

    #[allow(dead_code)]
    pub fn draw_rect(&mut self, x: i32, y: i32, w: i32, h: i32) {
        let (x, y, w, h) = (x as f32, y as f32, w as f32, h as f32);
        let lines = [
            (x, y, x + w, y),         // top line
            (x, y, x, y + h),         // left line
            (x + w, y, x + w, y + h), // right line
            (x, y + h, x + w, y + h), // bottom line
        ];
        for (x0, y0, x1, y1) in lines {
            self.draw.vertices.push(Vertex::with_color(
                Position(x0 as f32, y0 as f32, 0.0),
                self.draw.active_color,
            ));
            self.draw.vertices.push(Vertex::with_color(
                Position(x1 as f32, y1 as f32, 0.0),
                self.draw.active_color,
            ));
        }
        self.draw.sections.push(VertexSection {
            length: 8,
            primitive: PrimitiveType::Line,
            texture_id: self.shader.white_texture_id,
            color_key: self.draw.active_color_key,
        })
    }

    #[allow(dead_code)]
    pub fn draw_rect_fill(&mut self, x: i32, y: i32, w: i32, h: i32) {
        let (x, y, w, h) = (x as f32, y as f32, w as f32, h as f32);
        let color = self.draw.active_color;
        let vertices = &mut self.draw.vertices;

        let top_left = Position(x, y, 0.0);
        let top_right = Position(x + w, y, 0.0);
        let bottom_left = Position(x, y + h, 0.0);
        let bottom_right = Position(x + w, y + h, 0.0);

        // first triangle
        vertices.push(Vertex::with_color(top_left, color));
        vertices.push(Vertex::with_color(top_right, color));
        vertices.push(Vertex::with_color(bottom_left, color));

        // second triangle
        vertices.push(Vertex::with_color(top_right, color));
        vertices.push(Vertex::with_color(bottom_left, color));
        vertices.push(Vertex::with_color(bottom_right, color));

        self.draw.sections.push(VertexSection {
            length: 6,
            primitive: PrimitiveType::Triangle,
            texture_id: self.shader.white_texture_id,
            color_key: self.draw.active_color_key,
        })
    }

    #[allow(dead_code)]
    pub fn draw_circle(&mut self, center_x: i32, center_y: i32, radius: u32) {
        let circle_vertices = midpoint::circle_points(radius).into_iter().map(|(x, y)| {
            Vertex::with_color(
                Position((center_x + x) as f32, (center_y + y) as f32, 0.0),
                self.draw.active_color,
            )
        });

        let prev_vertices_len = self.draw.vertices.len();
        self.draw.vertices.extend(circle_vertices);

        self.draw.sections.push(VertexSection {
            length: self.draw.vertices.len() - prev_vertices_len,
            primitive: PrimitiveType::Point,
            texture_id: self.shader.white_texture_id,
            color_key: self.draw.active_color_key,
        })
    }

    #[allow(dead_code)]
    pub fn draw_fill_circle(&mut self, center_x: i32, center_y: i32, radius: u32) {
        let half_circle_points = midpoint::circle_points(radius)
            .into_iter()
            .filter(|(_, y)| *y >= 0) // grab upper half of circle
            .unique_by(|(x, _)| *x); // make sure we don't overlap any lines (messes with transparency)
        let line_vertices = half_circle_points.flat_map(|(x, y)| {
            [
                // start the line on upper half circle
                Vertex::with_color(
                    Position((center_x + x) as f32, (center_y + y) as f32, 0.0),
                    self.draw.active_color,
                ),
                // end the line on lower half circle
                Vertex::with_color(
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
            texture_id: self.shader.white_texture_id,
            color_key: self.draw.active_color_key,
        })
    }

    #[allow(dead_code)]
    #[rustfmt::skip]
    /// Draw a texture to the screen.
    /// * `texture_id` the texture to draw
    /// * `draw_rect`  specifies the quad that the texture will be drawn on
    /// * `clip_rect`  optionally specifies what portion of the texture to use
    pub fn draw_texture(
        &mut self,
        texture_id: TextureID,
        draw_rect: Rect,
        clip_rect: Option<Rect>,
    ) {
        let (draw_x, draw_y, draw_w, draw_h) = (
            draw_rect.x as f32,
            draw_rect.y as f32,
            draw_rect.w as f32,
            draw_rect.h as f32,
        );
        let vertices = &mut self.draw.vertices;

        let top_left_xy = Position(draw_x, draw_y, 0.0);
        let top_right_xy = Position(draw_x + draw_w, draw_y, 0.0);
        let bottom_left_xy = Position(draw_x, draw_y + draw_h, 0.0);
        let bottom_right_xy = Position(draw_x + draw_w, draw_y + draw_h, 0.0);

        let top_left_uv;
        let top_right_uv;
        let bottom_left_uv;
        let bottom_right_uv;

        if let Some(clip_rect) = clip_rect {
            let texture = self.shader.textures[&texture_id];
            let (tex_w, tex_h) = (texture.width as f32, texture.height as f32);
            let (clip_x, clip_y, clip_w, clip_h) = (
                clip_rect.x as f32,
                clip_rect.y as f32,
                clip_rect.w as f32,
                clip_rect.h as f32,
            );
            top_left_uv = TextureUV(clip_x / tex_w, 1.0 - clip_y / tex_h);
            top_right_uv = TextureUV((clip_x + clip_w) / tex_w, 1.0 - clip_y / tex_h);
            bottom_left_uv = TextureUV(clip_x / tex_w, 1.0 - (clip_y + clip_h) / tex_h);
            bottom_right_uv = TextureUV((clip_x + clip_w) / tex_w, 1.0 - (clip_y + clip_h) / tex_h);
        } else {
            top_left_uv = TextureUV(0.0, 1.0);
            top_right_uv = TextureUV(1.0, 1.0);
            bottom_left_uv = TextureUV(0.0, 0.0);
            bottom_right_uv = TextureUV(1.0, 0.0);
        }

        // first triangle
        vertices.push(Vertex::with_uv(top_left_xy, top_left_uv));
        vertices.push(Vertex::with_uv(top_right_xy, top_right_uv));
        vertices.push(Vertex::with_uv(bottom_left_xy, bottom_left_uv));

        // second triangle
        vertices.push(Vertex::with_uv(top_right_xy, top_right_uv));
        vertices.push(Vertex::with_uv(bottom_left_xy, bottom_left_uv));
        vertices.push(Vertex::with_uv(bottom_right_xy, bottom_right_uv));

        self.draw.sections.push(VertexSection {
            length: 6,
            primitive: PrimitiveType::Triangle,
            texture_id: texture_id.0,
            color_key: self.draw.active_color_key,
        })
    }

    fn set_color_key_uniform(&self, color_key: ColorRGBA) {
        set_uniform_vec4f(
            self.shader.program,
            "color_key",
            color_key.0 as f32 / 255.0,
            color_key.1 as f32 / 255.0,
            color_key.2 as f32 / 255.0,
            color_key.3 as f32 / 255.0,
        );
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.shader.program);
            gl::DeleteShader(self.shader.fragment_shader);
            gl::DeleteShader(self.shader.vertex_shader);
            gl::DeleteBuffers(1, &self.shader.vao);
            gl::DeleteTextures(1, &self.shader.white_texture_id);
            for (id, _) in &self.shader.textures {
                gl::DeleteTextures(1, &id.0);
            }
            gl::DeleteVertexArrays(1, &self.shader.vao);
        }
    }
}

impl Vertex {
    fn with_color(Position(x, y, z): Position, ColorRGBA(r, g, b, a): ColorRGBA) -> Self {
        Vertex {
            pos: VertexPosition {
                x: x as f32,
                y: y as f32,
                z: z as f32,
            },
            color: VertexColor {
                r: r as f32 / 255.0,
                g: g as f32 / 255.0,
                b: b as f32 / 255.0,
                a: a as f32 / 255.0,
            },
            texture_uv: VertexTextureUV { u: 0.0, v: 0.0 },
        }
    }

    fn with_uv(Position(x, y, z): Position, TextureUV(u, v): TextureUV) -> Self {
        Vertex {
            pos: VertexPosition {
                x: x as f32,
                y: y as f32,
                z: z as f32,
            },
            color: VertexColor {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            texture_uv: VertexTextureUV { u, v },
        }
    }

    /// Setup attribute pointers for binding `Vertex` data to a VAO
    fn set_attribute_pointers(vao: u32, vbo: u32) {
        unsafe {
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            // position
            let position_location = 0;
            let position_size = size_of::<VertexPosition>() / size_of::<GLfloat>();
            let position_stride = size_of::<Vertex>();
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

            // color
            let color_location = 1;
            let color_size = size_of::<VertexColor>() / size_of::<GLfloat>();
            let color_stride = size_of::<Vertex>();
            let color_offset = size_of::<VertexPosition>();

            gl::VertexAttribPointer(
                color_location,
                color_size as i32,
                gl::FLOAT,
                gl::FALSE,
                color_stride as i32,
                color_offset as *const c_void,
            );
            gl::EnableVertexAttribArray(1);

            // texture uv
            let texture_uv_location = 2;
            let texture_uv_size = size_of::<VertexTextureUV>() / size_of::<GLfloat>();
            let texture_uv_stride = size_of::<Vertex>();
            let texture_uv_offset = size_of::<VertexPosition>() + size_of::<VertexColor>();

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

        assert_no_gl_error!();
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

        assert_no_gl_error!();

        program
    }
}

fn size_of_buf<T>(buf: &[T]) -> usize {
    buf.len() * size_of::<T>()
}

fn new_vbo() -> u32 {
    let mut vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut vbo);
        assert_no_gl_error!();
    }
    vbo
}

fn new_vao() -> u32 {
    let mut vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        assert_no_gl_error!();
    }
    vao
}

fn new_texture() -> u32 {
    let mut texture_id = 0;
    unsafe {
        gl::GenTextures(1, &mut texture_id);
        gl::BindTexture(gl::TEXTURE_2D, texture_id);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

        gl::BindTexture(gl::TEXTURE_2D, 0);
        assert_no_gl_error!();
    }
    texture_id
}

fn set_uniform_vec4f(program: u32, name: &str, v0: f32, v1: f32, v2: f32, v3: f32) {
    unsafe {
        let name_cstr = CString::new(name).unwrap();
        let location = gl::GetUniformLocation(program, name_cstr.as_ptr() as *const i8);
        gl::Uniform4f(location, v0, v1, v2, v3);
    }
}

fn upload_data_to_texture(texture_id: u32, rgba_data: &[u8], width: u32, height: u32) {
    unsafe {
        gl::BindTexture(gl::TEXTURE_2D, texture_id);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            width as i32,
            height as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            rgba_data.as_ptr() as *const c_void,
        );
        assert_no_gl_error!();
    }
}
