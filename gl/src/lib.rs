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

//! # Usage
//!
//! You can import the pointer style loader and type aliases like so:
//!
//! ~~~no_run
//! # #![allow(unused_imports)]
//! extern crate gl;
//! # fn main() {
//! // include the OpenGL type aliases
//! use gl::types::*;
//! # }
//! ~~~
//!
//! You must load the function pointers into their respective function pointers
//! using the `load_with` function. You must supply a loader function from your
//! context library. This is how it would look using [glfw-rs](https://github.com/PistonDevelopers/glfw-rs):
//!
//! ~~~ignore
//! // the supplied function must be of the type:
//! // `&fn(symbol: &'static str) -> *const std::os::raw::c_void`
//! // `window` is a glfw::Window
//! gl::load_with(|s| window.get_proc_address(s) as *const _);
//!
//! // loading a specific function pointer
//! gl::Viewport::load_with(|s| window.get_proc_address(s) as *const _);
//! ~~~
//!
//! Calling a function that has not been loaded will result in a failure like:
//! `panic!("gl::Viewport was not loaded")`, which avoids a segfault. This feature
//! does not cause any run time overhead because the failing functions are
//! assigned only when `load_with` is called.
//!
//! All OpenGL function calls are `unsafe`.
//!
//! ~~~no_run
//! # #![allow(path_statement)]
//! # extern crate gl;
//! # fn main() {
//! // accessing an enum
//! gl::TEXTURE_2D;
//!
//! // calling a function
//! unsafe { gl::DrawArrays(gl::TRIANGLES, 0, 3) };
//! # }
//! ~~~
//!
//! Each function pointer has an associated boolean value allowing you to
//! check if a function has been loaded at run time. The function accesses a
//! corresponding global boolean that is set when `load_with` is called, so there
//! shouldn't be much overhead.
//!
//! ~~~no_run
//! if gl::Viewport::is_loaded() {
//!     // do something...
//! }
//! ~~~
//!

#![no_std]
#![crate_name = "gl"]
#![crate_type = "lib"]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
