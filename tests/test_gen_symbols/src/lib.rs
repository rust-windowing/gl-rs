// Copyright 2015 Brendan Zabarauskas and the gl-rs developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(
    clippy::missing_transmute_annotations,
    clippy::too_many_arguments,
    clippy::unused_unit,
    clippy::upper_case_acronyms
)]

use std::os::raw;

include!(concat!(env!("OUT_DIR"), "/test_gen_symbols.rs"));

pub fn compile_test_gl() {
    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT);
        let _: raw::c_uint = gl::CreateProgram();
        gl::CompileShader(5);
        gl::GetActiveUniformBlockiv(
            0,
            0,
            gl::UNIFORM_BLOCK_REFERENCED_BY_GEOMETRY_SHADER,
            std::ptr::null_mut(),
        );

        let _: *mut raw::c_void = gl::MapBuffer(0, 0);
    }
}

pub fn compile_test_gles1() {
    unsafe {
        gles1::Clear(gles1::COLOR_BUFFER_BIT);
    }
}

pub fn compile_test_gles2() {
    unsafe {
        gles2::Clear(gles2::COLOR_BUFFER_BIT);
        let _: raw::c_uint = gles2::CreateProgram();
        gles2::CompileShader(5);
    }
}

pub fn compile_test_glsc2() {
    unsafe {
        glsc2::Clear(glsc2::COLOR_BUFFER_BIT);
    }
}

pub fn compile_test_glx() {
    unsafe {
        let _ = glx::GetProcAddress(std::ptr::null());
        glx::SwapBuffers(std::ptr::null_mut(), 0);
    }
}

pub fn compile_test_wgl() {
    unsafe {
        let _: wgl::types::HGLRC = wgl::CreateContext(std::ptr::null());
    }
}

pub fn compile_test_egl() {
    unsafe {
        let _ = [
            egl::SURFACE_TYPE,
            egl::WINDOW_BIT,
            egl::BLUE_SIZE,
            8,
            egl::GREEN_SIZE,
            8,
            egl::RED_SIZE,
            8,
            egl::NONE,
        ];

        let _ = egl::GetDisplay(egl::DEFAULT_DISPLAY);
        egl::Terminate(std::ptr::null());
    }
}
