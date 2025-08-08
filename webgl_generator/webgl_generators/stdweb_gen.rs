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

use std::collections::BTreeSet;
use std::io;

use utils::*;
use webgl_registry::*;

#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub struct StdwebGenerator;

#[derive(Clone, Debug)]
struct GenericContext {
    args: BTreeSet<String>,
    constraints: Vec<String>,
}

impl GenericContext {
    pub fn new() -> Self {
        GenericContext {
            args: BTreeSet::new(),
            constraints: Vec::new(),
        }
    }
    pub fn arg(&mut self, desired_name: &str) -> String {
        for i in 0.. {
            let name = format!("{}{}", desired_name, i);
            if !self.args.contains(&name) {
                self.args.insert(name.clone());
                return name;
            }
        }
        unreachable!()
    }
    pub fn constrain(&mut self, constraint: String) {
        self.constraints.push(constraint);
    }
    pub fn args(&self) -> String {
        if self.args.is_empty() {
            String::new()
        } else {
            let args: Vec<_> = self.args.iter().cloned().collect();
            format!("<{}>", args.join(", "))
        }
    }
    pub fn constraints(&self) -> String {
        if self.constraints.is_empty() {
            String::new()
        } else {
            format!(" where {}", self.constraints.join(", "))
        }
    }
}

#[derive(Clone, Debug)]
enum ArgWrapper {
    None,
    AsTypedArray,
    AsArrayBufferView,
    Optional(Box<ArgWrapper>),
    Sequence(Box<ArgWrapper>),
    DoubleCast,
    Once,
}

impl ArgWrapper {
    fn wrap(&self, arg: &str) -> String {
        match self {
            &ArgWrapper::None => arg.into(),
            &ArgWrapper::AsTypedArray => format!("unsafe {{ {}.as_typed_array() }}", arg),
            &ArgWrapper::AsArrayBufferView => {
                format!("unsafe {{ {}.as_array_buffer_view() }}", arg)
            },
            ArgWrapper::Optional(inner) => {
                format!("{}.map(|inner| {})", arg, inner.wrap("inner"))
            },
            ArgWrapper::Sequence(inner) => format!(
                "{}.iter().map(|inner| {}).collect::<Vec<_>>()",
                arg,
                inner.wrap("inner")
            ),
            &ArgWrapper::DoubleCast => format!("({} as f64)", arg),
            &ArgWrapper::Once => format!("Once({})", arg),
        }
    }
}

#[derive(Clone, Debug)]
struct ProcessedArg {
    type_: String,
    wrapper: ArgWrapper,
    optional: bool,
}

impl ProcessedArg {
    fn simple<S: Into<String>>(name: S) -> ProcessedArg {
        ProcessedArg {
            type_: name.into(),
            wrapper: ArgWrapper::None,
            optional: false,
        }
    }
}

