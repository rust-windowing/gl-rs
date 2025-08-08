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

extern crate khronos_api;

use std::borrow::Cow;
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use xml::attribute::OwnedAttribute;
use xml::reader::XmlEvent;
use xml::EventReader as XmlEventReader;

use registry::{Binding, Cmd, Enum, GlxOpcode, Group, Registry};
use {Api, Fallbacks, Profile};

pub fn from_xml<R: io::Read>(src: R, filter: &Filter, require_feature: bool) -> Registry {
    XmlEventReader::new(src)
        .into_iter()
        .map(Result::unwrap)
        .filter_map(ParseEvent::from_xml)
        .parse(filter, require_feature)
}

#[derive(Debug, PartialEq, Eq)]
struct Attribute {
    key: String,
    value: String,
}

impl Attribute {
    fn new<Key, Value>(key: Key, value: Value) -> Attribute
    where
        Key: ToString,
        Value: ToString,
    {
        Attribute {
            key: key.to_string(),
            value: value.to_string(),
        }
    }
}

impl From<OwnedAttribute> for Attribute {
    fn from(attribute: OwnedAttribute) -> Attribute {
        Attribute::new(attribute.name.local_name, attribute.value)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ParseEvent {
    Start(String, Vec<Attribute>),
    End(String),
    Text(String),
}

impl ParseEvent {
    fn from_xml(event: XmlEvent) -> Option<ParseEvent> {
        match event {
            XmlEvent::StartDocument { .. } => None,
            XmlEvent::EndDocument => None,
            XmlEvent::StartElement {
                name, attributes, ..
            } => {
                let attributes = attributes.into_iter().map(Attribute::from).collect();
                Some(ParseEvent::Start(name.local_name, attributes))
            },
            XmlEvent::EndElement { name } => Some(ParseEvent::End(name.local_name)),
            XmlEvent::Characters(chars) => Some(ParseEvent::Text(chars)),
            XmlEvent::ProcessingInstruction { .. } => None,
            XmlEvent::CData(_) => None,
            XmlEvent::Comment(_) => None,
            XmlEvent::Whitespace(_) => None,
        }
    }
}

fn api_from_str(src: &str) -> Result<Option<Api>, ()> {
    match src {
        "gl" => Ok(Some(Api::Gl)),
        "glx" => Ok(Some(Api::Glx)),
        "wgl" => Ok(Some(Api::Wgl)),
        "egl" => Ok(Some(Api::Egl)),
        "glcore" => Ok(Some(Api::GlCore)),
        "gles1" => Ok(Some(Api::Gles1)),
        "gles2" => Ok(Some(Api::Gles2)),
        "glsc2" => Ok(Some(Api::Glsc2)),
        "disabled" => Ok(None),
        _ => Err(()),
    }
}

fn profile_from_str(src: &str) -> Result<Profile, ()> {
    match src {
        "core" => Ok(Profile::Core),
        "compatibility" => Ok(Profile::Compatibility),
        _ => Err(()),
    }
}

fn underscore_numeric_prefix(src: &str) -> String {
    match src.chars().next() {
        Some(c) if c.is_numeric() => format!("_{src}"),
        Some(_) | None => src.to_string(),
    }
}

fn underscore_keyword(ident: String) -> String {
    match ident.as_ref() {
        "in" => "in_".to_string(),
        "ref" => "ref_".to_string(),
        "type" => "type_".to_string(),
        "box" => "box_".to_string(),
        _ => ident,
    }
}

fn trim_str<'a>(s: &'a str, trim: &str) -> &'a str {
    s.strip_prefix(trim).unwrap_or(s)
}

fn trim_enum_prefix(ident: &str, api: Api) -> String {
    let ident = match api {
        Api::Gl | Api::GlCore | Api::Gles1 | Api::Gles2 | Api::Glsc2 => trim_str(ident, "GL_"),
        Api::Glx => trim_str(ident, "GLX_"),
        Api::Wgl => trim_str(ident, "WGL_"),
        Api::Egl => trim_str(ident, "EGL_"),
    };
    underscore_numeric_prefix(ident)
}

fn make_enum(ident: String, ty: Option<String>, value: String, alias: Option<String>) -> Enum {
    let (ty, value, cast) = {
        if value.starts_with("((") && value.ends_with(")") {
            // Some enums have a value of the form `'((' type ')' expr ')'`.

            // nothing to see here....
            // just brute forcing some paren matching... (ﾉ ◕ ◡ ◕)ﾉ *:･ﾟ✧
            let working = &value[2..value.len() - 1];
            if let Some((i, _)) = working.match_indices(")").next() {
                let ty = working[..i].to_string();
                let value = working[i + 1..].to_string();

                (Cow::Owned(ty), value, true)
            } else {
                panic!("Unexpected value format: {}", value)
            }
        } else {
            let ty = match ty {
                Some(ref ty) if ty == "u" => "GLuint",
                Some(ref ty) if ty == "ull" => "GLuint64",
                Some(ty) => panic!("Unhandled enum type: {}", ty),
                None if value.starts_with("\"") => "&str",
                None if ident == "TRUE" || ident == "FALSE" => "GLboolean",
                None => "GLenum",
            };
            (Cow::Borrowed(ty), value, false)
        }
    };

    Enum {
        ident,
        value,
        cast,
        alias,
        ty,
    }
}

fn make_egl_enum(ident: String, ty: Option<String>, value: String, alias: Option<String>) -> Enum {
    let (ty, value, cast) = {
        if value.starts_with("EGL_CAST(") && value.ends_with(")") {
            // Handling "SpecialNumbers" in the egl.xml file
            // The values for these enums has the form `'EGL_CAST(' type ',' expr ')'`.
            let working = &value[9..value.len() - 1];
            if let Some((i, _)) = working.match_indices(",").next() {
                let ty = working[..i].to_string();
                let value = working[i + 1..].to_string();

                (Cow::Owned(ty), value, true)
            } else {
                panic!("Unexpected value format: {}", value)
            }
        } else {
            match value.chars().next() {
                Some('-') | Some('0'..='9') => (),
                _ => panic!("Unexpected value format: {}", value),
            }

            let ty = match ty {
                Some(ref ty) if ty == "ull" => "EGLuint64KHR",
                Some(ty) => panic!("Unhandled enum type: {}", ty),
                None if value.starts_with('-') => "EGLint",
                None if ident == "TRUE" || ident == "FALSE" => "EGLBoolean",
                None => "EGLenum",
            };
            (Cow::Borrowed(ty), value, false)
        }
    };

    Enum {
        ident,
        value,
        cast,
        alias,
        ty,
    }
}

