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

extern crate gl_generator;

use gl_generator::*;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::*;

fn main() {
    let dest = env::var("OUT_DIR").unwrap();
    let mut file = File::create(Path::new(&dest).join("test_no_warnings.rs")).unwrap();

    // Gl

    let gl_registry = Registry::new(Api::Gl, (4, 6), Profile::Core, Fallbacks::All, []);

    writeln!(&mut file, "mod gl_global {{").unwrap();
    gl_registry
        .write_bindings(GlobalGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    writeln!(&mut file, "mod gl_static {{").unwrap();
    gl_registry
        .write_bindings(StaticGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    writeln!(&mut file, "mod gl_struct {{").unwrap();
    gl_registry
        .write_bindings(StructGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    writeln!(&mut file, "mod gl_static_struct {{").unwrap();
    gl_registry
        .write_bindings(StaticStructGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    writeln!(&mut file, "mod gl_debug_struct {{").unwrap();
    gl_registry
        .write_bindings(DebugStructGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    // Glx

    let glx_registry = Registry::new(Api::Glx, (1, 4), Profile::Core, Fallbacks::All, []);

    writeln!(&mut file, "mod glx_global {{").unwrap();
    glx_registry
        .write_bindings(GlobalGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    writeln!(&mut file, "mod glx_static {{").unwrap();
    glx_registry
        .write_bindings(StaticGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    writeln!(&mut file, "mod glx_struct {{").unwrap();
    glx_registry
        .write_bindings(StructGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    writeln!(&mut file, "mod glx_static_struct {{").unwrap();
    glx_registry
        .write_bindings(StaticStructGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    writeln!(&mut file, "mod glx_debug_struct {{").unwrap();
    glx_registry
        .write_bindings(DebugStructGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    // Wgl

    let wgl_registry = Registry::new(Api::Wgl, (1, 0), Profile::Core, Fallbacks::All, []);

    writeln!(&mut file, "mod wgl_global {{").unwrap();
    wgl_registry
        .write_bindings(GlobalGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    writeln!(&mut file, "mod wgl_static {{").unwrap();
    wgl_registry
        .write_bindings(StaticGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    writeln!(&mut file, "mod wgl_struct {{").unwrap();
    wgl_registry
        .write_bindings(StructGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    writeln!(&mut file, "mod wgl_static_struct {{").unwrap();
    wgl_registry
        .write_bindings(StaticStructGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    writeln!(&mut file, "mod wgl_debug_struct {{").unwrap();
    wgl_registry
        .write_bindings(DebugStructGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    // Gles1

    let gles1_registry = Registry::new(Api::Gles1, (1, 1), Profile::Core, Fallbacks::All, []);

    writeln!(&mut file, "mod gles1_global {{").unwrap();
    gles1_registry
        .write_bindings(GlobalGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    writeln!(&mut file, "mod gles1_static {{").unwrap();
    gles1_registry
        .write_bindings(StaticGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    writeln!(&mut file, "mod gles1_struct {{").unwrap();
    gles1_registry
        .write_bindings(StructGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    writeln!(&mut file, "mod gles1_static_struct {{").unwrap();
    gles1_registry
        .write_bindings(StaticStructGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    writeln!(&mut file, "mod gles1_debug_struct {{").unwrap();
    gles1_registry
        .write_bindings(DebugStructGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    // Gles2

    let gles2_registry = Registry::new(Api::Gles2, (3, 1), Profile::Core, Fallbacks::All, []);

    writeln!(&mut file, "mod gles2_global {{").unwrap();
    gles2_registry
        .write_bindings(GlobalGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    writeln!(&mut file, "mod gles2_static {{").unwrap();
    gles2_registry
        .write_bindings(StaticGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    writeln!(&mut file, "mod gles2_struct {{").unwrap();
    gles2_registry
        .write_bindings(StructGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    writeln!(&mut file, "mod gles2_static_struct {{").unwrap();
    gles2_registry
        .write_bindings(StaticStructGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    writeln!(&mut file, "mod gles2_debug_struct {{").unwrap();
    gles2_registry
        .write_bindings(DebugStructGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    // Egl

    let egl_registry = Registry::new(Api::Egl, (1, 5), Profile::Core, Fallbacks::All, []);

    writeln!(&mut file, "mod egl_global {{ {}", build_egl_symbols()).unwrap();
    egl_registry
        .write_bindings(GlobalGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    writeln!(&mut file, "mod egl_static {{ {}", build_egl_symbols()).unwrap();
    egl_registry
        .write_bindings(StaticGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    writeln!(&mut file, "mod egl_struct {{ {}", build_egl_symbols()).unwrap();
    egl_registry
        .write_bindings(StructGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    writeln!(
        &mut file,
        "mod egl_static_struct {{ {}",
        build_egl_symbols()
    )
    .unwrap();
    egl_registry
        .write_bindings(StaticStructGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();

    writeln!(&mut file, "mod egl_debug_struct {{ {}", build_egl_symbols()).unwrap();
    egl_registry
        .write_bindings(DebugStructGenerator, &mut file)
        .unwrap();
    writeln!(&mut file, "}}").unwrap();
}

fn build_egl_symbols() -> &'static str {
    "
        #![allow(non_camel_case_types)]

        use std::os::raw;

        pub type khronos_utime_nanoseconds_t = raw::c_int;
        pub type khronos_uint64_t = u64;
        pub type khronos_ssize_t = isize;
        pub type EGLNativeDisplayType = *const raw::c_void;
        pub type EGLNativePixmapType = *const raw::c_void;
        pub type EGLNativeWindowType = *const raw::c_void;
        pub type EGLint = raw::c_int;
        pub type NativeDisplayType = *const raw::c_void;
        pub type NativePixmapType = *const raw::c_void;
        pub type NativeWindowType = *const raw::c_void;
    "
}
