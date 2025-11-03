# `khronos_api`

[![CI](https://github.com/rust-windowing/gl-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/rust-windowing/gl-rs/actions/workflows/ci.yml)
[![Version](https://img.shields.io/crates/v/khronos_api.svg?logo=rust)](https://crates.io/crates/khronos_api)
[![Docs](https://img.shields.io/docsrs/khronos_api.svg?logo=docsdotrs)](https://docs.rs/khronos_api)
[![License](https://img.shields.io/crates/l/khronos_api.svg)](./LICENSE)
[![Downloads](https://img.shields.io/crates/d/khronos_api.svg)](https://crates.io/crates/khronos_api)

The Khronos XML API Registry, exposed as byte string constants.

```toml
[build-dependencies]
khronos_api = "3.2.0"
```

The following constants are provided:

- `GL_XML`: the contents of [`gl.xml`](https://github.com/KhronosGroup/OpenGL-Registry/blob/master/xml/gl.xml)
- `EGL_XML`: the contents of [`egl.xml`](https://github.com/KhronosGroup/EGL-Registry/blob/master/api/egl.xml)
- `WGL_XML`: the contents of [`wgl.xml`](https://github.com/KhronosGroup/OpenGL-Registry/blob/master/xml/wgl.xml)
- `GLX_XML`: the contents of [`glx.xml`](https://github.com/KhronosGroup/OpenGL-Registry/blob/master/xml/glx.xml)
- `GL_ANGLE_EXT_XML`: the contents of [`gl_angle_ext.xml`](https://github.com/google/angle/blob/master/scripts/gl_angle_ext.xml)
- `EGL_ANGLE_EXT_XML`: the contents of [`egl_angle_ext.xml`](https://github.com/google/angle/blob/master/scripts/egl_angle_ext.xml)
- `WEBGL_IDL`: the contents of [`webgl.idl`](https://github.com/KhronosGroup/WebGL/blob/master/specs/latest/1.0/webgl.idl)
- `WEBGL2_IDL`: the contents of [`webgl2.idl`](https://github.com/KhronosGroup/WebGL/blob/master/specs/latest/2.0/webgl2.idl)
- `WEBGL_EXT_XML`: the contents of the WebGL extension XML files, discovered by a build script
