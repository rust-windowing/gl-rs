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

use std::collections::btree_map::{self, Entry};
use std::collections::{BTreeMap, BTreeSet};
use std::io;

use webidl::ast;

use utils::{multimap_insert, parse_defs};
use webgl_generators::Generator;

use super::named::{
    Argument, Attribute, Callback, Const, Dictionary, Enum, Field, Interface, Member, Mixin,
    NamedType, Operation,
};
use super::types::{Primitive, Type, TypeKind};
use super::{Api, Exts, HIDDEN_NAMES, RENDERING_CONTEXTS};

#[derive(Debug, Default)]
pub struct Registry {
    pub types: BTreeMap<String, NamedType>,
    pub extensions: BTreeSet<String>,
}

/// Helper for iterating over specific kinds of named type
pub struct TypeIter<'a, T: 'a, F: FnMut(&'a NamedType) -> Option<&'a T>> {
    inner: btree_map::Iter<'a, String, NamedType>,
    f: F,
}

impl<'a, T: 'a, F: FnMut(&'a NamedType) -> Option<&'a T>> Iterator for TypeIter<'a, T, F> {
    type Item = (&'a String, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        // Implement a variation of `flat_map`
        for (k, v) in self.inner.by_ref() {
            if let Some(t) = (self.f)(v) {
                return Some((k, t));
            }
        }
        None
    }
}

impl Registry {
    /// Construct a new registry given a maximum API version
    /// and a set of extensions to support.
    pub fn new(api: Api, exts: Exts) -> Registry {
        let mut result = Registry::default();

        // First load definitions from all API versions
        // up to the one requested.
        for idl_const in api.idl_consts() {
            for def in parse_defs(idl_const) {
                result.load_definition(def);
            }
        }

        // Next find all requested extensions
        for ext in exts.enumerate() {
            if api < ext.min_api {
                continue;
            }

            // Make a note that we included this extension
            result.extensions.insert(ext.name.clone());

            // Load the definitions
            for def in parse_defs(ext.idl.as_bytes()) {
                result.load_definition(def);
            }

            // Attach overview doc comment
            let ext_iface = result
                .types
                .get_mut(&ext.name)
                .and_then(NamedType::as_interface_mut)
                .expect(&ext.name);
            ext_iface.doc_comment = ext.overview;

            // Attach individual function doc comments
            for (name, new_fun) in ext.new_funs {
                let members = ext_iface.members.get_mut(&name).unwrap();
                for member in members {
                    if let Member::Operation(ref mut op) = *member {
                        op.doc_comment = new_fun.clone();
                    }
                }
            }
        }

        // Find the latest version of the rendering context, and generate a
        // version-independent rendering context helper type called "GLContext".
        for name in RENDERING_CONTEXTS.iter().rev() {
            if let Some(NamedType::Interface(mut iface)) = result.types.get(name.1).cloned() {
                iface.rendering_context = None;
                result
                    .types
                    .insert("GLContext".into(), NamedType::Interface(iface));
                break;
            }
        }

        // Hide types that are listed in our "hidden names" array
        for &hidden_name in HIDDEN_NAMES {
            if let Some(interface) = result
                .types
                .get_mut(hidden_name)
                .and_then(NamedType::as_interface_mut)
            {
                interface.is_hidden = true;
            }
            if let Some(dictionary) = result
                .types
                .get_mut(hidden_name)
                .and_then(NamedType::as_dictionary_mut)
            {
                dictionary.is_hidden = true;
            }
        }

        // Done!
        result
    }

    /// Iterator over types matched by a filtering function
    pub fn iter_types<'a, T, F: FnMut(&'a NamedType) -> Option<&'a T>>(
        &'a self,
        f: F,
    ) -> TypeIter<'a, T, F> {
        TypeIter {
            inner: self.types.iter(),
            f,
        }
    }

    /// Resolves a named type
    pub fn resolve_type(&self, name: &str) -> &NamedType {
        self.types.get(name).expect(name)
    }

    /// Use the specified generator to generate bindings from this registry
    /// and write them to a stream.
    pub fn write_bindings<W, G>(&self, generator: G, output: &mut W) -> io::Result<()>
    where
        G: Generator,
        W: io::Write,
    {
        generator.write(self, output)
    }

    fn load_const(&mut self, const_: ast::Const) -> Option<(String, Member)> {
        use self::ast::{ConstType, ConstValue};

        let type_ = ast::Type {
            extended_attributes: Vec::new(),
            kind: match const_.type_ {
                ConstType::Boolean => ast::TypeKind::Boolean,
                ConstType::Byte => ast::TypeKind::Byte,
                ConstType::Octet => ast::TypeKind::Octet,
                ConstType::RestrictedDouble => ast::TypeKind::RestrictedDouble,
                ConstType::UnrestrictedDouble => ast::TypeKind::UnrestrictedDouble,
                ConstType::RestrictedFloat => ast::TypeKind::RestrictedFloat,
                ConstType::UnrestrictedFloat => ast::TypeKind::UnrestrictedFloat,
                ConstType::SignedLong => ast::TypeKind::SignedLong,
                ConstType::UnsignedLong => ast::TypeKind::UnsignedLong,
                ConstType::SignedLongLong => ast::TypeKind::SignedLongLong,
                ConstType::UnsignedLongLong => ast::TypeKind::UnsignedLongLong,
                ConstType::SignedShort => ast::TypeKind::SignedShort,
                ConstType::UnsignedShort => ast::TypeKind::UnsignedShort,
                ConstType::Identifier(s) => ast::TypeKind::Identifier(s),
            },
            nullable: const_.nullable,
        };

        Some((
            const_.name,
            Member::Const(Const {
                type_: self.load_type(type_),
                value: match const_.value {
                    ConstValue::BooleanLiteral(b) => format!("{:?}", b),
                    ConstValue::FloatLiteral(v) => format!("{:?}", v),
                    ConstValue::SignedIntegerLiteral(v) => format!("{:?}", v),
                    ConstValue::UnsignedIntegerLiteral(v) => format!("{:?}", v),
                    ConstValue::Null => "None".into(),
                },
            }),
        ))
    }

    fn load_attribute(&mut self, attribute: ast::Attribute) -> Option<(String, Member)> {
        use self::ast::Attribute::*;
        match attribute {
            Regular(a) => {
                let type_ = self.load_type(a.type_);
                Some((
                    a.name,
                    Member::Attribute(Attribute {
                        type_,
                        setter: !a.read_only,
                        getter: !a.inherits,
                    }),
                ))
            },
            _ => None,
        }
    }

    fn load_argument(&mut self, argument: ast::Argument) -> Argument {
        let type_ = self.load_type(argument.type_);
        Argument {
            name: argument.name,
            optional: argument.optional,
            type_,
            variadic: argument.variadic,
        }
    }

    fn load_operation(&mut self, operation: ast::Operation) -> Option<(String, Member)> {
        use self::ast::Operation::*;
        use self::ast::ReturnType;
        match operation {
            Regular(o) => {
                if let Some(name) = o.name {
                    Some((
                        name,
                        Member::Operation(Operation {
                            args: o
                                .arguments
                                .into_iter()
                                .map(|a| self.load_argument(a))
                                .collect(),
                            return_type: match o.return_type {
                                ReturnType::NonVoid(t) => Some(self.load_type(t)),
                                ReturnType::Void => None,
                            },
                            doc_comment: String::new(),
                        }),
                    ))
                } else {
                    None
                }
            },
            _ => None,
        }
    }

    fn load_mixin_member(&mut self, member: ast::MixinMember) -> Option<(String, Member)> {
        use self::ast::MixinMember::*;
        match member {
            Const(c) => self.load_const(c),
            Attribute(a) => self.load_attribute(a),
            Operation(o) => self.load_operation(o),
        }
    }

    fn load_mixin(&mut self, mixin: ast::NonPartialMixin) {
        let mut members = BTreeMap::new();
        for (name, member) in mixin
            .members
            .into_iter()
            .flat_map(|m| self.load_mixin_member(m))
        {
            multimap_insert(&mut members, name, member);
        }

        self.load_named_type(&mixin.name, NamedType::Mixin(Mixin { members }));
    }

    fn load_interface_member(&mut self, member: ast::InterfaceMember) -> Option<(String, Member)> {
        use self::ast::InterfaceMember::*;
        match member {
            Const(c) => self.load_const(c),
            Attribute(a) => self.load_attribute(a),
            Operation(o) => self.load_operation(o),
            _ => None,
        }
    }

    fn load_interface(&mut self, interface: ast::NonPartialInterface) {
        fn has_attr(attrs: &Vec<ast::ExtendedAttribute>, name: &str) -> bool {
            use self::ast::ExtendedAttribute::*;
            for attr in attrs {
                if let NoArguments(ref other) = *attr {
                    if let ast::Other::Identifier(n) = other {
                        return n == name;
                    }
                }
            }
            false
        }

        let mut members = BTreeMap::new();
        for (name, member) in interface
            .members
            .into_iter()
            .flat_map(|m| self.load_interface_member(m))
        {
            multimap_insert(&mut members, name, member);
        }

        let mut result = Interface {
            inherits: interface.inherits,
            mixins: BTreeSet::new(),
            members,
            is_hidden: false,
            has_class: !has_attr(&interface.extended_attributes, "NoInterfaceObject"),
            rendering_context: None,
            doc_comment: String::new(),
        };

        for &(context_id, context_interface) in RENDERING_CONTEXTS {
            if context_interface == interface.name {
                result.rendering_context = Some(context_id);
                break;
            }
        }

        match self.types.entry(interface.name) {
            Entry::Vacant(v) => {
                v.insert(NamedType::Interface(result));
            },
            Entry::Occupied(o) => {
                assert!(
                    result.members.is_empty(),
                    "Duplicate interface: {}",
                    o.key()
                );
            },
        }
    }

    fn load_includes(&mut self, includes: ast::Includes) {
        if let Some(interface) = self
            .types
            .get_mut(&includes.includer)
            .and_then(NamedType::as_interface_mut)
        {
            interface.mixins.insert(includes.includee);
        }
    }

    // Load a type kind into the registry under a given name, and return
    // a reference to it.
    fn load_named_type(&mut self, name: &str, named_type: NamedType) -> Type {
        if !self.types.contains_key(name) {
            self.types.insert(name.into(), named_type);
        }
        name.into()
    }

    // Convert an AST type kind into a type kind the generator can understand
    fn load_type_kind(&mut self, kind: ast::TypeKind) -> TypeKind {
        use self::ast::TypeKind::*;

        match kind {
            // Primitives
            Boolean => TypeKind::Primitive(Primitive::Bool),
            Byte => TypeKind::Primitive(Primitive::I8),
            Octet => TypeKind::Primitive(Primitive::U8),
            SignedShort => TypeKind::Primitive(Primitive::I16),
            UnsignedShort => TypeKind::Primitive(Primitive::U16),
            SignedLong => TypeKind::Primitive(Primitive::I32),
            UnsignedLong => TypeKind::Primitive(Primitive::U32),
            SignedLongLong => TypeKind::Primitive(Primitive::I64),
            UnsignedLongLong => TypeKind::Primitive(Primitive::U64),
            RestrictedFloat | UnrestrictedFloat => TypeKind::Primitive(Primitive::F32),
            RestrictedDouble | UnrestrictedDouble => TypeKind::Primitive(Primitive::F64),

            // Strings
            DOMString | USVString => TypeKind::String,
            ByteString => unimplemented!(),

            // TypedArrays
            Int8Array => TypeKind::TypedArray(Primitive::I8),
            Uint8Array => TypeKind::TypedArray(Primitive::U8),
            Int16Array => TypeKind::TypedArray(Primitive::I16),
            Uint16Array => TypeKind::TypedArray(Primitive::U16),
            Int32Array => TypeKind::TypedArray(Primitive::I32),
            Uint32Array => TypeKind::TypedArray(Primitive::U32),
            Float32Array => TypeKind::TypedArray(Primitive::F32),
            Float64Array => TypeKind::TypedArray(Primitive::F64),

            // Sequence
            Sequence(inner) => TypeKind::Sequence(Box::new(self.load_type(*inner))),

            // Identifier
            Identifier(s) => match &*s {
                "BufferSource" => TypeKind::BufferSource,
                "HTMLCanvasElement" => TypeKind::CanvasElement,
                "ArrayBufferView" => TypeKind::ArrayBufferView,
                other => TypeKind::Named(other.into()),
            },

            // Composite
            Union(inners) => {
                if inners.len() <= 2 {
                    TypeKind::Union(
                        inners
                            .into_iter()
                            .map(|inner| self.load_type(inner))
                            .collect(),
                    )
                } else {
                    TypeKind::Any
                }
            },

            // Misc
            ArrayBuffer => TypeKind::ArrayBuffer,
            Object => TypeKind::Object,
            _ => TypeKind::Any,
        }
    }

    // Convert an AST type into a type the generator can understand
    fn load_type(&mut self, t: ast::Type) -> Type {
        Type {
            kind: self.load_type_kind(t.kind),
            optional: t.nullable,
        }
    }

    fn load_typedef(&mut self, typedef: ast::Typedef) {
        let type_ = self.load_type(typedef.type_);
        self.load_named_type(&typedef.name, NamedType::Typedef(type_));
    }

    fn load_field(&mut self, field: ast::DictionaryMember) -> (String, Field) {
        let type_ = self.load_type(field.type_);

        (field.name, Field { type_ })
    }

    fn load_dictionary(&mut self, dictionary: ast::NonPartialDictionary) {
        let fields = dictionary
            .members
            .into_iter()
            .map(|m| self.load_field(m))
            .collect();

        match self.types.entry(dictionary.name) {
            Entry::Vacant(v) => {
                v.insert(NamedType::Dictionary(Dictionary {
                    inherits: dictionary.inherits,
                    fields,
                    is_hidden: false,
                }));
            },
            Entry::Occupied(mut o) => {
                let key = o.key().clone();
                let d = o.get_mut().as_dictionary_mut().unwrap();
                // Dictionary is being extended, so make these fields all optional
                for (k, mut field) in fields {
                    field.type_.optional = true;
                    if d.fields.insert(k.clone(), field).is_some() {
                        panic!("Duplicate field: {}.{}", key, k);
                    }
                }
            },
        }
    }

    fn load_enum(&mut self, enum_: ast::Enum) {
        let variants = enum_.variants.into_iter().collect();
        self.load_named_type(&enum_.name, NamedType::Enum(Enum { variants }));
    }

    fn load_callback(&mut self, callback: ast::Callback) {
        use self::ast::ReturnType;
        let args = callback
            .arguments
            .into_iter()
            .map(|a| self.load_argument(a))
            .collect();
        let return_type = match callback.return_type {
            ReturnType::NonVoid(t) => Some(self.load_type(t)),
            ReturnType::Void => None,
        };
        self.load_named_type(
            &callback.name,
            NamedType::Callback(Callback { args, return_type }),
        );
    }

    fn load_definition(&mut self, def: ast::Definition) {
        use self::ast::Definition::*;
        match def {
            Mixin(ast::Mixin::NonPartial(m)) => self.load_mixin(m),
            Interface(ast::Interface::NonPartial(i)) => self.load_interface(i),
            Includes(i) => self.load_includes(i),
            Typedef(t) => self.load_typedef(t),
            Dictionary(ast::Dictionary::NonPartial(d)) => self.load_dictionary(d),
            Enum(e) => self.load_enum(e),
            Callback(c) => self.load_callback(c),
            _ => {},
        }
    }
}
