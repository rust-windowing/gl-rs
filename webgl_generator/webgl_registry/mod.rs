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

use std::collections::BTreeMap;
use std::{fmt, str};

use khronos_api;

use utils::convert_html_to_doc_comment;

pub use self::named::*;
pub use self::registry::*;
pub use self::types::*;

mod named;
mod registry;
mod types;

const HIDDEN_NAMES: &[&str] = &["WebGLObject", "WebGLContextEventInit"];
const RENDERING_CONTEXTS: &[(&str, &str)] = &[
    ("webgl", "WebGLRenderingContext"),
    ("webgl2", "WebGL2RenderingContext"),
];

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Api {
    WebGl,
    WebGl2,
}

#[derive(Debug)]
struct ExtensionIDL {
    pub name: String,
    pub idl: String,
    pub overview: String,
    pub new_funs: BTreeMap<String, String>,
    pub min_api: Api,
}

pub enum Exts<'a> {
    Include(&'a [&'a str]),
    Exclude(&'a [&'a str]),
}

impl<'a> Exts<'a> {
    pub const NONE: Exts<'a> = Exts::Include(&[]);
    pub const ALL: Exts<'a> = Exts::Exclude(&[]);

    fn enumerate(&self) -> Vec<ExtensionIDL> {
        use regex::{Regex, RegexBuilder};

        // The Khronos IDL files are... not quite right, so let's fix them up!
        let enum_regex = Regex::new("([(, ])enum\\b").unwrap();
        let missing_semicolon_regex = RegexBuilder::new("^}$").multi_line(true).build().unwrap();
        let shared_callback_regex = Regex::new("\\bAcquireResourcesCallback\\b").unwrap();

        let mut result = Vec::new();
        for &ext_xml in khronos_api::WEBGL_EXT_XML {
            let elem: xml::Element = str::from_utf8(ext_xml).unwrap().parse().unwrap();

            let overview_html = format!("{}", elem.get_child("overview", None).unwrap());

            let mut new_funs = BTreeMap::new();
            for new_fun in elem.get_children("newfun", None) {
                for f in new_fun.get_children("function", None) {
                    let name = f.get_attribute("name", None).unwrap().into();
                    let f_html = format!("{}", f);
                    new_funs.insert(name, convert_html_to_doc_comment(&f_html));
                }
            }

            let dependencies = elem.get_child("depends", None).unwrap();
            let api_dep = dependencies.get_child("api", None).unwrap();

            let mut ext = ExtensionIDL {
                name: elem.get_child("name", None).unwrap().content_str(),
                idl: elem.get_child("idl", None).unwrap().content_str(),
                overview: format!(
                    "/// Extension\n/// \n{}",
                    convert_html_to_doc_comment(&overview_html)
                ),
                new_funs,
                min_api: match api_dep.get_attribute("version", None).unwrap() {
                    "1.0" => Api::WebGl,
                    "2.0" => Api::WebGl2,
                    other => panic!("Unknown API version: {}", other),
                },
            };

            if match *self {
                Exts::Include(names) => names.contains(&&*ext.name),
                Exts::Exclude(names) => !names.contains(&&*ext.name),
            } {
                ext.idl = enum_regex.replace_all(&ext.idl, "${1}GLenum").into();
                ext.idl = missing_semicolon_regex.replace_all(&ext.idl, "};").into();
                ext.idl = shared_callback_regex
                    .replace_all(&ext.idl, "AcquireSharedResourcesCallback")
                    .into();
                result.push(ext);
            }
        }

        result
    }
}

impl Api {
    fn idl_consts(&self) -> &'static [&'static [u8]] {
        match *self {
            Api::WebGl => &[khronos_api::WEBGL_IDL],
            Api::WebGl2 => &[khronos_api::WEBGL_IDL, khronos_api::WEBGL2_IDL],
        }
    }
}

impl fmt::Display for Api {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Api::WebGl => write!(fmt, "webgl"),
            Api::WebGl2 => write!(fmt, "webgl2"),
        }
    }
}
