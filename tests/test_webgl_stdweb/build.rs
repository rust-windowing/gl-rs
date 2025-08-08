// Copyright 2016 Brendan Zabarauskas and the gl-rs developers
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

extern crate webgl_generator;

use std::env;
use std::fs::File;
use std::path::*;
use webgl_generator::*;

fn main() {
    let dest = env::var("OUT_DIR").unwrap();
    let mut file1 = File::create(Path::new(&dest).join("test_webgl_stdweb.rs")).unwrap();
    let mut file2 = File::create(Path::new(&dest).join("test_webgl2_stdweb.rs")).unwrap();

    Registry::new(Api::WebGl, Exts::ALL)
        .write_bindings(StdwebGenerator, &mut file1)
        .unwrap();

    Registry::new(Api::WebGl2, Exts::ALL)
        .write_bindings(StdwebGenerator, &mut file2)
        .unwrap();
}