fn process_arg_type_kind(
    type_kind: &TypeKind,
    registry: &Registry,
    gc: &mut GenericContext,
) -> ProcessedArg {
    let (name, flat_kind) = type_kind.flatten(registry);
    match flat_kind {
        TypeKind::Primitive(p) => match *p {
            Primitive::I64 => ProcessedArg {
                type_: name.unwrap().into(),
                wrapper: ArgWrapper::DoubleCast,
                optional: false,
            },
            Primitive::U64 => ProcessedArg {
                type_: name.unwrap().into(),
                wrapper: ArgWrapper::DoubleCast,
                optional: false,
            },
            _ => ProcessedArg::simple(name.unwrap()),
        },
        &TypeKind::String => ProcessedArg::simple("&str"),
        &TypeKind::ArrayBuffer => ProcessedArg::simple("&ArrayBuffer"),
        &TypeKind::BufferSource => ProcessedArg::simple("&ArrayBuffer"),
        &TypeKind::CanvasElement => ProcessedArg::simple("&CanvasElement"),
        TypeKind::TypedArray(p) => {
            let lt = gc.arg("'a");
            let gp = gc.arg("T");
            gc.constrain(format!("{}: AsTypedArray<{}, {}>", gp, lt, p.name()));
            ProcessedArg {
                type_: gp,
                wrapper: ArgWrapper::AsTypedArray,
                optional: false,
            }
        },
        &TypeKind::ArrayBufferView => {
            let lt = gc.arg("'a");
            let gp = gc.arg("T");
            gc.constrain(format!("{}: AsArrayBufferView<{}>", gp, lt));
            ProcessedArg {
                type_: gp,
                wrapper: ArgWrapper::AsArrayBufferView,
                optional: false,
            }
        },
        TypeKind::Sequence(t) => {
            let inner = process_arg_type(t, registry, gc);
            ProcessedArg {
                type_: format!("&[{}]", inner.type_),
                wrapper: match inner.wrapper {
                    ArgWrapper::None => ArgWrapper::None,
                    other => ArgWrapper::Sequence(Box::new(other)),
                },
                optional: false,
            }
        },
        TypeKind::Union(ts) => {
            let t = ts
                .iter()
                .filter_map(|t| match t.kind {
                    TypeKind::TypedArray(_) => Some(t),
                    TypeKind::Sequence(_) => None,
                    _ => panic!("Union support is limited!"),
                })
                .next()
                .expect("Union did not contain a TypedArray");

            process_arg_type(t, registry, gc)
        },
        TypeKind::Named(actual_name) => {
            match registry.resolve_type(actual_name) {
                &NamedType::Dictionary(_) | &NamedType::Interface(_) => {
                    ProcessedArg::simple(format!("&{}", name.unwrap()))
                },
                &NamedType::Enum(_) => ProcessedArg::simple(name.unwrap()),
                NamedType::Typedef(t) => {
                    // We have to "look through" the typedef, as the correct parameter
                    // type is not representable using the alias.
                    assert!(t.optional);
                    process_arg_type(t, registry, gc)
                },
                &NamedType::Callback(_) => {
                    let gp = gc.arg("F");
                    gc.constrain(format!("{}: FnOnce() + 'static", gp));
                    ProcessedArg {
                        type_: gp,
                        wrapper: ArgWrapper::Once,
                        optional: false,
                    }
                },
                &NamedType::Mixin(_) => panic!("Mixins are not usable as types!"),
            }
        },
        &TypeKind::Any | &TypeKind::Object => {
            let gp = gc.arg("T");
            gc.constrain(format!("{}: JsSerialize", gp));
            ProcessedArg::simple(gp)
        },
    }
}

fn process_arg_type(type_: &Type, registry: &Registry, gc: &mut GenericContext) -> ProcessedArg {
    let mut result = process_arg_type_kind(&type_.kind, registry, gc);
    if type_.optional && !result.optional {
        result.type_ = format!("Option<{}>", result.type_);
        result.wrapper = match result.wrapper {
            ArgWrapper::None => ArgWrapper::None,
            other => ArgWrapper::Optional(Box::new(other)),
        };
        result.optional = true;
    }
    result
}

#[derive(Clone, Debug)]
enum ResultWrapper {
    TryInto,
    Ok,
}

impl ResultWrapper {
    fn wrap(&self, content: &str) -> String {
        match *self {
            ResultWrapper::TryInto => format!("{}.try_into().unwrap()", content),
            ResultWrapper::Ok => format!("{}.try_into().ok()", content),
        }
    }
}

#[derive(Clone, Debug)]
struct ProcessedResult {
    type_: String,
    wrapper: ResultWrapper,
    optional: bool,
}

impl ProcessedResult {
    fn simple<S: Into<String>>(name: S) -> ProcessedResult {
        ProcessedResult {
            type_: name.into(),
            wrapper: ResultWrapper::TryInto,
            optional: false,
        }
    }
}

fn process_result_type_kind(type_kind: &TypeKind, registry: &Registry) -> ProcessedResult {
    match type_kind {
        TypeKind::Primitive(p) => ProcessedResult::simple(p.name()),
        &TypeKind::String => ProcessedResult::simple("String"),
        &TypeKind::ArrayBuffer | &TypeKind::ArrayBufferView => {
            ProcessedResult::simple("ArrayBuffer")
        },
        &TypeKind::BufferSource => unimplemented!("BufferSource not supported in output"),
        &TypeKind::CanvasElement => ProcessedResult::simple("CanvasElement"),
        TypeKind::TypedArray(p) => {
            ProcessedResult::simple(format!("TypedArray<{}>", p.name()))
        },
        TypeKind::Sequence(t) => {
            let inner = process_result_type(t, registry);
            ProcessedResult::simple(format!("Vec<{}>", inner.type_))
        },
        TypeKind::Union(ts) => {
            let t = ts
                .iter()
                .filter_map(|t| match t.kind {
                    TypeKind::TypedArray(_) => Some(t),
                    TypeKind::Sequence(_) => None,
                    _ => panic!("Union support is limited!"),
                })
                .next()
                .expect("Union did not contain a TypedArray");

            process_result_type(t, registry)
        },
        TypeKind::Named(name) => match registry.resolve_type(name) {
            &NamedType::Dictionary(_) | &NamedType::Interface(_) | &NamedType::Enum(_) => {
                ProcessedResult::simple(name.as_str())
            },
            NamedType::Typedef(t) => {
                let inner = process_result_type(t, registry);
                ProcessedResult {
                    type_: name.clone(),
                    wrapper: inner.wrapper.clone(),
                    optional: inner.optional,
                }
            },
            &NamedType::Callback(_) => unimplemented!(),
            &NamedType::Mixin(_) => panic!("Mixins are not usable as types!"),
        },
        &TypeKind::Any | &TypeKind::Object => ProcessedResult::simple("Value"),
    }
}

