use super::{Impl, ImplFor, Parent, StreamBuilder, StringOrIdent};
use crate::parse::{Generic, Generics, Visibility};
use crate::prelude::{Delimiter, Ident, Span};
use crate::Result;

/// Builder to generate an `enum <Name> { <value> { ... }, ... }`
///
/// ```
/// # use virtue::prelude::Generator;
/// # let mut generator = Generator::with_name("Fooz");
/// {
///     let mut enumgen = generator.generate_enum("Foo");
///     enumgen
///         .add_value("ZST")
///         .make_zst();
///     enumgen
///         .add_value("Named")
///         .add_field("bar", "u16")
///         .add_field("baz", "String");
///     enumgen
///         .add_value("Unnamed")
///         .add_field("", "u16")
///         .add_field("baz", "String")
///         .make_tuple();
/// }
/// # generator.assert_eq("enum Foo { ZST , Named { bar : u16 , baz : String , } , Unnamed (u16 , String ,) , }");
/// # Ok::<_, virtue::Error>(())
/// ```
///
/// Generates:
/// ```
/// enum Foo {
///     ZST,
///     Named {
///         bar: u16,
///         baz: String,
///     },
///     Unnamed(u16, String),
/// };
/// ```
pub struct GenEnum<'a, P: Parent> {
    parent: &'a mut P,
    name: Ident,
    visibility: Visibility,
    generics: Option<Generics>,
    values: Vec<EnumValue>,
    additional: Vec<StreamBuilder>,
}

impl<'a, P: Parent> GenEnum<'a, P> {
    pub(crate) fn new(parent: &'a mut P, name: impl Into<String>) -> Self {
        Self {
            parent,
            name: Ident::new(name.into().as_str(), Span::call_site()),
            visibility: Visibility::Default,
            generics: None,
            values: Vec::new(),
            additional: Vec::new(),
        }
    }

    /// Make the enum `pub`. By default the struct will have no visibility modifier and will only be visible in the current scope.
    pub fn make_pub(&mut self) -> &mut Self {
        self.visibility = Visibility::Pub;
        self
    }

    /// Inherit the generic parameters of the parent type.
    ///
    /// ```
    /// # use virtue::prelude::Generator;
    /// # use virtue::parse::{Generic, Lifetime};
    /// # use proc_macro2::{Ident, Span};
    /// # let mut generator = Generator::with_name("Bar").with_lifetime("a");
    /// // given a derive on enum Bar<'a>
    /// generator
    ///     .generate_enum("Foo")
    ///     .inherit_generics()
    ///     .add_value("Bar")
    ///     .make_tuple()
    ///     .add_field("bar", "&'a str");
    /// # generator.assert_eq("enum Foo < 'a > { Bar (&'a str ,) , }");
    /// # Ok::<_, virtue::Error>(())
    /// ```
    ///
    /// Generates:
    /// ```ignore
    /// // given a derive on enum Bar<'a>
    /// enum Foo<'a> {
    ///     Bar(&'a str)
    /// }
    /// ```
    pub fn inherit_generics(&mut self) -> &mut Self {
        self.generics = self.parent.generics().cloned();
        self
    }

    /// Append generic parameters to the type.
    ///
    /// ```
    /// # use virtue::prelude::Generator;
    /// # use virtue::parse::{Generic, Lifetime};
    /// # use proc_macro2::{Ident, Span};
    /// # let mut generator = Generator::with_name("Bar").with_lifetime("a");
    /// generator
    ///     .generate_enum("Foo")
    ///     .with_generics([Lifetime { ident: Ident::new("a", Span::call_site()), constraint: vec![] }.into()])
    ///     .add_value("Bar")
    ///     .make_tuple()
    ///     .add_field("bar", "&'a str");
    /// # generator.assert_eq("enum Foo < 'a > { Bar (&'a str ,) , }");
    /// # Ok::<_, virtue::Error>(())
    /// ```
    ///
    /// Generates:
    /// ```ignore
    /// enum Foo<'a> {
    ///     Bar(&'a str)
    /// }
    /// ```
    pub fn with_generics(&mut self, generics: impl IntoIterator<Item = Generic>) -> &mut Self {
        self.generics
            .get_or_insert_with(|| Generics(Vec::new()))
            .extend(generics);
        self
    }

    /// Add a generic parameter to the type.
    ///
    /// ```
    /// # use virtue::prelude::Generator;
    /// # use virtue::parse::{Generic, Lifetime};
    /// # use proc_macro2::{Ident, Span};
    /// # let mut generator = Generator::with_name("Bar").with_lifetime("a");
    /// generator
    ///     .generate_enum("Foo")
    ///     .with_generic(Lifetime { ident: Ident::new("a", Span::call_site()), constraint: vec![] }.into())
    ///     .add_value("Bar")
    ///     .make_tuple()
    ///     .add_field("bar", "&'a str");
    /// # generator.assert_eq("enum Foo < 'a > { Bar (&'a str ,) , }");
    /// # Ok::<_, virtue::Error>(())
    /// ```
    ///
    /// Generates:
    /// ```ignore
    /// enum Foo<'a> {
    ///     Bar(&'a str)
    /// }
    /// ```
    pub fn with_generic(&mut self, generic: Generic) -> &mut Self {
        self.generics
            .get_or_insert_with(|| Generics(Vec::new()))
            .push(generic);
        self
    }

