use crate::{
    geometry::{Dimension, Rect},
    graphics::midpoint,
};
use gl::types::*;
use glam::Mat4;
use image::GenericImageView;
use itertools::Itertools;
use std::{
    collections::HashMap,
    ffi::{c_void, CString},
    mem::size_of,
    path::Path,
};

#[derive(Debug)]
pub struct Renderer {
    // Used for rendering user draw calls
    shader: ShaderData,
    /// Used for fixed resolution rendering
    canvas: Canvas,
    /// Store data from user draw calls
    draw: DrawData,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Hash)]
pub struct TextureID(u32);

#[derive(Debug)]
pub enum LoadError {
    ImageError(image::ImageError),
}

#[derive(Debug)]
struct ShaderProgram(GLuint);

#[derive(Debug)]
struct Shader(GLuint);

#[derive(Debug)]
struct ShaderData {
    program: ShaderProgram,
    _vertex_shader: Shader,
    _fragment_shader: Shader,
    vbo: u32,
    vao: u32,
    white_texture_id: u32,
    textures: HashMap<TextureID, TextureData>,
}

#[derive(Debug, Default)]
pub struct Canvas {
    pub pos: glam::IVec2,
    pub dim: Dimension,
    pub scaled_dim: Dimension,
    pub scale: f32,
    fbo: u32,
    vao: u32,
    texture: u32,
}

#[derive(Debug, Clone, Copy)]
struct TextureData {
    width: u32,
    height: u32,
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
    draw_color: ColorRGBA,
    texture_blend_color: ColorRGBA,
    active_color_key: ColorRGBA,
    vertices: Vec<Vertex>,
    sections: Vec<VertexSection>,
    window_width: f32,
    window_height: f32,
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

pub fn load_texture_from_image_path(
    renderer: &mut Renderer,
    path: &Path,
) -> Result<TextureID, LoadError> {
    let image = image::open(path)
        .map_err(|e| LoadError::ImageError(e))?
        .flipv();
    let (width, height) = image.dimensions();
    let data = image
        .pixels()
        .flat_map(|(_x, _y, pixel)| pixel.0)
        .collect::<Vec<u8>>();
    let id = renderer.add_texture(&data, width, height);

    Ok(id)
}

pub fn reload_texture_from_image_path(
    id: TextureID,
    renderer: &mut Renderer,
    path: &Path,
) -> Result<(), LoadError> {
    let image = image::open(path)
        .map_err(|e| LoadError::ImageError(e))?
        .flipv();
    let (width, height) = image.dimensions();
    let data = image
        .pixels()
        .flat_map(|(_x, _y, pixel)| pixel.0)
        .collect::<Vec<u8>>();
    renderer.reload_texture(id, &data, width, height);

    Ok(())
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
    pub fn new(
        window_width: u32,
        window_height: u32,
        canvas_width: u32,
        canvas_height: u32,
    ) -> Self {
        // Enable OpenGL debug logging
        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::DebugMessageCallback(on_opengl_debug_message, std::ptr::null());
        }

        // Setup shader program
        let vertex_shader = compile_shader(VERTEX_SHADER_SRC, gl::VERTEX_SHADER);
        let fragment_shader = compile_shader(FRAGMENT_SHADER_SRC, gl::FRAGMENT_SHADER);
        let program = link_program(&vertex_shader, &fragment_shader);

        // Setup drawing buffer
        let primitives_vbo = new_vbo();
        let primitives_vao = new_vao();
        Vertex::set_attribute_pointers(primitives_vao, primitives_vbo);

        // Setup default texture when drawing primitves
        let white_texture = new_texture();
        set_texture_image(white_texture, 1, 1, Some(&[255, 255, 255, 255]));

        // Setup drawing canvas (for fixed resolution rendering)
        let canvas_vao = new_vao();
        let canvas_vbo = new_vbo();
        let canvas_fbo = new_fbo();
        let canvas_texture = new_texture();
        set_texture_image(canvas_texture, canvas_width, canvas_height, None);
        set_framebuffer_texture(canvas_fbo, canvas_texture);
        set_vertex_data(
            canvas_vbo,
            &[
                Vertex::with_uv(Position(-1.0, -1.0, 0.0), TextureUV(0.0, 0.)), // bottom left
                Vertex::with_uv(Position(-1.0, 1.0, 0.0), TextureUV(0.0, 1.0)), // top left
                Vertex::with_uv(Position(1.0, 1.0, 0.0), TextureUV(1.0, 1.0)),  // top right
                Vertex::with_uv(Position(-1.0, -1.0, 0.0), TextureUV(0.0, 0.0)), // bottom left
                Vertex::with_uv(Position(1.0, 1.0, 0.0), TextureUV(1.0, 1.0)),  // top right
                Vertex::with_uv(Position(1.0, -1.0, 0.0), TextureUV(1.0, 0.0)), // bottom right
            ],
        );
        Vertex::set_attribute_pointers(canvas_vao, canvas_vbo);

        // Enable alpha blending
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        Renderer {
            shader: ShaderData {
                program,
                _vertex_shader: vertex_shader,
                _fragment_shader: fragment_shader,
                vbo: primitives_vbo,
                vao: primitives_vao,
                white_texture_id: white_texture,
                textures: HashMap::new(),
            },
            canvas: Canvas::new(
                canvas_fbo,
                canvas_vao,
                canvas_texture,
                window_width as f32,
                window_height as f32,
                canvas_width,
                canvas_height,
            ),
            draw: DrawData {
                draw_color: ColorRGBA(0, 0, 0, 255),
                texture_blend_color: ColorRGBA(255, 255, 255, 255),
                active_color_key: ColorRGBA(0, 0, 0, 0),
                vertices: Vec::new(),
                sections: Vec::new(),
                window_width: window_width as f32,
                window_height: window_height as f32,
            },
        }
    }

