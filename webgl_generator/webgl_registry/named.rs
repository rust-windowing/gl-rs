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

use std::collections::{BTreeMap, BTreeSet};

use super::{Registry, Type};
use utils::{multimap_append, multimap_insert};

#[derive(Debug, Clone)]
pub enum NamedType {
    Mixin(Mixin),
    Interface(Interface),
    Dictionary(Dictionary),
    Enum(Enum),
    Typedef(Type),
    Callback(Callback),
}

#[derive(Debug, Clone)]
pub struct Mixin {
    pub members: BTreeMap<String, Vec<Member>>,
}

#[derive(Debug, Clone)]
pub struct Interface {
    pub inherits: Option<String>,
    pub mixins: BTreeSet<String>,
    pub members: BTreeMap<String, Vec<Member>>,
    pub is_hidden: bool,
    pub has_class: bool,
    pub rendering_context: Option<&'static str>,
    pub doc_comment: String,
}

#[derive(Debug, Clone)]
pub struct Dictionary {
    pub inherits: Option<String>,
    pub fields: BTreeMap<String, Field>,
    pub is_hidden: bool,
}

#[derive(Debug, Clone)]
pub struct Enum {
    pub variants: BTreeSet<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Callback {
    pub args: Vec<Argument>,
    pub return_type: Option<Type>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Const {
    pub type_: Type,
    pub value: String,
}

#[derive(Debug, Eq, Clone)]
pub struct Argument {
    pub name: String,
    pub optional: bool,
    pub type_: Type,
    pub variadic: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Operation {
    pub args: Vec<Argument>,
    pub return_type: Option<Type>,
    pub doc_comment: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Attribute {
    pub type_: Type,
    pub setter: bool,
    pub getter: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Member {
    Const(Const),
    Operation(Operation),
    Attribute(Attribute),
}

#[derive(Debug, Clone)]
pub struct Field {
    pub type_: Type,
}

#[derive(Debug)]
pub struct VisitOptions {
    pub visit_mixins: bool,
}

impl NamedType {
    pub fn as_mixin(&self) -> Option<&Mixin> {
        if let NamedType::Mixin(m) = self {
            Some(m)
        } else {
            None
        }
    }
    pub fn as_interface(&self) -> Option<&Interface> {
        if let NamedType::Interface(i) = self {
            Some(i)
        } else {
            None
        }
    }
    pub fn as_dictionary(&self) -> Option<&Dictionary> {
        if let NamedType::Dictionary(d) = self {
            Some(d)
        } else {
            None
        }
    }
    pub fn as_enum(&self) -> Option<&Enum> {
        if let NamedType::Enum(e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn as_typedef(&self) -> Option<&Type> {
        if let NamedType::Typedef(t) = self {
            Some(t)
        } else {
            None
        }
    }
    pub fn as_mixin_mut(&mut self) -> Option<&mut Mixin> {
        if let &mut NamedType::Mixin(ref mut m) = self {
            Some(m)
        } else {
            None
        }
    }
    pub fn as_interface_mut(&mut self) -> Option<&mut Interface> {
        if let &mut NamedType::Interface(ref mut i) = self {
            Some(i)
        } else {
            None
        }
    }
    pub fn as_dictionary_mut(&mut self) -> Option<&mut Dictionary> {
        if let &mut NamedType::Dictionary(ref mut d) = self {
            Some(d)
        } else {
            None
        }
    }
    pub fn as_enum_mut(&mut self) -> Option<&mut Enum> {
        if let &mut NamedType::Enum(ref mut e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn as_typedef_mut(&mut self) -> Option<&mut Type> {
        if let &mut NamedType::Typedef(ref mut t) = self {
            Some(t)
        } else {
            None
        }
    }
}

impl PartialEq for Argument {
    fn eq(&self, other: &Self) -> bool {
        self.type_ == other.type_
    }
}

impl Default for VisitOptions {
    fn default() -> Self {
        VisitOptions { visit_mixins: true }
    }
}

impl Dictionary {
    pub fn collect_fields<'a>(&'a self, registry: &'a Registry) -> BTreeMap<&'a str, &'a Field> {
        let mut fields = BTreeMap::new();

        // Inherits
        if let Some(inherit_name) = self.inherits.as_ref() {
            let inherit = registry
                .types
                .get(inherit_name)
                .and_then(NamedType::as_dictionary)
                .expect(inherit_name);
            fields.append(&mut inherit.collect_fields(registry));
        }

        // Fields
        for (name, field) in &self.fields {
            fields.insert(name, field);
        }

        fields
    }
}

impl Interface {
    pub fn collect_members<'a>(
        &'a self,
        registry: &'a Registry,
        options: &VisitOptions,
    ) -> BTreeMap<&'a str, Vec<&'a Member>> {
        let mut members = BTreeMap::new();

        // Mixins
        for mixin_name in &self.mixins {
            let mixin = registry
                .types
                .get(mixin_name)
                .and_then(NamedType::as_mixin)
                .expect(mixin_name);
            if options.visit_mixins {
                multimap_append(&mut members, mixin.collect_members(registry, options));
            }
        }

        // Members
        for (name, ms) in &self.members {
            for member in ms {
                multimap_insert(&mut members, &**name, member);
            }
        }

        members
    }
}

impl Mixin {
    pub fn collect_members<'a>(
        &'a self,
        _registry: &'a Registry,
        _options: &VisitOptions,
    ) -> BTreeMap<&'a str, Vec<&'a Member>> {
        let mut members = BTreeMap::new();

        // Members
        for (name, ms) in &self.members {
            for member in ms {
                multimap_insert(&mut members, &**name, member);
            }
        }

        members
    }
}