fn trim_cmd_prefix(ident: &str, api: Api) -> &str {
    match api {
        Api::Gl | Api::GlCore | Api::Gles1 | Api::Gles2 | Api::Glsc2 => trim_str(ident, "gl"),
        Api::Glx => trim_str(ident, "glX"),
        Api::Wgl => trim_str(ident, "wgl"),
        Api::Egl => trim_str(ident, "egl"),
    }
}

fn merge_map(a: &mut BTreeMap<String, Vec<String>>, b: BTreeMap<String, Vec<String>>) {
    for (k, v) in b {
        match a.entry(k) {
            Entry::Occupied(mut ent) => {
                ent.get_mut().extend(v);
            },
            Entry::Vacant(ent) => {
                ent.insert(v);
            },
        }
    }
}

#[derive(Clone)]
struct Feature {
    pub api: Api,
    pub _name: String,
    pub number: String,
    pub requires: Vec<Require>,
    pub removes: Vec<Remove>,
}

#[derive(Clone)]
struct Require {
    pub api: Option<Api>,
    /// A reference to the earlier types, by name
    pub enums: Vec<String>,
    /// A reference to the earlier types, by name
    pub commands: Vec<String>,
}

#[derive(Clone)]
struct Remove {
    // always Core, for now
    pub profile: Profile,
    /// A reference to the earlier types, by name
    pub enums: Vec<String>,
    /// A reference to the earlier types, by name
    pub commands: Vec<String>,
}

#[derive(Clone)]
struct Extension {
    pub name: String,
    /// which apis this extension is defined for (see Feature.api)
    pub supported: Vec<Api>,
    pub requires: Vec<Require>,
}

pub struct Filter {
    pub api: Api,
    pub fallbacks: Fallbacks,
    pub extensions: BTreeSet<String>,
    pub profile: Profile,
    pub version: String,
}

trait Parse: Sized + Iterator<Item = ParseEvent> {
    fn parse(mut self, filter: &Filter, require_feature: bool) -> Registry {
        self.consume_start_element("registry");

        let mut enums = BTreeMap::new();
        let mut cmds = BTreeMap::new();
        let mut features = Vec::new();
        let mut extensions = BTreeMap::new();
        let mut aliases = BTreeMap::new();
        let mut groups = BTreeMap::new();

        while let Some(event) = self.next() {
            match event {
                // ignores
                ParseEvent::Text(_) => (),
                ParseEvent::Start(ref name, _) if name == "comment" => self.skip_to_end("comment"),
                ParseEvent::Start(ref name, _) if name == "types" => self.skip_to_end("types"),
                // ignore groups, they are deprecated in favour of `<enums group="">` elements which
                // are above all easier to parse like that.
                ParseEvent::Start(ref name, _) if name == "groups" => {
                    self.skip_to_end("groups");
                },

                // add enum namespace
                ParseEvent::Start(ref name, ref attributes) if name == "enums" => {
                    let enms = self.consume_enums(filter.api);
                    enums.extend(enms.iter().cloned());
                    let enums_group = get_attribute(attributes, "group");
                    let enums_type = get_attribute(attributes, "type");
                    if let Some(group) = enums_group {
                        let prev = groups.insert(
                            group.clone(),
                            Group {
                                ident: group.clone(),
                                enums_type,
                                enums: vec![],
                            },
                        );
                        assert!(prev.is_none(), "Group {group} is already inserted");
                    }
                },

                // add command namespace
                ParseEvent::Start(ref name, _) if name == "commands" => {
                    let (new_cmds, new_aliases) = self.consume_cmds(filter.api);
                    cmds.extend(new_cmds);
                    merge_map(&mut aliases, new_aliases);
                },

                ParseEvent::Start(ref name, ref attributes) if name == "feature" => {
                    debug!("Parsing feature: {attributes:?}");
                    features.push(Feature::convert(&mut self, attributes));
                },

                ParseEvent::Start(ref name, _) if name == "extensions" => loop {
                    match self.next().unwrap() {
                        ParseEvent::Start(ref name, ref attributes) if name == "extension" => {
                            let ext = Extension::convert(&mut self, attributes);
                            extensions.insert(ext.name.clone(), ext);
                        },
                        ParseEvent::End(ref name) if name == "extensions" => break,
                        event => panic!("Unexpected message {:?}", event),
                    }
                },

                // finished building the registry
                ParseEvent::End(ref name) if name == "registry" => break,

                // error handling
                event => panic!("Expected </registry>, found: {:?}", event),
            }
        }

        let mut desired_enums = BTreeSet::new();
        let mut desired_cmds = BTreeSet::new();

        // find the features we want
        let mut found_feature = false;
        for feature in &features {
            // XXX: verify that the string comparison with <= actually works as desired
            if feature.api == filter.api && feature.number <= filter.version {
                for require in &feature.requires {
                    desired_enums.extend(require.enums.iter().cloned());
                    desired_cmds.extend(require.commands.iter().cloned());
                }

                for remove in &feature.removes {
                    if remove.profile == filter.profile {
                        for enm in &remove.enums {
                            debug!("Removing {enm}");
                            let s = desired_enums.remove(enm);
                            assert!(s, "{enm} not known");
                        }
                        for cmd in &remove.commands {
                            debug!("Removing {cmd}");
                            let s = desired_cmds.remove(cmd);
                            assert!(s, "{cmd} not known");
                        }
                    }
                }
            }
            if feature.number == filter.version {
                found_feature = true;
            }
        }

        if !found_feature && require_feature {
            panic!("Did not find version {} in the registry", filter.version);
        }

        for extension_name in &filter.extensions {
            let extension = extensions.get(extension_name).unwrap_or_else(|| {
                panic!(
                    "Requested extension `{}` not defined for the {} API",
                    extension_name, filter.api
                )
            });
            if !extension.supported.contains(&filter.api) {
                panic!(
                    "Requested extension `{}` doesn't support the {} API",
                    extension.name, filter.api
                );
            }
            for require in &extension.requires {
                // TODO: Just like enums, this filter should be moved to the point where the element
                // is parsed so that we can skip it directly, filter.api is already known at that
                // point.  The same is true for <feature> and <extension>, though the latter allows
                // us to provide a more useful message to the user.
                if require.api.is_none_or(|a| a == filter.api) {
                    desired_enums.extend(require.enums.iter().cloned());
                    desired_cmds.extend(require.commands.iter().cloned());
                }
            }
        }

        let enums = desired_enums
            .iter()
            .map(|n| enums.remove(n).expect(n))
            .collect();
        let cmds = desired_cmds
            .iter()
            .map(|n| cmds.remove(n).expect(n))
            .collect();

        Registry {
            api: filter.api,
            enums,
            // : enums.into_iter().filter(is_desired_enum).collect(),
            cmds,
            // : cmds.into_iter().filter(is_desired_cmd).collect(),
            aliases: if filter.fallbacks == Fallbacks::None {
                BTreeMap::new()
            } else {
                aliases
            },
            groups,
        }
    }

