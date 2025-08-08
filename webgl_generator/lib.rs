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

//! A WebGL bindings generator. It defines a function named `Registry::write_bindings` which can be
//! used to generate all constants and functions of a given WebGL version.
//!
//! See the `webgl` crate for an example of use.
extern crate heck;
extern crate html2text;
extern crate khronos_api;
extern crate regex;
extern crate webidl;
extern crate xml;

mod utils;
mod webgl_generators;
mod webgl_registry;

pub use webgl_generators::stdweb_gen::StdwebGenerator;
pub use webgl_generators::Generator;

pub use webgl_registry::*;
