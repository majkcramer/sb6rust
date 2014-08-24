#![feature(globs)]

extern crate gl;
extern crate native;
extern crate sb6;

use gl::types::*;
use std::mem;
use std::ptr;
use vmath::Mat4;

mod vmath;

static vertex_positions: [GLfloat, ..108] = [
    -0.25,  0.25, -0.25,
    -0.25, -0.25, -0.25,
     0.25, -0.25, -0.25,

     0.25, -0.25, -0.25,
     0.25,  0.25, -0.25,
    -0.25,  0.25, -0.25,

     0.25, -0.25, -0.25,
     0.25, -0.25,  0.25,
     0.25,  0.25, -0.25,

     0.25, -0.25,  0.25,
     0.25,  0.25,  0.25,
     0.25,  0.25, -0.25,

     0.25, -0.25,  0.25,
    -0.25, -0.25,  0.25,
     0.25,  0.25,  0.25,

    -0.25, -0.25,  0.25,
    -0.25,  0.25,  0.25,
     0.25,  0.25,  0.25,

    -0.25, -0.25,  0.25,
    -0.25, -0.25, -0.25,
    -0.25,  0.25,  0.25,

    -0.25, -0.25, -0.25,
    -0.25,  0.25, -0.25,
    -0.25,  0.25,  0.25,

    -0.25, -0.25,  0.25,
     0.25, -0.25,  0.25,
     0.25, -0.25, -0.25,

     0.25, -0.25, -0.25,
    -0.25, -0.25, -0.25,
    -0.25, -0.25,  0.25,

    -0.25,  0.25, -0.25,
     0.25,  0.25, -0.25,
     0.25,  0.25,  0.25,

     0.25,  0.25,  0.25,
    -0.25,  0.25,  0.25,
    -0.25,  0.25, -0.25
];

static VS_SRC: &'static str = "\
#version 330 core                                                  \n
                                                                   \n
in vec4 position;                                                  \n
                                                                   \n
out VS_OUT                                                         \n
{                                                                  \n
    vec4 color;                                                    \n
} vs_out;                                                          \n
                                                                   \n
uniform mat4 mv_matrix;                                            \n
uniform mat4 proj_matrix;                                          \n
                                                                   \n
void main(void)                                                    \n
{                                                                  \n
    gl_Position = proj_matrix * mv_matrix * position;              \n
    vs_out.color = position * 2.0 + vec4(0.5, 0.5, 0.5, 0.0);      \n
}                                                                  \n
";

static FS_SRC: &'static str = "\
#version 330 core                                                  \n\
                                                                   \n\
out vec4 color;                                                    \n\
                                                                   \n\
in VS_OUT                                                          \n\
{                                                                  \n\
    vec4 color;                                                    \n\
} fs_in;                                                           \n\
                                                                   \n\
void main(void)                                                    \n\
{                                                                  \n\
    color = fs_in.color;                                           \n\
}                                                                  \n\
";

struct MyApp {
    info: sb6::AppInfo,
    program: GLuint,
    vao: GLuint,
    buffer: GLuint,
    mv_location: GLint,
    proj_location: GLint,
    aspect: f32,
    proj_matrix: Mat4
}

impl MyApp {
    fn new(init: sb6::AppInfo) -> MyApp {
        MyApp {
            info: init,
            program: 0,
            vao: 0,
            buffer: 0,
            mv_location: -1,
            proj_location: -1,
            aspect: init.windowWidth as f32 / init.windowHeight as f32,
            proj_matrix: Mat4::identity()
        }
    }
}

impl sb6::App for MyApp {
    fn get_app_info(&self) -> &sb6::AppInfo { &self.info }
    fn startup(&mut self) {
        unsafe {
            self.program = gl::CreateProgram();

            let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
            FS_SRC.with_c_str(
                |ptr| gl::ShaderSource(fs, 1, &ptr, ptr::null()));
            gl::CompileShader(fs);
            sb6::check_compile_status(fs);

            let vs = gl::CreateShader(gl::VERTEX_SHADER);
            VS_SRC.with_c_str(
                |ptr| gl::ShaderSource(vs, 1, &ptr, ptr::null()));
            gl::CompileShader(vs);
            sb6::check_compile_status(vs);

            gl::AttachShader(self.program, vs);
            gl::AttachShader(self.program, fs);
            gl::LinkProgram(self.program);
            sb6::check_link_status(self.program);

            gl::DeleteShader(vs);
            gl::DeleteShader(fs);

            self.mv_location = "mv_matrix".with_c_str(
                |ptr| gl::GetUniformLocation(self.program, ptr));
            self.proj_location = "proj_matrix".with_c_str(
                |ptr| gl::GetUniformLocation(self.program, ptr));

            gl::GenVertexArrays(1, &mut self.vao);
            gl::BindVertexArray(self.vao);

            gl::GenBuffers(1, &mut self.buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.buffer);
            gl::BufferData(gl::ARRAY_BUFFER,
                           mem::size_of_val(&vertex_positions) as GLsizeiptr,
                           mem::transmute(vertex_positions.as_ptr()),
                           gl::STATIC_DRAW);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
            gl::EnableVertexAttribArray(0);

            gl::Enable(gl::CULL_FACE);
            gl::FrontFace(gl::CW);

            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LEQUAL);
        }
        self.on_resize(800, 600);
    }

    fn shutdown(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.buffer);
            gl::DeleteProgram(self.program);
        }
        self.mv_location = -1;
        self.proj_location = -1;
        self.buffer = 0;
        self.vao = 0;
        self.program = 0;
    }

    fn on_resize(&mut self, width: int, height: int) {
        self.aspect = width as f32 / height as f32;
        self.proj_matrix = Mat4::perspective(50.0, self.aspect, 0.1, 1000.0);
    }

    fn render(&self, time: f64) {
        static green: [GLfloat, ..4] = [ 0.0, 0.25, 0.0, 1.0 ];
        static one: GLfloat = 1.0;

        unsafe {
            gl::Viewport(0, 0, self.info.windowWidth as i32,
                         self.info.windowHeight as i32);

            gl::ClearBufferfv(gl::COLOR, 0, green.as_ptr());
            gl::ClearBufferfv(gl::DEPTH, 0, &one);

            gl::UseProgram(self.program);

            gl::UniformMatrix4fv(self.proj_location, 1, gl::FALSE,
                                 self.proj_matrix.as_ptr());

            let f = time as f32 * 0.3;
            let mv_matrix =
                Mat4::translate(0.0, 0.0, -4.0) *
                Mat4::translate((2.1 * f).sin() * 0.5,
                (1.7 * f).cos() * 0.5,
                (1.3 * f).sin() * (1.5 * f).cos() * 2.0) *
                Mat4::rotate(time as f32 * 45.0, 0.0, 1.0, 0.0) *
                Mat4::rotate(time as f32 * 81.0, 1.0, 0.0, 0.0);
            gl::UniformMatrix4fv(self.mv_location, 1, gl::FALSE,
                                 mv_matrix.as_ptr());

            gl::DrawArrays(gl::TRIANGLES, 0, 36);
        }
    }
}

fn main() {
    let mut init = sb6::AppInfo::default();
    init.title = "OpenGL SuperBible - Moving Triangle";
    let mut app = MyApp::new(init);
    sb6::run(&mut app);
}

#[start]
fn start(argc: int, argv: *const *const u8) -> int {
    native::start(argc, argv, main)
}
