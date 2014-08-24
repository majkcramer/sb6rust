#![feature(globs)]

extern crate gl;
extern crate native;
extern crate sb6;

use gl::types::*;
use std::ptr;

#[cfg(not(screen_space_color))]
static VS_SRC: &'static str = "\
#version 420 core                                                          \n\
                                                                           \n\
void main(void)                                                            \n\
{                                                                          \n\
    const vec4 vertices[] = vec4[](vec4( 0.25, -0.25, 0.5, 1.0),           \n\
                                   vec4(-0.25, -0.25, 0.5, 1.0),           \n\
                                   vec4( 0.25,  0.25, 0.5, 1.0));          \n\
                                                                           \n\
    gl_Position = vertices[gl_VertexID];                                   \n\
}                                                                          \n\
";

#[cfg(not(screen_space_color))]
static FS_SRC: &'static str = "\
#version 420 core                                                          \n\
                                                                           \n\
out vec4 color;                                                            \n\
                                                                           \n\
void main(void)                                                            \n\
{                                                                          \n\
    color = vec4(sin(gl_FragCoord.x * 0.25) * 0.5 + 0.5,                   \n\
                 cos(gl_FragCoord.y * 0.25) * 0.5 + 0.5,                   \n\
                 sin(gl_FragCoord.x * 0.15) * cos(gl_FragCoord.y * 0.1),   \n\
                 1.0);                                                     \n\
}                                                                          \n\
";

#[cfg(screen_space_color)]
static VS_SRC: &'static str = "\
#version 420 core                                                          \n\
                                                                           \n\
out vec4 vs_color;                                                         \n\
void main(void)                                                            \n\
{                                                                          \n\
    const vec4 vertices[] = vec4[](vec4( 0.25, -0.25, 0.5, 1.0),           \n\
                                   vec4(-0.25, -0.25, 0.5, 1.0),           \n\
                                   vec4( 0.25,  0.25, 0.5, 1.0));          \n\
    const vec4 colors[] = vec4[](vec4(1.0, 0.0, 0.0, 1.0),                 \n\
                                 vec4(0.0, 1.0, 0.0, 1.0),                 \n\
                                 vec4(0.0, 0.0, 1.0, 1.0));                \n\
                                                                           \n\
    gl_Position = vertices[gl_VertexID];                                   \n\
    vs_color = colors[gl_VertexID];                                        \n\
}                                                                          \n\
";

#[cfg(screen_space_color)]
static FS_SRC: &'static str = "\
#version 420 core                                                          \n\
                                                                           \n\
in vec4 vs_color;                                                          \n\
out vec4 color;                                                            \n\
                                                                           \n\
void main(void)                                                            \n\
{                                                                          \n\
    color = vs_color;                                                      \n\
}                                                                          \n\
";

struct MyApp {
    info: sb6::AppInfo,
    program: GLuint,
    vao: GLuint
}

impl MyApp {
    fn new(init: sb6::AppInfo) -> MyApp {
        MyApp { info: init, program: 0, vao: 0 }
    }
}

impl sb6::App for MyApp {
    fn get_app_info(&self) -> &sb6::AppInfo { &self.info }

    fn startup(&mut self) {
        self.program = gl::CreateProgram();

        let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
        unsafe {
            FS_SRC.with_c_str(
                |ptr| gl::ShaderSource(fs, 1, &ptr, ptr::null()));
            gl::CompileShader(fs);
        }
        sb6::check_compile_status(fs);

        let vs = gl::CreateShader(gl::VERTEX_SHADER);
        unsafe {
            VS_SRC.with_c_str(
                |ptr| gl::ShaderSource(vs, 1, &ptr, ptr::null()));
            gl::CompileShader(vs);
        }
        sb6::check_compile_status(vs);

        gl::AttachShader(self.program, vs);
        gl::AttachShader(self.program, fs);
        gl::LinkProgram(self.program);
        sb6::check_link_status(self.program);

        gl::DeleteShader(vs);
        gl::DeleteShader(fs);
        
        unsafe {
            gl::GenVertexArrays(1, &mut self.vao);
        }
        gl::BindVertexArray(self.vao);
    }

    fn shutdown(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
        }
        gl::DeleteProgram(self.program);
        self.vao = 0;
        self.program = 0;
    }

    fn render(&self, _: f64) {
        static green: [GLfloat, ..4] = [ 0.0, 0.25, 0.0, 1.0 ];
        unsafe {
            gl::ClearBufferfv(gl::COLOR, 0, green.as_ptr());
        }

        gl::UseProgram(self.program);
        gl::DrawArrays(gl::TRIANGLES, 0, 3);
    }
}

fn main() {
    let mut init = sb6::AppInfo::default();
    init.title = "OpenGL SuperBible - Simple Triangle";
    let mut app = MyApp::new(init);
    sb6::run(&mut app);
}

#[start]
fn start(argc: int, argv: *const *const u8) -> int {
    native::start(argc, argv, main)
}