    fn consume_characters(&mut self) -> String {
        match self.next().unwrap() {
            ParseEvent::Text(ch) => ch,
            event => panic!("Expected characters, found: {:?}", event),
        }
    }

    fn consume_start_element(&mut self, expected_name: &str) -> Vec<Attribute> {
        match self.next().unwrap() {
            ParseEvent::Start(name, attributes) => {
                if expected_name == name {
                    attributes
                } else {
                    panic!("Expected <{}>, found: <{}>", expected_name, name)
                }
            },
            event => panic!("Expected <{}>, found: {:?}", expected_name, event),
        }
    }

    fn consume_end_element(&mut self, expected_name: &str) {
        match self.next().unwrap() {
            ParseEvent::End(ref name) if expected_name == name => (),
            event => panic!("Expected </{}>, found: {:?}", expected_name, event),
        }
    }

    fn skip_to_end(&mut self, expected_name: &str) {
        loop {
            match self.next().unwrap() {
                ParseEvent::End(ref name) if expected_name == name => break,
                _ => {},
            }
        }
    }

    fn consume_two<'a, T: FromXml, U: FromXml>(
        &mut self,
        one: &'a str,
        two: &'a str,
        end: &'a str,
    ) -> (Vec<T>, Vec<U>) {
        debug!("consume_two: looking for {one} and {two} until {end}");

        let mut ones = Vec::new();
        let mut twos = Vec::new();

        loop {
            match self.next().unwrap() {
                ParseEvent::Start(ref name, ref attributes) => {
                    debug!("Found start element <{name:?} {attributes:?}>");
                    debug!("one and two are {one} and {two}");

                    let n = name.clone();

                    if one == n {
                        ones.push(FromXml::convert(self, attributes));
                    } else if "type" == n {
                        // XXX: GL1.1 contains types, which we never care about anyway.
                        // Make sure consume_two doesn't get used for things which *do*
                        // care about type.
                        warn!("Ignoring type!");
                        continue;
                    } else if two == n {
                        twos.push(FromXml::convert(self, attributes));
                    } else {
                        panic!("Unexpected element: <{:?} {:?}>", n, &attributes);
                    }
                },
                ParseEvent::End(ref name) => {
                    debug!("Found end element </{name:?}>");

                    if one == name || two == name {
                        continue;
                    } else if "type" == name {
                        // XXX: GL1.1 contains types, which we never care about anyway.
                        // Make sure consume_two doesn't get used for things which *do*
                        // care about type.
                        warn!("Ignoring type!");
                        continue;
                    } else if end == name {
                        return (ones, twos);
                    } else {
                        panic!("Unexpected end element {:?}", name);
                    }
                },
                event => panic!("Unexpected message {:?}", event),
            }
        }
    }

    fn consume_enums(&mut self, api: Api) -> Vec<(String, Enum)> {
        let mut enums = Vec::new();
        loop {
            match self.next().unwrap() {
                // ignores
                ParseEvent::Text(_) => {},
                ParseEvent::Start(ref name, _) if name == "unused" => self.skip_to_end("unused"),

                // add enum definition
                ParseEvent::Start(ref name, ref attributes) if name == "enum" => {
                    let enum_api = get_attribute(attributes, "api")
                        .map(|api| api_from_str(&api).unwrap().unwrap());
                    let enm = self.consume_enum(api, attributes);
                    if enum_api.is_none_or(|a| a == api) {
                        enums.push(enm);
                    }
                },

                // finished building the namespace
                ParseEvent::End(ref name) if name == "enums" => break,
                // error handling
                event => panic!("Expected </enums>, found: {:?}", event),
            }
        }
        enums
    }

    fn consume_enum(&mut self, api: Api, attributes: &[Attribute]) -> (String, Enum) {
        let name = get_attribute(attributes, "name").unwrap();
        let ident = trim_enum_prefix(&name, api).to_string();
        let value = get_attribute(attributes, "value").unwrap();
        let alias = get_attribute(attributes, "alias");
        let ty = get_attribute(attributes, "type");
        self.consume_end_element("enum");

        (
            name,
            match api {
                Api::Egl => make_egl_enum(ident, ty, value, alias),
                _ => make_enum(ident, ty, value, alias),
            },
        )
    }

    fn consume_cmds(&mut self, api: Api) -> (Vec<(String, Cmd)>, BTreeMap<String, Vec<String>>) {
        let mut cmds = Vec::new();
        let mut aliases: BTreeMap<String, Vec<String>> = BTreeMap::new();
        loop {
            match self.next().unwrap() {
                // add command definition
                ParseEvent::Start(ref name, _) if name == "command" => {
                    let new = self.consume_cmd(api);
                    if let Some(ref v) = new.1.alias {
                        match aliases.entry(v.clone()) {
                            Entry::Occupied(mut ent) => {
                                ent.get_mut().push(new.1.proto.ident.clone());
                            },
                            Entry::Vacant(ent) => {
                                ent.insert(vec![new.1.proto.ident.clone()]);
                            },
                        }
                    }
                    cmds.push(new);
                },
                // finished building the namespace
                ParseEvent::End(ref name) if name == "commands" => break,
                // error handling
                event => panic!("Expected </commands>, found: {:?}", event),
            }
        }
        (cmds, aliases)
    }

    fn consume_cmd(&mut self, api: Api) -> (String, Cmd) {
        // consume command prototype
        self.consume_start_element("proto");
        let mut proto = self.consume_binding("proto", &[]);
        let name = std::mem::take(&mut proto.ident);
        proto.ident = trim_cmd_prefix(&name, api).to_string();

        let mut params = Vec::new();
        let mut alias = None;
        let mut vecequiv = None;
        let mut glx = None;
        loop {
            match self.next().unwrap() {
                ParseEvent::Start(ref name, ref attributes) if name == "param" => {
                    params.push(self.consume_binding("param", attributes));
                },
                ParseEvent::Start(ref name, ref attributes) if name == "alias" => {
                    alias = get_attribute(attributes, "name");
                    // TODO: Use untrimmed names for aliases?
                    alias = alias.map(|t| trim_cmd_prefix(&t, api).to_string());
                    self.consume_end_element("alias");
                },
                ParseEvent::Start(ref name, ref attributes) if name == "vecequiv" => {
                    vecequiv = get_attribute(attributes, "vecequiv");
                    self.consume_end_element("vecequiv");
                },
                ParseEvent::Start(ref name, ref attributes) if name == "glx" => {
                    glx = Some(GlxOpcode {
                        opcode: get_attribute(attributes, "opcode").unwrap(),
                        name: get_attribute(attributes, "name"),
                    });
                    self.consume_end_element("glx");
                },
                ParseEvent::End(ref name) if name == "command" => break,
                event => panic!("Expected </command>, found: {:?}", event),
            }
        }

        (
            name,
            Cmd {
                proto,
                params,
                alias,
                vecequiv,
                glx,
            },
        )
    }

    fn consume_binding(&mut self, outside_tag: &str, attributes: &[Attribute]) -> Binding {
        // consume type
        let mut ty = String::new();
        loop {
            match self.next().unwrap() {
                ParseEvent::Text(text) => ty.push_str(&text),
                ParseEvent::Start(ref name, _) if name == "ptype" => (),
                ParseEvent::End(ref name) if name == "ptype" => (),
                ParseEvent::Start(ref name, _) if name == "name" => break,
                event => panic!("Expected binding, found: {:?}", event),
            }
        }

        // consume identifier
        let ident = underscore_keyword(self.consume_characters());
        self.consume_end_element("name");

        // consume the type suffix
        loop {
            match self.next().unwrap() {
                ParseEvent::Text(text) => ty.push_str(&text),
                ParseEvent::End(ref name) if name == outside_tag => break,
                event => panic!("Expected binding, found: {:?}", event),
            }
        }

        Binding {
            ident,
            ty: to_rust_ty(ty),
            group: get_attribute(attributes, "group"),
        }
    }
}

