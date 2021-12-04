use crate::error::Error;
use crate::prelude::{Delimiter, Group, Ident, Punct, TokenTree};
use std::iter::Peekable;

mod attributes;
mod body;
mod data_type;
mod generics;
mod visibility;
mod utils;

pub use self::attributes::{Attribute, AttributeLocation, FieldAttribute};
pub use self::body::{EnumBody, EnumVariant, Fields, StructBody, UnnamedField};
pub use self::data_type::DataType;
pub use self::generics::{GenericConstraints, Generics, Lifetime, SimpleGeneric};
pub use self::visibility::Visibility;

use crate::generate::Generator;

#[non_exhaustive]
pub enum Parse {
    Struct {
        attributes: Vec<Attribute>,
        visibility: Visibility,
        name: Ident,
        generics: Option<Generics>,
        generic_constraints: Option<GenericConstraints>,
        body: StructBody,
    },
    Enum {
        attributes: Vec<Attribute>,
        visibility: Visibility,
        name: Ident,
        generics: Option<Generics>,
        generic_constraints: Option<GenericConstraints>,
        body: EnumBody,
    }
}

impl Parse {
    pub fn into_generator(self) -> (Generator, Body) {
        match self {
            Parse::Struct {
                name,
                generics,
                generic_constraints,
                body,
                ..
            } => {
                (Generator::new(name, generics, generic_constraints), Body::Struct(body))
            },
            Parse::Enum {
                name,
                generics,
                generic_constraints,
                body,
                ..
            } => {
                (Generator::new(name, generics, generic_constraints), Body::Enum(body))
            }
        }
    }
}

pub enum Body {
    Struct(StructBody),
    Enum(EnumBody),
}
