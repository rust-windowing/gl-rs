# gl-rs

[![Version](https://img.shields.io/crates/v/gl.svg)](https://crates.io/crates/gl)
[![License](https://img.shields.io/crates/l/gl.svg)](https://github.com/brendanzab/gl-rs/blob/master/LICENSE)
[![Downloads](https://img.shields.io/crates/d/gl.svg)](https://crates.io/crates/gl)

An no_std OpenGL function pointer loader for the Rust Programming Language.

```toml
[dependencies]
gl = "0.14.0"
```

## Basic usage

You can import the pointer style loader and type aliases like so:

```rust
extern crate gl;
// include the OpenGL type aliases
use gl::types::*;
```

You must load the function pointers into their respective function pointers
using the `load_with` function. You must supply a loader function from your
context library. This is how it would look using [glfw-rs](https://github.com/PistonDevelopers/glfw-rs):

```rust
// the supplied function must be of the type:
// `&fn(symbol: &'static str) -> *const std::os::raw::c_void`
// `window` is a glfw::Window
gl::load_with(|s| window.get_proc_address(s) as *const _);

// loading a specific function pointer
gl::Viewport::load_with(|s| window.get_proc_address(s) as *const _);
```

Calling a function that has not been loaded will result in a failure like:
`panic!("gl::Viewport was not loaded")`, which avoids a segfault. This feature
does not cause any run time overhead because the failing functions are
assigned only when `load_with` is called.

All OpenGL function calls are `unsafe`.

```rust
// accessing an enum
gl::TEXTURE_2D;

// calling a function
unsafe { gl::DrawArrays(gl::TRIANGLES, 0, 3) };
```

Each function pointer has an associated boolean value allowing you to
check if a function has been loaded at run time. The function accesses a
corresponding global boolean that is set when `load_with` is called, so there
shouldn't be much overhead.

```rust
if gl::Viewport::is_loaded() {
    // do something...
}
```

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
