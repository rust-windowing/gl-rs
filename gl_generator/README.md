# gl_generator

[![Version](https://img.shields.io/crates/v/gl_generator.svg)](https://crates.io/crates/gl_generator)
[![License](https://img.shields.io/crates/l/gl_generator.svg)](https://github.com/brendanzab/gl-rs/blob/master/LICENSE)
[![Downloads](https://img.shields.io/crates/d/gl_generator.svg)](https://crates.io/crates/gl_generator)

Code generators for creating no_std bindings to the Khronos OpenGL APIs.

## Usage

If you need a specific version of OpenGL, or you need a different API
(OpenGL ES, EGL, WGL, GLX), or if you need certain extensions, you should use
the `gl_generator` crate.

See [gfx_gl](https://github.com/gfx-rs/gfx_gl) for an example of using a
custom OpenGL loader for a project.

Add this to your `Cargo.toml`:

```toml
[build-dependencies]
gl_generator = "0.14.0"
```

Under the `[package]` section, add:

```toml
build = "build.rs"
```

Create a `build.rs` to pull your specific version/API:

```rust
extern crate gl_generator;

use gl_generator::{Api, Fallbacks, GlobalGenerator, Profile, Registry};
use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    let dest = env::var("OUT_DIR").unwrap();
    let mut file = File::create(&Path::new(&dest).join("bindings.rs")).unwrap();

    Registry::new(Api::Gl, (4, 6), Profile::Core, Fallbacks::All, [])
        .write_bindings(GlobalGenerator, &mut file)
        .unwrap();
}
```

Then use it like this:

```rust
mod gl {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

/// Simple loading example
fn main() {
    let window = ...;

    // Assuming `window` is a glfw::Window, made current
    gl::load_with(|s| window.get_proc_address(s) as *const _);
}
```

The `build.rs` file will generate all the OpenGL functions in a file named `bindings.rs` plus all enumerations and all types in the `types` submodule. Note that while the generated bindings are `no_std`, the generator itself is not.

## Symbol string types
In previous versions, if you needed a null-terminated c-string from the `gl::load_with` function, you had to convert the &str to a CString. This limited the use of this library in `no_std` environments.

In the latest release you can add the `"cstr_symbols"` feature to
your `Cargo.toml` and the symbol name will now be a null-terminated `&CStr`, so you won't have to convert between them anymore.

```rust
// Quick example with sdl3-sys

// Without the feature
gl::load_with(| procname: &str | {
    match CString::new(procname) { // Allocates and requires std
        Ok(procname) => unsafe {
           SDL_GL_GetProcAddress(procname.as_ptr()) as *const _
        },
        Err(_) => ptr::null(),
    }
});

// With the feature
gl::load_with(| procname: &CStr | {
    unsafe { SDL_GL_GetProcAddress(procname.as_ptr()) as *const _ }
});
```

## Generator types

### Global generator

The global generator is the one used by default by the `gl` crate. See the 
[README](https://github.com/brendanzab/gl-rs/tree/master/gl) for more details.

### Struct generator

The struct generator is a cleaner alternative to the global generator.

The main difference is that you must call `gl::Gl::load_with` instead of
`gl::load_with`, and this functions returns a struct of type `Gl`. The OpenGL
functions are not static functions but member functions in this `Gl` struct.
This is important when the GL functions are associated with the current
context, as is true on Windows.

The enumerations and types are still static and available in a similar way as
in the global generator.

### Static generator

The static generator generates plain old bindings. You don't need to load the
functions.

This generator should only be used only if the platform you are compiling for
is guaranteed to support the requested API. Otherwise you will get a
compilation error.
For example, you can use it for WGL and OpenGL 1.1 on Windows or GLX and
OpenGL 1.3 on Linux, because Windows and Linux are guaranteed to provide
implementations for these APIs.

You will need to manually provide the linkage. For example to use WGL or
OpenGL 1.1 on Windows, you will need to add
`#[link="OpenGL32.lib"] extern {}` somewhere in your code.

### Custom Generators

The `gl_generator` can be extended with custom generators. This is a niche
feature useful only in very rare cases. To create a custom generator, implement
the `gl_generator::Generator` trait. See the source of the
`gl_generator::generators` module for examples.

Various utility functions are provided in the `generators` module, but the api
is unstable, so it has been placed behind a feature flag. In access these
functions, you will need to add the `"unstable_generator_utils"` feature to
your `Cargo.toml`:

```toml
[build-dependencies.gl_generator]
version = "0.14.0"
features = ["unstable_generator_utils"]
```

## Extra features

The global and struct generators will attempt to use fallbacks functions when
they are available. For example, if `glGenFramebuffers` cannot be loaded it will
also attempt to load `glGenFramebuffersEXT` as a fallback.
