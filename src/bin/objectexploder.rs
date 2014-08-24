#![feature(globs)]
#![feature(macro_rules)]

extern crate gl;
extern crate native;
extern crate sb6;

use gl::types::*;
use std::ptr;
use sb6::{ AppInfo, App, check_compile_status, check_link_status };
use sb6::{ Object };
use vmath::Mat4;

mod vmath;

static VS_SRC: &'static str = "\
#version 410 core                                                  \n\
                                                                   \n\
layout (location = 0) in vec4 position;                            \n\
layout (location = 1) in vec3 normal;                              \n\
                                                                   \n\
out VS_OUT                                                         \n\
{                                                                  \n\
    vec3 normal;                                                   \n\
    vec4 color;                                                    \n\
} vs_out;                                                          \n\
                                                                   \n\
uniform mat4 mv_matrix;                                            \n\
uniform mat4 proj_matrix;                                          \n\
                                                                   \n\
void main(void)                                                    \n\
{                                                                  \n\
    gl_Position = proj_matrix * mv_matrix * position;              \n\
    vs_out.color = position * 2.0 + vec4(0.5, 0.5, 0.5, 0.0);      \n\
    vs_out.normal = normalize(mat3(mv_matrix) * normal);           \n\
}                                                                  \n\
";

static GS_SRC: &'static str = "\
#version 410 core                                                      \n\
                                                                       \n\
layout (triangles) in;                                                 \n\
layout (triangle_strip, max_vertices = 3) out;                         \n\
                                                                       \n\
in VS_OUT                                                              \n\
{                                                                      \n\
    vec3 normal;                                                       \n\
    vec4 color;                                                        \n\
} gs_in[];                                                             \n\
                                                                       \n\
out GS_OUT                                                             \n\
{                                                                      \n\
    vec3 normal;                                                       \n\
    vec4 color;                                                        \n\
} gs_out;                                                              \n\
                                                                       \n\
uniform float explode_factor = 0.2;                                    \n\
                                                                       \n\
void main(void)                                                        \n\
{                                                                      \n\
    vec3 ab = gl_in[1].gl_Position.xyz - gl_in[0].gl_Position.xyz;     \n\
    vec3 ac = gl_in[2].gl_Position.xyz - gl_in[0].gl_Position.xyz;     \n\
    vec3 face_normal = -normalize(cross(ab, ac));                      \n\
    for (int i = 0; i < gl_in.length(); i++)                           \n\
    {                                                                  \n\
        gl_Position = gl_in[i].gl_Position + vec4(face_normal * explode_factor, 0.0);    \n\
        gs_out.normal = gs_in[i].normal;                               \n\
        gs_out.color = gs_in[i].color;                                 \n\
        EmitVertex();                                                  \n\
    }                                                                  \n\
    EndPrimitive();                                                    \n\
}                                                                      \n\
";

static FS_SRC: &'static str = "\
#version 410 core                                                  \n\
                                                                   \n\
out vec4 color;                                                    \n\
                                                                   \n\
in GS_OUT                                                          \n\
{                                                                  \n\
    vec3 normal;                                                   \n\
    vec4 color;                                                    \n\
} fs_in;                                                           \n\
                                                                   \n\
void main(void)                                                    \n\
{                                                                  \n\
    color = vec4(1.0) * abs(normalize(fs_in.normal).z);            \n\
}                                                                  \n\
";

struct MyApp {
    info: sb6::AppInfo,
    program: GLuint,
    mv_location: GLint,
    proj_location: GLint,
    explode_factor_location: GLint,
    object: Object,
}

impl MyApp {
    fn new(init: sb6::AppInfo) -> MyApp {
        MyApp {
            info: init,
            program: 0,
            mv_location: -1,
            proj_location: -1,
            explode_factor_location: -1,
            object: Object::new()
        }
    }
}

impl sb6::App for MyApp {
    fn get_app_info(&self) -> &sb6::AppInfo { &self.info }