impl<T> Parse for T where T: Sized + Iterator<Item = ParseEvent> {}

fn get_attribute(attribs: &[Attribute], key: &str) -> Option<String> {
    attribs
        .iter()
        .find(|attrib| attrib.key == key)
        .map(|attrib| attrib.value.clone())
}

trait FromXml {
    fn convert<P: Parse>(parser: &mut P, a: &[Attribute]) -> Self;
}

impl FromXml for Require {
    fn convert<P: Parse>(parser: &mut P, a: &[Attribute]) -> Require {
        debug!("Doing a FromXml on Require");
        let api = get_attribute(a, "api").map(|api| api_from_str(&api).unwrap().unwrap());
        let (enums, commands) = parser.consume_two("enum", "command", "require");
        Require {
            api,
            enums,
            commands,
        }
    }
}

impl FromXml for Remove {
    fn convert<P: Parse>(parser: &mut P, a: &[Attribute]) -> Remove {
        debug!("Doing a FromXml on Remove");
        let profile = get_attribute(a, "profile").unwrap();
        let profile = profile_from_str(&profile).unwrap();
        let (enums, commands) = parser.consume_two("enum", "command", "remove");

        Remove {
            profile,
            enums,
            commands,
        }
    }
}

impl FromXml for Feature {
    fn convert<P: Parse>(parser: &mut P, a: &[Attribute]) -> Feature {
        debug!("Doing a FromXml on Feature");
        let api = get_attribute(a, "api").unwrap();
        let api = api_from_str(&api).unwrap().unwrap();
        let name = get_attribute(a, "name").unwrap();
        let number = get_attribute(a, "number").unwrap();

        debug!("Found api = {api}, name = {name}, number = {number}");

        let (require, remove) = parser.consume_two("require", "remove", "feature");

        Feature {
            api,
            _name: name,
            number,
            requires: require,
            removes: remove,
        }
    }
}

impl FromXml for Extension {
    fn convert<P: Parse>(parser: &mut P, a: &[Attribute]) -> Extension {
        debug!("Doing a FromXml on Extension");
        let name = get_attribute(a, "name").unwrap();
        let supported = get_attribute(a, "supported")
            .unwrap()
            .split('|')
            .filter_map(|api| {
                api_from_str(api).unwrap_or_else(|()| panic!("unsupported API `{}`", api))
            })
            .collect::<Vec<_>>();
        let mut require = Vec::new();
        loop {
            match parser.next().unwrap() {
                ParseEvent::Start(ref name, ref attributes) if name == "require" => {
                    require.push(FromXml::convert(parser, attributes));
                },
                ParseEvent::End(ref name) if name == "extension" => break,
                event => panic!("Unexpected message {:?}", event),
            }
        }

        Extension {
            name,
            supported,
            requires: require,
        }
    }
}

impl FromXml for String {
    fn convert<P: Parse>(_: &mut P, a: &[Attribute]) -> String {
        get_attribute(a, "name").unwrap()
    }
}