    pub fn render(&mut self) {
        /* Update canvas */
        self.canvas
            .update(self.draw.window_width, self.draw.window_height);

        /* Draw to canvas */
        unsafe {
            gl::UseProgram(self.shader.program.0);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.canvas.fbo);
            self.set_projection_matrix(
                0.0,
                self.canvas.dim.width as f32,
                self.canvas.dim.height as f32,
                0.0,
            );
            gl::Viewport(
                0,
                0,
                self.canvas.dim.width as i32,
                self.canvas.dim.height as i32,
            );

            // clear
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // draw vertices
            set_vertex_data(self.shader.vbo, &self.draw.vertices);
            gl::BindVertexArray(self.shader.vao);
            let mut buffer_index = 0;
            for section in &self.draw.sections {
                let mode = match section.primitive {
                    PrimitiveType::Triangle => gl::TRIANGLES,
                    PrimitiveType::Line => gl::LINES,
                    PrimitiveType::Point => gl::POINTS,
                };

                self.set_color_key_uniform(section.color_key);
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, section.texture_id);
                gl::DrawArrays(mode, buffer_index, section.length as i32);
                buffer_index += section.length as i32;
            }
        }

        /* Render canvas to screen */
        unsafe {
            gl::UseProgram(self.shader.program.0);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            self.set_projection_matrix(-1.0, 1.0, -1.0, 1.0);
            gl::Viewport(
                self.canvas.pos.x,
                self.canvas.pos.y,
                self.canvas.scaled_dim.width as i32,
                self.canvas.scaled_dim.height as i32,
            );

            // clear
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::BindVertexArray(self.canvas.vao);
            gl::BindTexture(gl::TEXTURE_2D, self.canvas.texture);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }
    }

    pub fn canvas(&self) -> &Canvas {
        &self.canvas
    }
    pub fn on_window_resize(&mut self, width: u32, height: u32) {
        self.draw.window_width = width as f32;
        self.draw.window_height = height as f32;
    }

    pub fn add_texture(&mut self, rgba_data: &[u8], width: u32, height: u32) -> TextureID {
        let id = TextureID(new_texture());
        self.shader
            .textures
            .insert(id, TextureData { width, height });
        set_texture_image(id.0, width, height, Some(rgba_data));
        id
    }

    pub fn reload_texture(&mut self, id: TextureID, rgba_data: &[u8], width: u32, height: u32) {
        self.shader
            .textures
            .insert(id, TextureData { width, height });
        set_texture_image(id.0, width, height, Some(rgba_data));
    }

    pub fn clear(&mut self) {
        self.draw.draw_color = ColorRGBA(0, 0, 0, 255);
        self.draw.vertices.clear();
        self.draw.sections.clear();
    }

    #[allow(dead_code)]
    pub fn set_draw_color(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.draw.draw_color = ColorRGBA(r, g, b, a);
    }

    #[allow(dead_code)]
    pub fn set_texture_blend_color(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.draw.texture_blend_color = ColorRGBA(r, g, b, a);
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
            self.draw.draw_color,
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
        let color = self.draw.draw_color;

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
    pub fn draw_rect(&mut self, rect: Rect) {
        let (x, y, w, h) = (rect.x, rect.y, rect.w as i32, rect.h as i32);
        let lines = [
            (x, y, x + w, y),         // top line
            (x, y, x, y + h),         // left line
            (x + w, y, x + w, y + h), // right line
            (x, y + h, x + w, y + h), // bottom line
        ];
        for (x0, y0, x1, y1) in lines {
            self.draw_line(x0, y0, x1, y1);
        }
    }

    #[allow(dead_code)]
    pub fn draw_rect_fill(&mut self, rect: Rect) {
        let (x, y, w, h) = (rect.x as f32, rect.y as f32, rect.w as f32, rect.h as f32);
        let color = self.draw.draw_color;
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
                self.draw.draw_color,
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
                    self.draw.draw_color,
                ),
                // end the line on lower half circle
                Vertex::with_color(
                    Position((center_x + x) as f32, (center_y - y) as f32, 0.0),
                    self.draw.draw_color,
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

        let blend_color = self.draw.texture_blend_color;

        // first triangle
        vertices.push(Vertex::new(top_left_xy, blend_color, top_left_uv));
        vertices.push(Vertex::new(top_right_xy, blend_color, top_right_uv));
        vertices.push(Vertex::new(bottom_left_xy, blend_color, bottom_left_uv));

        // second triangle
        vertices.push(Vertex::new(top_right_xy, blend_color, top_right_uv));
        vertices.push(Vertex::new(bottom_left_xy, blend_color, bottom_left_uv));
        vertices.push(Vertex::new(bottom_right_xy, blend_color, bottom_right_uv));

        self.draw.sections.push(VertexSection {
            length: 6,
            primitive: PrimitiveType::Triangle,
            texture_id: texture_id.0,
            color_key: self.draw.active_color_key,
        })
    }

    pub fn set_resolution(&mut self, resolution_width: u32, resolution_height: u32) {
        set_texture_image(
            self.canvas.texture,
            resolution_width,
            resolution_height,
            None,
        );
        self.canvas.dim.width = resolution_width;
        self.canvas.dim.height = resolution_height;
    }

    fn set_color_key_uniform(&self, color_key: ColorRGBA) {
        set_uniform_vec4f(
            self.shader.program.0,
            "color_key",
            color_key.0 as f32 / 255.0,
            color_key.1 as f32 / 255.0,
            color_key.2 as f32 / 255.0,
            color_key.3 as f32 / 255.0,
        );
    }

    fn set_projection_matrix(&self, left: f32, right: f32, bottom: f32, top: f32) {
        let projection = Mat4::orthographic_lh(left, right, bottom, top, -1.0, 1.0);
        unsafe {
            gl::UseProgram(self.shader.program.0);
            let projection_name = CString::new("projection").unwrap();
            let location = gl::GetUniformLocation(self.shader.program.0, projection_name.as_ptr());
            gl::UniformMatrix4fv(location, 1, gl::FALSE, &projection.to_cols_array()[0]);
        }
    }
}