    fn startup(&mut self) {
        self.program = gl::CreateProgram();

        let vs = gl::CreateShader(gl::VERTEX_SHADER);
        let gs = gl::CreateShader(gl::GEOMETRY_SHADER);
        let fs = gl::CreateShader(gl::FRAGMENT_SHADER);

        unsafe {
            VS_SRC.with_c_str(
                |ptr| gl::ShaderSource(vs, 1, &ptr, ptr::null()));
            GS_SRC.with_c_str(
                |ptr| gl::ShaderSource(gs, 1, &ptr, ptr::null()));
            FS_SRC.with_c_str(
                |ptr| gl::ShaderSource(fs, 1, &ptr, ptr::null()));
        }

        gl::CompileShader(vs);
        gl::CompileShader(gs);
        gl::CompileShader(fs);

        sb6::check_compile_status(vs);
        sb6::check_compile_status(gs);
        sb6::check_compile_status(fs);

        gl::AttachShader(self.program, vs);
        gl::AttachShader(self.program, gs);
        gl::AttachShader(self.program, fs);
        gl::LinkProgram(self.program);
        sb6::check_link_status(self.program);

        gl::DeleteShader(vs);
        gl::DeleteShader(gs);
        gl::DeleteShader(fs);
        
        unsafe {
            self.mv_location = "mv_matrix".with_c_str(
                |ptr| gl::GetUniformLocation(self.program, ptr));
            self.proj_location = "proj_matrix".with_c_str(
                |ptr| gl::GetUniformLocation(self.program, ptr));
            self.proj_location = "explode_factor".with_c_str(
                |ptr| gl::GetUniformLocation(self.program, ptr));
        }

        match self.object.load("media/objects/bunny_1k.sbm") {
            Ok(_) => (),
            e => fail!("failed to load sbm file: {}", e)
        }

        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LEQUAL);
    }

    fn shutdown(&mut self) {
        self.object.free();
        gl::DeleteProgram(self.program);
        self.mv_location = -1;
        self.proj_location = -1;
        self.explode_factor_location = -1;
        self.program = 0;
    }

    fn render(&self, time: f64) {
        static black: [GLfloat, ..4] = [ 0.0, 0.0, 0.0, 1.0 ];
        static one: GLfloat = 1.0;
        let time = time as f32;

        let info = self.get_app_info();
        let aspect = info.windowWidth as f32 / info.windowWidth as f32;
        let proj_matrix =  Mat4::perspective(50.0, aspect, 0.1, 1000.0);
        let mv_matrix = Mat4::translate(0.0, 0.0, -3.0) *
            Mat4::rotate(time * 45.0, 0.0, 1.0, 0.0) *
            Mat4::rotate(time * 81.0, 1.0, 0.0, 0.0);
        let explode_factor = (time * 8.0).sin() * (time * 6.0).cos() * 0.7 + 0.1;

        unsafe {
            gl::Viewport(0, 0, info.windowWidth as i32,
                info.windowHeight as i32);
            gl::ClearBufferfv(gl::COLOR, 0, black.as_ptr());
            gl::ClearBufferfv(gl::DEPTH, 0, &one);

            gl::UseProgram(self.program);

            gl::UniformMatrix4fv(self.proj_location, 1, gl::FALSE,
                proj_matrix.as_ptr());

            gl::UniformMatrix4fv(self.mv_location, 1, gl::FALSE,
                mv_matrix.as_ptr());

            gl::Uniform1f(self.explode_factor_location, explode_factor);

            self.object.render();
        }
    }
}

fn main() {
    let mut init = sb6::AppInfo::default();
    init.title = "OpenGL SuperBible - Exploder";
    let mut app = MyApp::new(init);
    sb6::run(&mut app);
}

#[start]
fn start(argc: int, argv: *const *const u8) -> int {
    native::start(argc, argv, main)
}