/// Converts a C style type definition to the Rust equivalent
pub fn to_rust_ty<T: AsRef<str>>(ty: T) -> Cow<'static, str> {
    let ty = match ty.as_ref().trim() {
        // gl.xml types
        "GLDEBUGPROC" => "types::GLDEBUGPROC",
        "GLDEBUGPROCAMD" => "types::GLDEBUGPROCAMD",
        "GLDEBUGPROCARB" => "types::GLDEBUGPROCARB",
        "GLDEBUGPROCKHR" => "types::GLDEBUGPROCKHR",
        "GLbitfield" => "types::GLbitfield",
        "GLboolean" => "types::GLboolean",
        "GLbyte" => "types::GLbyte",
        "GLclampd" => "types::GLclampd",
        "GLclampf" => "types::GLclampf",
        "GLclampx" => "types::GLclampx",
        "GLdouble" => "types::GLdouble",
        "GLeglImageOES" => "types::GLeglImageOES",
        "GLenum" => "types::GLenum",
        "GLfixed" => "types::GLfixed",
        "GLfloat" => "types::GLfloat",
        "GLhalfNV" => "types::GLhalfNV",
        "GLhandleARB" => "types::GLhandleARB",
        "GLint" => "types::GLint",
        "GLint64" => "types::GLint64",
        "GLint64EXT" => "types::GLint64EXT",
        "GLintptr" => "types::GLintptr",
        "GLintptrARB" => "types::GLintptrARB",
        "GLshort" => "types::GLshort",
        "GLsizei" => "types::GLsizei",
        "GLsizeiptr" => "types::GLsizeiptr",
        "GLsizeiptrARB" => "types::GLsizeiptrARB",
        "GLsync" => "types::GLsync",
        "GLubyte" => "types::GLubyte",
        "GLuint" => "types::GLuint",
        "GLuint64" => "types::GLuint64",
        "GLuint64EXT" => "types::GLuint64EXT",
        "GLushort" => "types::GLushort",
        "GLvdpauSurfaceNV" => "types::GLvdpauSurfaceNV",
        "void" => "()",
        "GLboolean *" => "*mut types::GLboolean",
        "GLchar *" => "*mut types::GLchar",
        "const GLchar*" => "*const types::GLchar",
        "GLcharARB *" => "*mut types::GLcharARB",
        "GLdouble *" => "*mut types::GLdouble",
        "GLenum *" => "*mut types::GLenum",
        "GLfixed *" => "*mut types::GLfixed",
        "GLfloat *" => "*mut types::GLfloat",
        "GLhandleARB *" => "*mut types::GLhandleARB",
        "GLint *" => "*mut types::GLint",
        "GLint64 *" => "*mut types::GLint64",
        "GLint64EXT *" => "*mut types::GLint64EXT",
        "GLsizei *" => "*mut types::GLsizei",
        "GLubyte *" => "*mut types::GLubyte",
        "GLuint *" => "*mut types::GLuint",
        "GLuint [2]" => "*mut [types::GLuint; 2]",
        "GLuint64 *" => "*mut types::GLuint64",
        "GLuint64EXT *" => "*mut types::GLuint64EXT",
        "GLushort *" => "*mut types::GLushort",
        "GLvoid *" => "*mut types::GLvoid",
        "GLvoid **" => "*const *mut types::GLvoid",
        "void *" => "*mut __gl_imports::raw::c_void",
        "void **" => "*const *mut __gl_imports::raw::c_void",
        "const GLboolean *" => "*const types::GLboolean",
        "const GLbyte *" => "*const types::GLbyte",
        "const GLchar *" => "*const types::GLchar",
        "const GLcharARB *" => "*const types::GLcharARB",
        "const GLclampf *" => "*const types::GLclampf",
        "const GLdouble *" => "*const types::GLdouble",
        "const GLenum *" => "*const types::GLenum",
        "const GLfixed *" => "*const types::GLfixed",
        "const GLfloat" => "types::GLfloat",
        "const GLfloat *" => "*const types::GLfloat",
        "const GLhalfNV *" => "*const types::GLhalfNV",
        "const GLint *" => "*const types::GLint",
        "const GLint*" => "*const types::GLint",
        "const GLint64 *" => "*const types::GLint64",
        "const GLint64EXT *" => "*const types::GLint64EXT",
        "const GLintptr *" => "*const types::GLintptr",
        "const GLshort *" => "*const types::GLshort",
        "const GLsizei*" | "const GLsizei *" => "*const types::GLsizei",
        "const GLsizeiptr *" => "*const types::GLsizeiptr",
        "const GLubyte *" => "*const types::GLubyte",
        "const GLuint *" => "*const types::GLuint",
        "const GLuint64 *" => "*const types::GLuint64",
        "const GLuint64EXT *" => "*const types::GLuint64EXT",
        "const GLushort *" => "*const types::GLushort",
        "const GLvdpauSurfaceNV *" => "*const types::GLvdpauSurfaceNV",
        "const GLvoid *" => "*const types::GLvoid",
        "const void*" | "const void *" => "*const __gl_imports::raw::c_void",
        "const void **" => "*const *const __gl_imports::raw::c_void",
        "const void *const*" => "*const *const __gl_imports::raw::c_void",
        "const GLboolean **" => "*const *const types::GLboolean",
        "const GLchar **" => "*const *const types::GLchar",
        "const GLcharARB **" => "*const *const types::GLcharARB",
        "const GLvoid **" => "*const *const types::GLvoid",
        "const GLchar *const*" => "*const *const types::GLchar",
        "const GLvoid *const*" => "*const *const types::GLvoid",
        "struct _cl_context *" => "*const types::_cl_context",
        "struct _cl_event *" => "*const types::_cl_event",
        "GLuint[2]" => "[Gluint; 2]",

        // glx.xml types
        "Bool" => "types::Bool",
        "Colormap" => "types::Colormap",
        "DMbuffer" => "types::DMbuffer",
        "Font" => "types::Font",
        "GLXContext" => "types::GLXContext",
        "GLXContextID" => "types::GLXContextID",
        "GLXDrawable" => "types::GLXDrawable",
        "GLXFBConfig" => "types::GLXFBConfig",
        "GLXFBConfigSGIX" => "types::GLXFBConfigSGIX",
        "GLXPbuffer" => "types::GLXPbuffer",
        "GLXPbufferSGIX" => "types::GLXPbufferSGIX",
        "GLXPixmap" => "types::GLXPixmap",
        "GLXVideoCaptureDeviceNV" => "types::GLXVideoCaptureDeviceNV",
        "GLXVideoDeviceNV" => "types::GLXVideoDeviceNV",
        "GLXVideoSourceSGIX" => "types::GLXVideoSourceSGIX",
        "GLXWindow" => "types::GLXWindow",
        // "GLboolean"                 => "types::GLboolean",
        // "GLenum"                    => "types::GLenum",
        // "GLint"                     => "types::GLint",
        // "GLsizei"                   => "types::GLsizei",
        // "GLuint"                    => "types::GLuint",
        "Pixmap" => "types::Pixmap",
        "Status" => "types::Status",
        "VLNode" => "types::VLNode",
        "VLPath" => "types::VLPath",
        "VLServer" => "types::VLServer",
        "Window" => "types::Window",
        "__GLXextFuncPtr" => "types::__GLXextFuncPtr",
        "const GLXContext" => "const types::GLXContext",
        "float" => "__gl_imports::raw::c_float",
        "int" => "__gl_imports::raw::c_int",
        "int64_t" => "i64",
        "unsigned int" => "__gl_imports::raw::c_uint",
        "unsigned long" => "__gl_imports::raw::c_ulong",
        // "void "                     => "()",
        "DMparams *" => "*mut types::DMparams",
        "Display *" => "*mut types::Display",
        "GLXFBConfig *" => "*mut types::GLXFBConfig",
        "GLXFBConfigSGIX *" => "*mut types::GLXFBConfigSGIX",
        "GLXHyperpipeConfigSGIX *" => "*mut types::GLXHyperpipeConfigSGIX",
        "GLXHyperpipeNetworkSGIX *" => "*mut types::GLXHyperpipeNetworkSGIX",
        "GLXVideoCaptureDeviceNV *" => "*mut types::GLXVideoCaptureDeviceNV",
        "GLXVideoDeviceNV *" => "*mut types::GLXVideoDeviceNV",
        // "GLuint *"                  => "*mut types::GLuint",
        "XVisualInfo *" => "*mut types::XVisualInfo",
        // "const GLubyte *"           => "*GLubyte",
        "const char *" => "*const __gl_imports::raw::c_char",
        "const int *" => "*const __gl_imports::raw::c_int",
        // "const void *"              => "*const __gl_imports::raw::c_void",
        "int *" => "*mut __gl_imports::raw::c_int",
        "int32_t *" => "*mut i32",
        "int64_t *" => "*mut i64",
        "long *" => "*mut __gl_imports::raw::c_long",
        "unsigned int *" => "*mut __gl_imports::raw::c_uint",
        "unsigned long *" => "*mut __gl_imports::raw::c_ulong",
        // "void *"                    => "*mut __gl_imports::raw::c_void",

        // wgl.xml types
        "BOOL" => "types::BOOL",
        "DWORD" => "types::DWORD",
        "FLOAT" => "types::FLOAT",
        // "GLbitfield"                => "types::GLbitfield",
        // "GLboolean"                 => "types::GLboolean",
        // "GLenum"                    => "types::GLenum",
        // "GLfloat"                   => "types::GLfloat",
        // "GLint"                     => "types::GLint",
        // "GLsizei"                   => "types::GLsizei",
        // "GLuint"                    => "types::GLuint",
        // "GLushort"                  => "types::GLushort",
        "HANDLE" => "types::HANDLE",
        "HDC" => "types::HDC",
        "HENHMETAFILE" => "types::HENHMETAFILE",
        "HGLRC" => "types::HGLRC",
        "HGPUNV" => "types::HGPUNV",
        "HPBUFFERARB" => "types::HPBUFFERARB",
        "HPBUFFEREXT" => "types::HPBUFFEREXT",
        "HPVIDEODEV" => "types::HPVIDEODEV",
        "HVIDEOINPUTDEVICENV" => "types::HVIDEOINPUTDEVICENV",
        "HVIDEOOUTPUTDEVICENV" => "types::HVIDEOOUTPUTDEVICENV",
        "INT" => "types::INT",
        "INT64" => "types::INT64",
        "LPCSTR" => "types::LPCSTR",
        "LPGLYPHMETRICSFLOAT" => "types::LPGLYPHMETRICSFLOAT",
        "LPVOID" => "types::LPVOID",
        "PGPU_DEVICE" => "types::PGPU_DEVICE",
        "PROC" => "types::PROC",
        "UINT" => "types::UINT",
        "VOID" => "types::VOID",
        // "int "                      => "__gl_imports::raw::c_int",
        // "unsigned int "             => "__gl_imports::raw::c_uint",
        // "void "                     => "()",
        "BOOL *" => "*mut types::BOOL",
        "DWORD *" => "*mut types::DWORD",
        "FLOAT *" => "*mut types::FLOAT",
        // "GLuint *"                  => "*mut types::GLuint",
        "HANDLE *" => "*mut types::HANDLE",
        "HGPUNV *" => "*mut types::HGPUNV",
        "HPVIDEODEV *" => "*mut types::HPVIDEODEV",
        "HVIDEOINPUTDEVICENV *" => "*mut types::HVIDEOINPUTDEVICENV",
        "HVIDEOOUTPUTDEVICENV *" => "*mut types::HVIDEOOUTPUTDEVICENV",
        "INT32 *" => "*mut types::INT32",
        "INT64 *" => "*mut types::INT64",
        "UINT *" => "*mut types::UINT",
        "USHORT *" => "*mut types::USHORT",
        "const COLORREF *" => "*const types::COLORREF",
        "const DWORD *" => "*const types::DWORD",
        "const FLOAT *" => "*const types::FLOAT",
        // "const GLushort *"          => "*const types::GLushort",
        "const HANDLE *" => "*const types::HANDLE",
        "const HGPUNV *" => "*const types::HGPUNV",
        "const LAYERPLANEDESCRIPTOR *" => "*const types::LAYERPLANEDESCRIPTOR",
        "const LPVOID *" => "*const types::LPVOID",
        "const PIXELFORMATDESCRIPTOR *" => "*const types::PIXELFORMATDESCRIPTOR",
        "const USHORT *" => "*const types::USHORT",
        // "const char *"              => "*const __gl_imports::raw::c_char",
        // "const int *"               => "*const __gl_imports::raw::c_int",
        "float *" => "*mut __gl_imports::raw::c_float",
        // "int *"                     => "*mut __gl_imports::raw::c_int",
        // "unsigned long *"           => "*mut __gl_imports::raw::c_ulong",
        // "void *"                    => "*mut __gl_imports::raw::c_void",

        // elx.xml types
        "khronos_utime_nanoseconds_t" => "types::khronos_utime_nanoseconds_t",
        "khronos_uint64_t" => "types::khronos_uint64_t",
        "khronos_ssize_t" => "types::khronos_ssize_t",
        "EGLNativeDisplayType" => "types::EGLNativeDisplayType",
        "EGLNativePixmapType" => "types::EGLNativePixmapType",
        "EGLNativeWindowType" => "types::EGLNativeWindowType",
        "EGLint" => "types::EGLint",
        "EGLint *" => "*mut types::EGLint",
        "const EGLint *" => "*const types::EGLint",
        "NativeDisplayType" => "types::NativeDisplayType",
        "NativePixmapType" => "types::NativePixmapType",
        "NativeWindowType" => "types::NativeWindowType",
        //"Bool"                      => "types::Bool",
        "EGLBoolean" => "types::EGLBoolean",
        "EGLenum" => "types::EGLenum",
        "EGLenum *" => "*mut types::EGLenum",
        "EGLAttribKHR" => "types::EGLAttribKHR",
        "EGLAttrib" => "types::EGLAttrib",
        "EGLAttrib *" => "*mut types::EGLAttrib",
        "const EGLAttrib *" => "*const types::EGLAttrib",
        "const EGLattrib *" => "*const types::EGLAttrib", // Due to a typo in khronos_api/api_angle/scripts/egl_angle_ext.xml - see brendanzab/gl-rs#491
        "EGLConfig" => "types::EGLConfig",
        "EGLConfig *" => "*mut types::EGLConfig",
        "EGLContext" => "types::EGLContext",
        "EGLDeviceEXT" => "types::EGLDeviceEXT",
        "EGLDisplay" => "types::EGLDisplay",
        "EGLSurface" => "types::EGLSurface",
        "EGLClientBuffer" => "types::EGLClientBuffer",
        "__eglMustCastToProperFunctionPointerType" => {
            "types::__eglMustCastToProperFunctionPointerType"
        },
        "EGLImageKHR" => "types::EGLImageKHR",
        "EGLImage" => "types::EGLImage",
        "EGLImage *" => "*mut types::EGLImage",
        "EGLOutputLayerEXT" => "types::EGLOutputLayerEXT",
        "EGLOutputPortEXT" => "types::EGLOutputPortEXT",
        "EGLSyncKHR" => "types::EGLSyncKHR",
        "EGLSync" => "types::EGLSync",
        "EGLTimeKHR" => "types::EGLTimeKHR",
        "EGLTime" => "types::EGLTime",
        "EGLSyncNV" => "types::EGLSyncNV",
        "EGLTimeNV" => "types::EGLTimeNV",
        "EGLuint64NV" => "types::EGLuint64NV",
        "EGLStreamKHR" => "types::EGLStreamKHR",
        "EGLuint64KHR" => "types::EGLuint64KHR",
        "EGLNativeFileDescriptorKHR" => "types::EGLNativeFileDescriptorKHR",
        "EGLsizeiANDROID" => "types::EGLsizeiANDROID",
        "EGLSetBlobFuncANDROID" => "types::EGLSetBlobFuncANDROID",
        "EGLGetBlobFuncANDROID" => "types::EGLGetBlobFuncANDROID",
        "EGLClientPixmapHI" => "types::EGLClientPixmapHI",
        "struct EGLClientPixmapHI *" => "*const types::EGLClientPixmapHI",
        "const EGLAttribKHR *" => "*const types::EGLAttribKHR",
        "const EGLuint64KHR *" => "*const types::EGLuint64KHR",
        "EGLAttribKHR *" => "*mut types::EGLAttribKHR",
        "EGLDeviceEXT *" => "*mut types::EGLDeviceEXT",
        "EGLNativeDisplayType *" => "*mut types::EGLNativeDisplayType",
        "EGLNativePixmapType *" => "*mut types::EGLNativePixmapType",
        "EGLNativeWindowType *" => "*mut types::EGLNativeWindowType",
        "EGLOutputLayerEXT *" => "*mut types::EGLOutputLayerEXT",
        "EGLTimeKHR *" => "*mut types::EGLTimeKHR",
        "EGLOutputPortEXT *" => "*mut types::EGLOutputPortEXT",
        "EGLuint64KHR *" => "*mut types::EGLuint64KHR",
        "const struct AHardwareBuffer *" => "*const __gl_imports::raw::c_void", // humm
        "char *" => "*const __gl_imports::raw::c_char",
        "struct wl_buffer *" => "*const __gl_imports::raw::c_void",
        "struct wl_display *" => "*const __gl_imports::raw::c_void",
        "struct wl_resource *" => "*const __gl_imports::raw::c_void",

        "GLeglClientBufferEXT" => "types::GLeglClientBufferEXT",
        "GLVULKANPROCNV" => "types::GLVULKANPROCNV",
        "EGLDEBUGPROCKHR" => "types::EGLDEBUGPROCKHR",
        "EGLObjectKHR" => "types::EGLObjectKHR",
        "EGLLabelKHR" => "types::EGLLabelKHR",
        "EGLnsecsANDROID" => "types::EGLnsecsANDROID",
        "EGLnsecsANDROID *" => "*mut types::EGLnsecsANDROID",
        "EGLBoolean *" => "*mut types::EGLBoolean",

        // failure
        _ => panic!("Type conversion not implemented for `{}`", ty.as_ref()),
    };

    Cow::Borrowed(ty)
}