    /// Add an enum value
    ///
    /// Returns a builder for the value that's similar to GenStruct
    pub fn add_value(&mut self, name: impl Into<String>) -> &mut EnumValue {
        self.values.push(EnumValue::new(name));
        self.values.last_mut().unwrap()
    }

    /// Add an `impl <name> for <enum>`
    pub fn impl_for(&mut self, name: impl Into<StringOrIdent>) -> ImplFor<Self> {
        ImplFor::new(self, name.into(), None)
    }

    /// Generate an `impl <name>` implementation. See [`Impl`] for more information.
    pub fn r#impl(&mut self) -> Impl<Self> {
        Impl::with_parent_name(self)
    }

    /// Generate an `impl <name>` implementation. See [`Impl`] for more information.
    ///
    /// Alias for [`impl`] which doesn't need a `r#` prefix.
    ///
    /// [`impl`]: #method.impl
    pub fn generate_impl(&mut self) -> Impl<Self> {
        Impl::with_parent_name(self)
    }
}

impl<'a, P: Parent> Parent for GenEnum<'a, P> {
    fn append(&mut self, builder: StreamBuilder) {
        self.additional.push(builder);
    }

    fn name(&self) -> &Ident {
        &self.name
    }

    fn generics(&self) -> Option<&Generics> {
        self.generics.as_ref()
    }

    fn generic_constraints(&self) -> Option<&crate::parse::GenericConstraints> {
        None
    }
}

impl<'a, P: Parent> Drop for GenEnum<'a, P> {
    fn drop(&mut self) {
        let mut builder = StreamBuilder::new();
        if self.visibility == Visibility::Pub {
            builder.ident_str("pub");
        }
        builder
            .ident_str("enum")
            .ident(self.name.clone())
            .append(
                self.generics()
                    .map(Generics::impl_generics)
                    .unwrap_or_default(),
            )
            .group(Delimiter::Brace, |b| {
                for value in &self.values {
                    build_value(b, value)?;
                }

                Ok(())
            })
            .expect("Could not build enum");

        for additional in std::mem::take(&mut self.additional) {
            builder.append(additional);
        }
        self.parent.append(builder);
    }
}

fn build_value(builder: &mut StreamBuilder, value: &EnumValue) -> Result {
    builder.ident(value.name.clone());

    match value.value_type {
        ValueType::Named => builder.group(Delimiter::Brace, |b| {
            for field in &value.fields {
                if field.vis == Visibility::Pub {
                    b.ident_str("pub");
                }
                b.ident_str(&field.name)
                    .punct(':')
                    .push_parsed(&field.ty)?
                    .punct(',');
            }
            Ok(())
        })?,
        ValueType::Unnamed => builder.group(Delimiter::Parenthesis, |b| {
            for field in &value.fields {
                if field.vis == Visibility::Pub {
                    b.ident_str("pub");
                }
                b.push_parsed(&field.ty)?.punct(',');
            }
            Ok(())
        })?,
        ValueType::Zst => builder,
    };

    builder.punct(',');

    Ok(())
}

pub struct EnumValue {
    name: Ident,
    fields: Vec<EnumField>,
    value_type: ValueType,
}

impl EnumValue {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: Ident::new(name.into().as_str(), Span::call_site()),
            fields: Vec::new(),
            value_type: ValueType::Named,
        }
    }

    /// Make the struct a zero-sized type (no fields)
    ///
    /// Any fields will be ignored
    pub fn make_zst(&mut self) -> &mut Self {
        self.value_type = ValueType::Zst;
        self
    }

    /// Make the struct fields unnamed
    ///
    /// The names of any field will be ignored
    pub fn make_tuple(&mut self) -> &mut Self {
        self.value_type = ValueType::Unnamed;
        self
    }

    /// Add a *private* field to the struct. For adding a public field, see `add_pub_field`
    ///
    /// Names are ignored when the Struct's fields are unnamed
    pub fn add_field(&mut self, name: impl Into<String>, ty: impl Into<String>) -> &mut Self {
        self.fields.push(EnumField {
            name: name.into(),
            vis: Visibility::Default,
            ty: ty.into(),
        });
        self
    }

    /// Add a *public* field to the struct. For adding a public field, see `add_field`
    ///
    /// Names are ignored when the Struct's fields are unnamed
    pub fn add_pub_field(&mut self, name: impl Into<String>, ty: impl Into<String>) -> &mut Self {
        self.fields.push(EnumField {
            name: name.into(),
            vis: Visibility::Pub,
            ty: ty.into(),
        });
        self
    }
}

struct EnumField {
    name: String,
    vis: Visibility,
    ty: String,
}

enum ValueType {
    Named,
    Unnamed,
    Zst,
}
