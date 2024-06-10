//! Code to help generate functions.
//!
//! The structure is:
//!
//! - [`Generator`]
//!   - `.impl_for()`: [`ImplFor`]
//!     - `.generate_fn()`: [`FnBuilder`]
//!       - `.body(|builder| { .. })`: [`StreamBuilder`]
//!
//! Afterwards, [`Generator::finish()`] **must** be called to take out the [`TokenStream`] produced.
//!
//! [`Generator::finish()`]: struct.Generator.html#method.finish
//! [`TokenStream`]: ../prelude/struct.TokenStream.html

mod gen_enum;
mod gen_struct;
mod generate_item;
mod generate_mod;
mod generator;
mod r#impl;
mod impl_for;
mod stream_builder;

use crate::parse::Visibility;
use crate::{
    parse::{GenericConstraints, Generics},
    prelude::{Delimiter, Ident},
};
use std::fmt;

pub use self::gen_enum::GenEnum;
pub use self::gen_struct::GenStruct;
pub use self::generate_item::{FnBuilder, FnSelfArg, GenConst};
pub use self::generate_mod::GenerateMod;
pub use self::generator::Generator;
pub use self::impl_for::ImplFor;
pub use self::r#impl::Impl;
pub use self::stream_builder::{PushParseError, StreamBuilder};

/// Helper trait to make it possible to nest several builders. Internal use only.
#[allow(missing_docs)]
pub trait Parent {
    fn append(&mut self, builder: StreamBuilder);
    fn name(&self) -> &Ident;
    fn generics(&self) -> Option<&Generics>;
    fn generic_constraints(&self) -> Option<&GenericConstraints>;
}

/// Helper enum to differentiate between a [`Ident`] or a [`String`].
#[allow(missing_docs)]
pub enum StringOrIdent {
    String(String),
    // Note that when this is a `string` this could be much more than a single ident.
    // Therefor you should never use [`StreamBuilder`]`.ident_str(StringOrIdent.to_string())`, but instead use `.push_parsed(StringOrIdent.to_string())?`.
    Ident(Ident),
}

impl fmt::Display for StringOrIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(s) => s.fmt(f),
            Self::Ident(i) => i.fmt(f),
        }
    }
}

impl From<String> for StringOrIdent {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}
impl From<Ident> for StringOrIdent {
    fn from(i: Ident) -> Self {
        Self::Ident(i)
    }
}
impl<'a> From<&'a str> for StringOrIdent {
    fn from(s: &'a str) -> Self {
        Self::String(s.to_owned())
    }
}

/// A struct or enum variant field.
pub struct Field {
    name: String,
    vis: Visibility,
    ty: String,
    attributes: Vec<(String, StreamBuilder)>,
}

impl Field {
    fn new(name: impl Into<String>, vis: Visibility, ty: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            vis,
            ty: ty.into(),
            attributes: Vec::new(),
        }
    }
}

/// A helper trait to share attribute code between struct and enum generators.
trait AttributeContainer {
    fn derives(&mut self) -> &mut Vec<StringOrIdent>;
    fn attributes(&mut self) -> &mut Vec<(String, StreamBuilder)>;

    fn with_derive(&mut self, derive: impl Into<StringOrIdent>) -> &mut Self {
        self.derives().push(derive.into());
        self
    }

    fn with_derives(&mut self, derives: impl IntoIterator<Item = StringOrIdent>) -> &mut Self {
        self.derives().extend(derives);
        self
    }

    fn with_attribute<T>(&mut self, name: impl Into<String>, value: T) -> crate::Result<&mut Self>
    where
        T: FnOnce(&mut StreamBuilder) -> crate::Result,
    {
        self.attributes().push((name.into(), {
            let mut b = StreamBuilder::new();
            value(&mut b)?;
            b
        }));
        Ok(self)
    }

    fn build_derives(&mut self, b: &mut StreamBuilder) -> &mut Self {
        let derives = std::mem::take(self.derives());
        if !derives.is_empty() {
            build_attribute(b, "derive", |b| {
                b.group(Delimiter::Parenthesis, |b| {
                    for (idx, derive) in derives.into_iter().enumerate() {
                        if idx > 0 {
                            b.punct(',');
                        }
                        match derive {
                            StringOrIdent::String(s) => b.ident_str(s),
                            StringOrIdent::Ident(i) => b.ident(i),
                        };
                    }
                    Ok(())
                })
            })
            .expect("could not build derives");
        }
        self
    }

    fn build_attributes(&mut self, b: &mut StreamBuilder) -> &mut Self {
        for (name, value) in std::mem::take(self.attributes()) {
            build_attribute(b, name, |b| Ok(b.extend(value.stream)))
                .expect("could not build attribute");
        }
        self
    }
}

impl AttributeContainer for Field {
    fn derives(&mut self) -> &mut Vec<StringOrIdent> {
        unreachable!("fields cannot have derives")
    }

    fn attributes(&mut self) -> &mut Vec<(String, StreamBuilder)> {
        &mut self.attributes
    }
}

/// A helper trait to share field attribute code between struct and enum generators.
trait FieldContainer {
    fn fields(&mut self) -> &mut Vec<Field>;

    fn add_field_with_attribute<T>(
        &mut self,
        name: impl Into<String>,
        ty: impl Into<String>,
        vis: Visibility,
        attribute_name: impl Into<String>,
        attribute_value: T,
    ) -> crate::Result<&mut Self>
    where
        T: FnOnce(&mut StreamBuilder) -> crate::Result,
    {
        let mut field = Field::new(name, vis, ty);
        field.with_attribute(attribute_name, attribute_value)?;
        self.fields().push(field);
        Ok(self)
    }
}

fn build_attribute<T>(b: &mut StreamBuilder, name: impl AsRef<str>, build: T) -> crate::Result
where
    T: FnOnce(&mut StreamBuilder) -> crate::Result<&mut StreamBuilder>,
{
    b.punct('#').group(Delimiter::Bracket, |b| {
        build(b.ident_str(name))?;
        Ok(())
    })?;

    Ok(())
}