#[cfg(test)]
mod tests {
    mod underscore_numeric_prefix {
        use registry::parse;

        #[test]
        fn test_numeric_prefix() {
            assert_eq!(parse::underscore_numeric_prefix("3"), "_3");
            assert_eq!(parse::underscore_numeric_prefix("123_FOO"), "_123_FOO");
        }

        #[test]
        fn test_non_numeric_prefix() {
            assert_eq!(parse::underscore_numeric_prefix(""), "");
            assert_eq!(parse::underscore_numeric_prefix("A"), "A");
            assert_eq!(parse::underscore_numeric_prefix("FOO"), "FOO");
        }
    }

    mod underscore_keyword {
        use registry::parse;

        #[test]
        fn test_keyword() {
            assert_eq!(parse::underscore_keyword("in".to_string()), "in_");
            assert_eq!(parse::underscore_keyword("ref".to_string()), "ref_");
            assert_eq!(parse::underscore_keyword("type".to_string()), "type_");
        }

        #[test]
        fn test_non_keyword() {
            assert_eq!(parse::underscore_keyword("foo".to_string()), "foo");
            assert_eq!(parse::underscore_keyword("bar".to_string()), "bar");
        }
    }
    mod make_enum {
        use registry::parse;

        #[test]
        fn test_cast_0() {
            let e = parse::make_enum(
                "FOO".to_string(),
                None,
                "((EGLint)-1)".to_string(),
                Some("BAR".to_string()),
            );
            assert_eq!(e.ident, "FOO");
            assert_eq!((&*e.ty, &*e.value), ("EGLint", "-1"));
            assert_eq!(e.alias, Some("BAR".to_string()));
        }