fn process_result_type(type_: &Type, registry: &Registry) -> ProcessedResult {
    let mut result = process_result_type_kind(&type_.kind, registry);
    if type_.optional && !result.optional {
        result.type_ = format!("Option<{}>", result.type_);
        result.wrapper = ResultWrapper::Ok;
        result.optional = true;
    }
    result
}

fn write_header<W>(registry: &Registry, dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    writeln!(
        dest,
        r#"
// {registry:?}
extern crate stdweb;

use self::stdweb::{{Reference, Value, UnsafeTypedArray, JsSerialize, InstanceOf}};
use self::stdweb::unstable::{{TryFrom, TryInto}};
use self::stdweb::web::{{RenderingContext, TypedArray, ArrayBuffer}};
use self::stdweb::web::html_element::CanvasElement;

type ConversionError = <Reference as TryFrom<Value>>::Error;

pub trait AsTypedArray<'a, T> {{
    type Result: JsSerialize;

    unsafe fn as_typed_array(self) -> Self::Result;
}}

pub trait AsArrayBufferView<'a> {{
    type Result: JsSerialize;

    unsafe fn as_array_buffer_view(self) -> Self::Result;
}}

pub trait Extension: TryFrom<Value> {{
    const NAME: &'static str;
}}

macro_rules! define_array {{
    ($elem:ty) => {{
        impl<'a> AsTypedArray<'a, $elem> for &'a TypedArray<$elem> {{
            type Result = Self;

            unsafe fn as_typed_array(self) -> Self::Result {{ self }}
        }}

        impl<'a> AsTypedArray<'a, $elem> for &'a [$elem] {{
            type Result = UnsafeTypedArray<'a, $elem>;

            unsafe fn as_typed_array(self) -> Self::Result {{ UnsafeTypedArray::new(self) }}
        }}

        impl<'a> AsArrayBufferView<'a> for &'a TypedArray<$elem> {{
            type Result = Self;

            unsafe fn as_array_buffer_view(self) -> Self::Result {{ self }}
        }}

        impl<'a> AsArrayBufferView<'a> for &'a [$elem] {{
            type Result = UnsafeTypedArray<'a, $elem>;

            unsafe fn as_array_buffer_view(self) -> Self::Result {{ UnsafeTypedArray::new(self) }}
        }}
    }}
}}

define_array!(i8);
define_array!(u8);
define_array!(i16);
define_array!(u16);
define_array!(i32);
define_array!(u32);
define_array!(f32);
define_array!(f64);
"#,
        registry = registry
    )?;
    Ok(())
}

impl super::Generator for StdwebGenerator {
    fn write<W>(&self, registry: &Registry, dest: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        write_header(registry, dest)?;
        write_typedefs(registry, dest)?;
        write_enums(registry, dest)?;
        write_dictionaries(registry, dest)?;
        write_interfaces(registry, dest)?;
        write_extensions(registry, dest)?;
        Ok(())
    }
}

fn write_typedefs<W>(registry: &Registry, dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    for (name, t) in registry.iter_types(NamedType::as_typedef) {
        write_typedef(name, t, registry, dest)?;
    }
    Ok(())
}

fn write_typedef<W>(name: &str, type_: &Type, registry: &Registry, dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    writeln!(
        dest,
        r#"#[allow(dead_code)] pub type {name} = {type_};"#,
        name = name,
        type_ = process_result_type(type_, registry).type_
    )?;
    Ok(())
}

fn write_enums<W>(registry: &Registry, dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    for (name, enum_) in registry.iter_types(NamedType::as_enum) {
        write_enum(name, enum_, registry, dest)?;
    }
    Ok(())
}

fn write_enum<W>(name: &str, enum_: &Enum, _registry: &Registry, dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    write!(
        dest,
        r#"
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum {name} {{
    "#,
        name = name
    )?;

    for variant in &enum_.variants {
        writeln!(
            dest,
            r#"
    #[serde(rename = "{raw_variant}")]
    {variant},"#,
            variant = camel(variant),
            raw_variant = variant
        )?;
    }

    writeln!(
        dest,
        r#"
}}
js_deserializable!({name});
js_serializable!({name});
    "#,
        name = name
    )?;
    Ok(())
}

