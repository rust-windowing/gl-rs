# gl-rs

[![Build Status](https://travis-ci.org/brendanzab/gl-rs.svg?branch=master)](https://travis-ci.org/brendanzab/gl-rs)

## Overview

This repository contains the necessary building blocks for OpenGL wrapper
libraries. For more information on each crate, see their respective READMEs
listed below.

The following crates are contained in this repository:

### gl

[![Version](https://img.shields.io/crates/v/gl.svg)](https://crates.io/crates/gl) [![License](https://img.shields.io/crates/l/gl.svg)](https://github.com/brendanzab/gl-rs/blob/master/LICENSE) [![Downloads](https://img.shields.io/crates/d/gl.svg)](https://crates.io/crates/gl)

[README](gl)

An no_std OpenGL function pointer loader for the Rust Programming Language.

```toml
[dependencies]
gl = "0.14.0"
```

### gl_generator

[![Version](https://img.shields.io/crates/v/gl_generator.svg)](https://crates.io/crates/gl_generator) [![License](https://img.shields.io/crates/l/gl_generator.svg)](https://github.com/brendanzab/gl-rs/blob/master/LICENSE) [![Downloads](https://img.shields.io/crates/d/gl_generator.svg)](https://crates.io/crates/gl_generator)

[README](gl_generator)

Code generators for creating no_std bindings to the Khronos OpenGL APIs.

```toml
[build-dependencies]
gl_generator = "0.14.0"
```

### khronos_api

[![Version](https://img.shields.io/crates/v/khronos_api.svg)](https://crates.io/crates/khronos_api) [![License](https://img.shields.io/crates/l/khronos_api.svg)](https://github.com/brendanzab/gl-rs/blob/master/LICENSE) [![Downloads](https://img.shields.io/crates/d/khronos_api.svg)](https://crates.io/crates/khronos_api)

[README](khronos_api)

The Khronos XML API Registry, exposed as byte string constants.

```toml
[build-dependencies]
khronos_api = "3.1.0"
```

#### Compiling from source

`khronos_api` makes use of git submodules. You will need to initialize these before building:

```sh
git submodule update --init
```

### webgl_generator

[![Version](https://img.shields.io/crates/v/webgl_generator.svg)](https://crates.io/crates/webgl_generator) [![License](https://img.shields.io/crates/l/webgl_generator.svg)](https://github.com/brendanzab/gl-rs/blob/master/LICENSE) [![Downloads](https://img.shields.io/crates/d/webgl_generator.svg)](https://crates.io/crates/webgl_generator)

[README](webgl_generator)

Code generators for creating bindings to the WebGL APIs.

```toml
[build-dependencies]
webgl_generator = "0.2.0"
```

### webgl-stdweb

[![Version](https://img.shields.io/crates/v/webgl_stdweb.svg)](https://crates.io/crates/webgl_stdweb) [![License](https://img.shields.io/crates/l/webgl_stdweb.svg)](https://github.com/brendanzab/gl-rs/blob/master/LICENSE) [![Downloads](https://img.shields.io/crates/d/webgl_stdweb.svg)](https://crates.io/crates/webgl_stdweb)

[README](webgl_stdweb)

WebGL bindings using stdweb

```toml
[build-dependencies]
webgl_stdweb = "0.3.0"
```