        #[test]
        fn test_cast_1() {
            let e = parse::make_enum(
                "FOO".to_string(),
                None,
                "((EGLint)(-1))".to_string(),
                Some("BAR".to_string()),
            );
            assert_eq!(e.ident, "FOO");
            assert_eq!((&*e.ty, &*e.value), ("EGLint", "(-1)"));
            assert_eq!(e.alias, Some("BAR".to_string()));
        }

        #[test]
        fn test_no_type() {
            let e = parse::make_enum(
                "FOO".to_string(),
                None,
                "value".to_string(),
                Some("BAR".to_string()),
            );
            assert_eq!(e.ident, "FOO");
            assert_eq!(e.value, "value");
            assert_eq!(e.alias, Some("BAR".to_string()));
            assert_eq!(e.ty, "GLenum");
            assert!(!e.cast);
        }

        #[test]
        fn test_u() {
            let e = parse::make_enum(
                "FOO".to_string(),
                Some("u".to_string()),
                String::new(),
                None,
            );
            assert_eq!(e.ty, "GLuint");
        }

        #[test]
        fn test_ull() {
            let e = parse::make_enum(
                "FOO".to_string(),
                Some("ull".to_string()),
                String::new(),
                None,
            );
            assert_eq!(e.ty, "GLuint64");
        }

