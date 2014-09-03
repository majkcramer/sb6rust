/*
 * Copyright © 2012-2013 Graham Sellers
 *
 * Permission is hereby granted, free of charge, to any person obtaining a
 * copy of this software and associated documentation files (the "Software"),
 * to deal in the Software without restriction, including without limitation
 * the rights to use, copy, modify, merge, publish, distribute, sublicense,
 * and/or sell copies of the Software, and to permit persons to whom the
 * Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice (including the next
 * paragraph) shall be included in all copies or substantial portions of the
 * Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL
 * THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
 * FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
 * DEALINGS IN THE SOFTWARE.
 */

extern crate gl;
extern crate glfw;

use gl::types::*;
use glfw::Context;
use std::ptr;
use std::str;

pub struct AppInfo {
    pub title: &'static str,
    pub window_width: u32,
    pub window_height: u32,
    pub major_version: u32,
    pub minor_version: u32,
    pub samples: uint,
    pub fullscreen: bool,
    pub vsync: bool,
    pub cursor: bool,
    pub stereo: bool,
    pub debug: bool
}

impl AppInfo {
    #[cfg(use_gl_3_3)]
    fn version() -> (u32, u32) { (3, 3) }
    #[cfg(not(use_gl_3_3))]
    fn version() -> (u32, u32) { (4, 4) }
    pub fn default() -> AppInfo {
        let (major, minor) = AppInfo::version();
        AppInfo {
        title: "SuperBible6 Example",
        window_width: 800,
        window_height: 600,
        major_version: major,
        minor_version: minor,
        samples: 0,
        fullscreen: false,
        vsync: false,
        cursor: true,
        stereo: false,
        debug: false
        }
    }
}

pub fn check_compile_status(shader: GLuint) {
        unsafe {
            // Get the compile status
            let mut status = gl::FALSE as GLint;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

            // Fail on error
            if status != (gl::TRUE as GLint) {
                let mut len = 0;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
                // subtract 1 to skip the trailing null character
                let mut buf = Vec::from_elem(len as uint - 1, 0u8);
                gl::GetShaderInfoLog(shader, len, ptr::mut_null(),
                    buf.as_mut_ptr() as *mut GLchar);
                fail!("{}", str::from_utf8(buf.as_slice()).expect(
                        "ShaderInfoLog not valid utf8"));
            }
        }
}

pub fn check_link_status(program: GLuint) {
    unsafe {
        // Get the link status
        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len: GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            // subtract 1 to skip the trailing null character
            let mut buf = Vec::from_elem(len as uint - 1, 0u8);
            gl::GetProgramInfoLog(program, len, ptr::mut_null(),
                buf.as_mut_ptr() as *mut GLchar);
            fail!("{}", str::from_utf8(buf.as_slice()).expect(
                    "ProgramInfoLog not valid utf8"));
        }
    }
}

pub trait App
{
    fn get_app_info(&self) -> &AppInfo;
    fn startup(&mut self);
    fn render(&self, time: f64);
    fn shutdown(&mut self);
    fn on_resize(&mut self, _: int, _: int) {}
}

fn handle_window_event<T: App>(app: &mut T, window: &glfw::Window,
                               event: glfw::WindowEvent) {
    match event {
        glfw::KeyEvent(glfw::KeyEscape, _, glfw::Press, _) => {
            window.set_should_close(true)
        }
        glfw::SizeEvent(w, h) => {
            app.on_resize(w as int, h as int)
        }
        _ => ()
    }
}

pub fn run<T: App>(app: &mut T) {
    let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    let (window, events) = {
        let info = app.get_app_info();
        glfw.window_hint(glfw::ContextVersion(
                info.major_version, info.minor_version));
        glfw.window_hint(glfw::OpenglProfile(glfw::OpenGlCoreProfile));
        glfw.window_hint(glfw::OpenglForwardCompat(true));
        glfw.create_window(
            info.window_width, info.window_height, info.title.as_slice(),
            glfw::Windowed).expect("Failed to create GLFW window.")
    };

    window.set_key_polling(true);
    window.set_size_polling(true);
    window.make_current();

    // Load the OpenGL function pointers
    gl::load_with(|s| glfw.get_proc_address(s));

    app.startup();

    while !window.should_close() {
        app.render(glfw.get_time());

        window.swap_buffers();

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event::<T>(app, &window, event);
        }
    }

    app.shutdown();
}
