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

// This build script is used to discover all of the accepted extensions
// described in the Khronos WebGL repository, and construct a static slice
// containing the XML specifications for all of them.
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::*;

fn main() {
    // Create and open a file in the output directory to contain our generated rust code
    let dest = env::var("OUT_DIR").unwrap();
    let mut file = File::create(Path::new(&dest).join("webgl_exts.rs")).unwrap();

    // The relative path within this crate containing the WebGL extensions.
    let root = Path::new("api_webgl/extensions");

    // Generate a slice literal, looking like this:
    // `&[&*include_bytes!(..), &*include_bytes!(..), ..]`
    writeln!(file, "&[").unwrap();

    // The slice will have one entry for each WebGL extension. To find the
    // extensions we mirror the behaviour of the `api_webgl/extensions/find-exts`
    // shell script.
    let mut paths: Vec<_> = root
        .read_dir()
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    // Sort the list of paths in order for the webgl_exts.rs file to be created
    // deterministically.
    paths.sort();
    for path in paths {
        let ext_name = path.file_name().unwrap().to_str().unwrap();

        // Each item which is a directory, and is not named `template`, may be
        // an extension.
        if path.is_dir() && ext_name != "template" {
            // If the directory contains a file named `extension.xml`, then this
            // really is an extension.
            let ext_path = path.join("extension.xml");
            if ext_path.is_file() {
                // Include the XML file.
                writeln!(
                    file,
                    // The path of the crate must be resolved at compile-time of the generated code
                    // using this environment variable, not at runtime of this build script using
                    // current_dir() or canonicalization.  These will differ on at least Bazel
                    // when/where the build script is containerized.
                    r#"&*include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/{}")),"#,
                    ext_path.to_str().unwrap()
                )
                .unwrap();
            }
        }
    }

    // Close the slice
    writeln!(file, "]").unwrap();
}