fn write_dictionaries<W>(registry: &Registry, dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    for (name, dictionary) in registry.iter_types(NamedType::as_dictionary) {
        write_dictionary(name, dictionary, registry, dest)?;
    }
    Ok(())
}

fn write_dictionary<W>(
    name: &str,
    dictionary: &Dictionary,
    registry: &Registry,
    dest: &mut W,
) -> io::Result<()>
where
    W: io::Write,
{
    if dictionary.is_hidden {
        return Ok(());
    }

    write!(
        dest,
        r#"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {name} {{
    "#,
        name = name
    )?;

    for (name, field) in dictionary.collect_fields(registry) {
        write_field(name, field, registry, dest)?;
    }

    writeln!(
        dest,
        r#"
}}
js_deserializable!({name});
js_serializable!({name});
    "#,
        name = name
    )?;
    Ok(())
}

fn write_field<W>(name: &str, field: &Field, registry: &Registry, dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    let mut serde_attrs = Vec::new();
    let field_name = unreserve(snake(name));
    if field_name != name {
        serde_attrs.push(format!(r#"rename = "{}""#, name));
    }
    let field_type = process_result_type(&field.type_, registry);
    if field_type.optional {
        serde_attrs.push(r#"default"#.into());
        serde_attrs.push(r#"skip_serializing_if = "Option::is_none""#.into());
    }

    if !serde_attrs.is_empty() {
        write!(
            dest,
            r#"
    #[serde({})]"#,
            serde_attrs.join(", ")
        )?;
    }

    writeln!(
        dest,
        r#"
    {name}: {type_},"#,
        name = field_name,
        type_ = field_type.type_
    )?;

    Ok(())
}

fn write_interfaces<W>(registry: &Registry, dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    for (name, interface) in registry.iter_types(NamedType::as_interface) {
        write_interface(name, interface, registry, dest)?;
    }
    Ok(())
}

fn write_interface<W>(
    name: &str,
    interface: &Interface,
    registry: &Registry,
    dest: &mut W,
) -> io::Result<()>
where
    W: io::Write,
{
    if interface.is_hidden {
        return Ok(());
    }

    let mut attrs = String::new();
    let custom_instance_check = if name == "GLContext" {
        Some((
            "reference",
            "[WebGLRenderingContext, WebGL2RenderingContext].includes(@{{reference}}.constructor)"
                .into(),
        ))
    } else if interface.has_class {
        attrs += &format!("#[reference(instance_of = {:?})]\n", name);
        None
    } else {
        Some(("_reference", "true".to_owned()))
    };

    write!(
        dest,
        r#"
{doc_comment}#[derive(Debug, Clone, ReferenceType)]
{attrs}pub struct {name}(Reference);

impl {name} {{
"#,
        name = name,
        attrs = attrs,
        doc_comment = interface.doc_comment
    )?;

    for (name, members) in interface.collect_members(registry, &VisitOptions::default()) {
        for (index, member) in members.into_iter().enumerate() {
            match member {
                Member::Const(const_) => {
                    assert!(index == 0);
                    write_const(name, const_, registry, dest)?;
                },
                Member::Attribute(attribute) => {
                    assert!(index == 0);
                    write_attribute(name, attribute, registry, dest)?;
                },
                Member::Operation(operation) => {
                    write_operation(name, index, operation, registry, dest)?;
                },
            }
        }
    }

    writeln!(
        dest,
        r#"
}}
"#
    )?;

    if let Some((param_name, instance_check)) = custom_instance_check {
        write!(
            dest,
            r#"
impl InstanceOf for {name} {{
    #[inline]
    fn instance_of({param_name}: &Reference) -> bool {{
        js!(
            return {instance_check};
        ).try_into().unwrap()
    }}
}}
"#,
            param_name = param_name,
            name = name,
            instance_check = instance_check
        )?;
    }

    if let Some(rendering_context) = interface.rendering_context {
        writeln!(
            dest,
            r#"impl RenderingContext for {name} {{
    type Error = ConversionError;
    fn from_canvas(canvas: &CanvasElement) -> Result<Self, ConversionError> {{
        js!(
            return @{{canvas}}.getContext("{rendering_context}");
        ).try_into()
    }}
}}
"#,
            name = name,
            rendering_context = rendering_context
        )?;
    }

    Ok(())
}

fn write_const<W>(name: &str, const_: &Const, registry: &Registry, dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    let const_type = process_result_type(&const_.type_, registry);
    write!(
        dest,
        r#"
    pub const {name}: {type_} = {value};"#,
        name = shouty_snake(name),
        type_ = const_type.type_,
        value = const_.value
    )?;
    Ok(())
}

fn write_attribute<W>(
    name: &str,
    attribute: &Attribute,
    registry: &Registry,
    dest: &mut W,
) -> io::Result<()>
where
    W: io::Write,
{
    if attribute.getter {
        let result_type = process_result_type(&attribute.type_, registry);
        let expr = result_type.wrapper.wrap(&format!(
            "(js! {{ return @{{self}}.{raw_name}; }} )",
            raw_name = name
        ));

        write!(
            dest,
            r#"

    pub fn {name}(&self) -> {type_} {{
        {expr}
    }}"#,
            name = unreserve(snake(name)),
            type_ = result_type.type_,
            expr = expr
        )?;
    }
    if attribute.setter {
        let mut gc = GenericContext::new();
        let arg_type = process_arg_type(&attribute.type_, registry, &mut gc);
        write!(
            dest,
            r#"

    pub fn set_{name}{gargs}(&self, value: {type_}){gwhere} {{
        js!( @(no_return) @{{self}}.{raw_name} = @{{{value}}}; );
    }}"#,
            name = snake(name),
            raw_name = name,
            type_ = arg_type.type_,
            gargs = gc.args(),
            gwhere = gc.constraints(),
            value = arg_type.wrapper.wrap("value")
        )?;
    }
    Ok(())
}

fn write_get_extension<W>(dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    write!(
        dest,
        r#"

    pub fn get_extension<E: Extension>(&self) -> Option<E> {{
        (js! {{ return @{{self}}.getExtension(@{{E::NAME}}); }} ).try_into().ok()
    }}"#
    )
}

fn write_operation<W>(
    name: &str,
    index: usize,
    operation: &Operation,
    registry: &Registry,
    dest: &mut W,
) -> io::Result<()>
where
    W: io::Write,
{
    if name == "getExtension" { return write_get_extension(dest) }

    let mut rust_name = unreserve(snake(name));
    if index > 0 {
        rust_name = format!("{}_{}", rust_name, index);
    }

    let mut gc = GenericContext::new();

    struct OperationArg {
        arg: String,
        js_arg: String,
    }

    let args: Vec<_> = operation
        .args
        .iter()
        .map(|a| {
            let processed = process_arg_type(&a.type_, registry, &mut gc);
            let arg = format!("{}: {}", unreserve(snake(&a.name)), processed.type_);
            let js_arg = format!(
                "@{{{}}}",
                processed.wrapper.wrap(&unreserve(snake(&a.name)))
            );
            OperationArg { arg, js_arg }
        })
        .collect();

    let rust_args = args
        .iter()
        .map(|a| a.arg.clone())
        .collect::<Vec<_>>()
        .join(", ");
    let js_args = args
        .iter()
        .map(|a| a.js_arg.clone())
        .collect::<Vec<_>>()
        .join(", ");

    if let Some(return_type) = operation.return_type.as_ref() {
        let result_type = process_result_type(return_type, registry);
        let expr = result_type.wrapper.wrap(&format!(
            "(js! {{ return @{{self}}.{raw_name}({js_args}); }} )",
            raw_name = name,
            js_args = js_args
        ));

        write!(
            dest,
            r#"

    {doc_comment}pub fn {name}{gargs}(&self, {args}) -> {return_type}{gwhere} {{
        {expr}
    }}"#,
            name = rust_name,
            gargs = gc.args(),
            args = rust_args,
            return_type = result_type.type_,
            gwhere = gc.constraints(),
            expr = expr,
            doc_comment = operation.doc_comment
        )?;
    } else {
        write!(
            dest,
            r#"

    {doc_comment}pub fn {name}{gargs}(&self, {args}){gwhere} {{
        js!( @(no_return) @{{self}}.{raw_name}({js_args}); );
    }}"#,
            name = rust_name,
            raw_name = name,
            gargs = gc.args(),
            args = rust_args,
            gwhere = gc.constraints(),
            js_args = js_args,
            doc_comment = operation.doc_comment
        )?;
    }
    Ok(())
}

fn write_extensions<W>(registry: &Registry, dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    for name in &registry.extensions {
        write_extension(name, registry, dest)?;
    }
    Ok(())
}

fn write_extension<W>(name: &str, _registry: &Registry, dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    writeln!(
        dest,
        r#"
impl Extension for {name} {{
    const NAME: &'static str = "{name}";
}}"#,
        name = name
    )
}
