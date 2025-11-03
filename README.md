# gl-rs

[![CI](https://github.com/rust-windowing/gl-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/rust-windowing/gl-rs/actions/workflows/ci.yml)

## Overview

This repository contains the necessary building blocks for OpenGL wrapper
libraries. For more information on each crate, see their respective READMEs
listed below.

The following crates are contained in this repository:

### [`gl`](./gl)

[![Version](https://img.shields.io/crates/v/gl.svg?logo=rust)](https://crates.io/crates/gl) [![Docs](https://img.shields.io/docsrs/gl.svg?logo=docsdotrs)](https://docs.rs/gl) [![License](https://img.shields.io/crates/l/gl.svg)](./gl/LICENSE) [![Downloads](https://img.shields.io/crates/d/gl.svg)](https://crates.io/crates/gl)

[README](./gl/README.md)

An OpenGL function pointer loader for the Rust Programming Language.

```toml
[dependencies]
gl = "0.14.0"
```

### [`gl_generator`](./gl_generator)

[![Version](https://img.shields.io/crates/v/gl_generator.svg?logo=rust)](https://crates.io/crates/gl_generator) [![Docs](https://img.shields.io/docsrs/gl_generator.svg?logo=docsdotrs)](https://docs.rs/gl_generator) [![License](https://img.shields.io/crates/l/gl_generator.svg)](./gl_generator/LICENSE) [![Downloads](https://img.shields.io/crates/d/gl_generator.svg)](https://crates.io/crates/gl_generator)

[README](./gl_generator/README.md)

Code generators for creating bindings to the Khronos OpenGL APIs.

```toml
[build-dependencies]
gl_generator = "0.14.0"
```

### [`khronos_api`](./khronos_api)

[![Version](https://img.shields.io/crates/v/khronos_api.svg?logo=rust)](https://crates.io/crates/khronos_api) [![Docs](https://img.shields.io/docsrs/khronos_api.svg?logo=docsdotrs)](https://docs.rs/khronos_api) [![License](https://img.shields.io/crates/l/khronos_api.svg)](./khronos_api/LICENSE) [![Downloads](https://img.shields.io/crates/d/khronos_api.svg)](https://crates.io/crates/khronos_api)

[README](./khronos_api/README.md)

The Khronos XML API Registry, exposed as byte string constants.

```toml
[build-dependencies]
khronos_api = "3.1.0"
```

## Compiling from source

`khronos_api` makes use of git submodules. You will need to initialize these before building:

```sh
git submodule update --init
```