impl Canvas {
    fn new(
        fbo: u32,
        vao: u32,
        texture: u32,
        window_width: f32,
        window_height: f32,
        canvas_width: u32,
        canvas_height: u32,
    ) -> Self {
        let mut canvas: Canvas = Default::default();

        canvas.dim.width = canvas_width;
        canvas.dim.height = canvas_height;
        canvas.scale = Self::calculate_scale(window_width, window_height, canvas.dim);
        canvas.scaled_dim = Self::calculate_scaled_dimensions(canvas.scale, canvas.dim);
        canvas.pos = Self::calculate_position(window_width, window_height, canvas.scaled_dim);
        canvas.fbo = fbo;
        canvas.vao = vao;
        canvas.texture = texture;

        canvas
    }

    fn update(&mut self, window_width: f32, window_height: f32) {
        self.scale = Self::calculate_scale(window_width, window_height, self.dim);
        self.scaled_dim = Self::calculate_scaled_dimensions(self.scale, self.dim);
        self.pos = Self::calculate_position(window_width, window_height, self.scaled_dim);
    }

    fn calculate_scale(window_width: f32, window_height: f32, dim: Dimension) -> f32 {
        f32::round(f32::min(
            window_width as f32 / dim.width as f32,
            window_height as f32 / dim.height as f32,
        ))
    }

    fn calculate_scaled_dimensions(scale: f32, dim: Dimension) -> Dimension {
        Dimension {
            width: (scale * dim.width as f32) as u32,
            height: (scale * dim.height as f32) as u32,
        }
    }

    fn calculate_position(
        window_width: f32,
        window_height: f32,
        scaled_dim: Dimension,
    ) -> glam::IVec2 {
        glam::ivec2(
            (window_width as i32 - scaled_dim.width as i32) / 2,
            (window_height as i32 - scaled_dim.height as i32) / 2,
        )
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.0);
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.0);
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
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
    fn new(
        Position(x, y, z): Position,
        ColorRGBA(r, g, b, a): ColorRGBA,
        TextureUV(u, v): TextureUV,
    ) -> Self {
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
            texture_uv: VertexTextureUV { u, v },
        }
    }

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

    #[allow(dead_code)]
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

fn compile_shader(src: &str, ty: GLenum) -> Shader {
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

    Shader(shader)
}

fn link_program(vs: &Shader, fs: &Shader) -> ShaderProgram {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs.0);
        gl::AttachShader(program, fs.0);
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

        ShaderProgram(program)
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

fn new_fbo() -> u32 {
    let mut fbo = 0;
    unsafe {
        gl::GenFramebuffers(1, &mut fbo);
        assert_no_gl_error!();
    }
    fbo
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

fn set_vertex_data(vbo: u32, vertices: &[Vertex]) {
    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            size_of_buf(vertices) as _,
            vertices.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );
        assert_no_gl_error!();
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }
}

fn set_texture_image(texture_id: u32, width: u32, height: u32, rgba_data: Option<&[u8]>) {
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
            if let Some(rgba_data) = rgba_data {
                rgba_data.as_ptr() as *const c_void
            } else {
                std::ptr::null()
            },
        );
        assert_no_gl_error!();
        gl::BindTexture(gl::TEXTURE_2D, 0);
    }
}

fn set_framebuffer_texture(fbo: u32, texture: u32) {
    unsafe {
        gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
        gl::FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            texture,
            0,
        );
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
}
