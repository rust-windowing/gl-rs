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

use std::os::raw;

pub mod gl {
    #![allow(
        clippy::missing_safety_doc,
        clippy::missing_transmute_annotations,
        clippy::too_many_arguments,
        clippy::unused_unit
    )]

    include!(concat!(env!("OUT_DIR"), "/test_symbols.rs"));
}

pub fn compile_test_symbols_exist() {
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
    }
}

#[test]
fn test_fallback_works() {
    fn loader(name: &str) -> *const raw::c_void {
        match name {
            "glGenFramebuffers" => std::ptr::null::<raw::c_void>(),
            "glGenFramebuffersEXT" => 42 as *const raw::c_void,
            name => panic!("test tried to load {name} unexpectedly!"),
        }
    }

    gl::GenFramebuffers::load_with(loader);
    assert!(gl::GenFramebuffers::is_loaded());
}
