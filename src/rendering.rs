use gl::types::*;
use std::ffi::CString;

const VERTEX_SHADER_SRC: &str = include_str!("shader.vert");
const FRAGMENT_SHADER_SRC: &str = include_str!("shader.frag");

pub struct Renderer {
    vertex_shader: u32,
    fragment_shader: u32,
    shader_program: u32,
    vertex_array_objects: [u32; 2],
}

impl Renderer {
    pub fn new() -> Self {
        let vertex_shader = compile_shader(VERTEX_SHADER_SRC, gl::VERTEX_SHADER);
        let fragment_shader = compile_shader(FRAGMENT_SHADER_SRC, gl::FRAGMENT_SHADER);
        let shader_program = link_program(vertex_shader, fragment_shader);
        let mut vertex_array_objects = [u32::default(); 2];

        unsafe {
            gl::GenVertexArrays(
                vertex_array_objects.len() as i32,
                vertex_array_objects.as_mut_ptr(),
            );
        }

        Renderer {
            vertex_shader,
            fragment_shader,
            shader_program,
            vertex_array_objects,
        }
    }

    pub fn render(&self) {
        unsafe {
            gl::ClearColor(0.0, 0.5, 0.5, 1.0); // set background
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(self.shader_program);
            gl::BindVertexArray(self.vertex_array_objects[0]);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::BindVertexArray(self.vertex_array_objects[1]);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.shader_program);
            gl::DeleteShader(self.fragment_shader);
            gl::DeleteShader(self.vertex_shader);
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

pub fn setup_triangle_program(game_renderer: &mut Renderer) {
    unsafe {
        // Setup vertices for a triangle
        let triangle_1: [GLfloat; 3 * 3] = [
            0.5, 0.5, 0.0, // top right
            0.5, -0.5, 0.0, // bottom right
            -0.5, 0.5, 0.0, // top left
        ];
        let triangle_2: [GLfloat; 3 * 3] = [
            0.5, -0.5, 0.0, // bottom right
            -0.5, -0.5, 0.0, // bottom left
            -0.5, 0.5, 0.0, // top left
        ];

        create_vertex_buffer_object(
            game_renderer.shader_program,
            game_renderer.vertex_array_objects[0],
            &triangle_1,
            "pos",
        );

        create_vertex_buffer_object(
            game_renderer.shader_program,
            game_renderer.vertex_array_objects[1],
            &triangle_2,
            "pos",
        );

        // Setup fragment output
        let out_variable_name = CString::new("frag_color").unwrap();
        gl::BindFragDataLocation(game_renderer.shader_program, 0, out_variable_name.as_ptr());
    }
}

// TODO move into the Renderer struct
fn create_vertex_buffer_object(
    shader_program: u32,
    vertex_array: u32,
    vertices: &[GLfloat],
    attribute_name: &str,
) -> u32 {
    unsafe {
        gl::BindVertexArray(vertex_array);
        let mut vertex_buffer_object = u32::default();
        gl::GenBuffers(1, &mut vertex_buffer_object);
        gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer_object);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
            std::mem::transmute(&vertices[0]),
            gl::STATIC_DRAW,
        );
        let attribute_name = CString::new(attribute_name).unwrap();
        let pos_attr = gl::GetAttribLocation(shader_program, attribute_name.as_ptr()) as GLuint;
        gl::EnableVertexAttribArray(pos_attr);
        let should_normalize_floats = gl::FALSE as GLboolean;
        let attribute_size = 3;
        gl::VertexAttribPointer(
            pos_attr,
            attribute_size,
            gl::FLOAT,
            should_normalize_floats,
            0,
            std::ptr::null(),
        );

        vertex_buffer_object
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