        #[test]
        #[should_panic]
        fn test_unknown_type() {
            parse::make_enum(
                "FOO".to_string(),
                Some("blargh".to_string()),
                String::new(),
                None,
            );
        }

        #[test]
        fn test_value_str() {
            let e = parse::make_enum("FOO".to_string(), None, "\"hi\"".to_string(), None);
            assert_eq!(e.ty, "&'static str");
        }

        #[test]
        fn test_ident_true() {
            let e = parse::make_enum("TRUE".to_string(), None, String::new(), None);
            assert_eq!(e.ty, "GLboolean");
        }

        #[test]
        fn test_ident_false() {
            let e = parse::make_enum("FALSE".to_string(), None, String::new(), None);
            assert_eq!(e.ty, "GLboolean");
        }
    }

    mod make_egl_enum {
        use registry::parse;

        #[test]
        fn test_cast_egl() {
            let e = parse::make_egl_enum(
                "FOO".to_string(),
                None,
                "EGL_CAST(EGLint,-1)".to_string(),
                Some("BAR".to_string()),
            );
            assert_eq!(e.ident, "FOO");
            assert_eq!((&*e.ty, &*e.value), ("EGLint", "-1"));
            assert_eq!(e.alias, Some("BAR".to_string()));
        }

        #[test]
        fn test_ident_true() {
            let e = parse::make_egl_enum("TRUE".to_string(), None, "1234".to_string(), None);
            assert_eq!(e.ty, "EGLBoolean");
        }

        #[test]
        fn test_ident_false() {
            let e = parse::make_egl_enum("FALSE".to_string(), None, "1234".to_string(), None);
            assert_eq!(e.ty, "EGLBoolean");
        }

        #[test]
        fn test_ull() {
            let e = parse::make_egl_enum(
                "FOO".to_string(),
                Some("ull".to_string()),
                "1234".to_string(),
                None,
            );
            assert_eq!(e.ty, "EGLuint64KHR");
        }

        #[test]
        fn test_negative_value() {
            let e = parse::make_egl_enum("FOO".to_string(), None, "-1".to_string(), None);
            assert_eq!(e.ty, "EGLint");
        }

        #[test]
        #[should_panic]
        fn test_unknown_type() {
            parse::make_egl_enum(
                "FOO".to_string(),
                Some("blargh".to_string()),
                String::new(),
                None,
            );
        }

        #[test]
        #[should_panic]
        fn test_unknown_value() {
            parse::make_egl_enum("FOO".to_string(), None, "a".to_string(), None);
        }

        #[test]
        #[should_panic]
        fn test_empty_value() {
            parse::make_egl_enum("FOO".to_string(), None, String::new(), None);
        }
    }

    mod parse_event {
        mod from_xml {
            use xml::attribute::OwnedAttribute;
            use xml::common::XmlVersion;
            use xml::name::OwnedName;
            use xml::namespace::Namespace;
            use xml::reader::XmlEvent;

            use registry::parse::{Attribute, ParseEvent};

            #[test]
            fn test_start_event() {
                let given = XmlEvent::StartElement {
                    name: OwnedName::local("element"),
                    attributes: vec![
                        OwnedAttribute::new(OwnedName::local("attr1"), "val1"),
                        OwnedAttribute::new(OwnedName::local("attr2"), "val2"),
                    ],
                    namespace: Namespace::empty(),
                };
                let expected = ParseEvent::Start(
                    "element".to_string(),
                    vec![
                        Attribute::new("attr1", "val1"),
                        Attribute::new("attr2", "val2"),
                    ],
                );
                assert_eq!(ParseEvent::from_xml(given), Some(expected));
            }

            #[test]
            fn test_end_element() {
                let given = XmlEvent::EndElement {
                    name: OwnedName::local("element"),
                };
                let expected = ParseEvent::End("element".to_string());
                assert_eq!(ParseEvent::from_xml(given), Some(expected));
            }

            #[test]
            fn test_characters() {
                let given = XmlEvent::Characters("text".to_string());
                let expected = ParseEvent::Text("text".to_string());
                assert_eq!(ParseEvent::from_xml(given), Some(expected));
            }

            #[test]
            fn test_start_document() {
                let given = XmlEvent::StartDocument {
                    version: XmlVersion::Version10,
                    encoding: "".to_string(),
                    standalone: None,
                };
                assert_eq!(ParseEvent::from_xml(given), None);
            }

            #[test]
            fn test_end_document() {
                let given = XmlEvent::EndDocument;
                assert_eq!(ParseEvent::from_xml(given), None);
            }

            #[test]
            fn test_processing_instruction() {
                let given = XmlEvent::ProcessingInstruction {
                    name: "".to_string(),
                    data: None,
                };
                assert_eq!(ParseEvent::from_xml(given), None);
            }

            #[test]
            fn test_cdata() {
                let given = XmlEvent::CData("CData".to_string());
                assert_eq!(ParseEvent::from_xml(given), None);
            }

            #[test]
            fn test_comment() {
                let given = XmlEvent::Comment("Comment".to_string());
                assert_eq!(ParseEvent::from_xml(given), None);
            }

            #[test]
            fn test_whitespace() {
                let given = XmlEvent::Whitespace("Whitespace".to_string());
                assert_eq!(ParseEvent::from_xml(given), None);
            }
        }
    }
}
