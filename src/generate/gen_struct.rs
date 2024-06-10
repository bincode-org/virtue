use super::{
    AttributeContainer, Field, FieldContainer, Impl, ImplFor, Parent, StreamBuilder, StringOrIdent,
};
use crate::parse::Visibility;
use crate::prelude::{Delimiter, Ident, Span};

/// Builder to generate a struct.
/// Defaults to a struct with named fields `struct <Name> { <field>: <ty>, ... }`
pub struct GenStruct<'a, P: Parent> {
    parent: &'a mut P,
    name: Ident,
    visibility: Visibility,
    fields: Vec<Field>,
    derives: Vec<StringOrIdent>,
    attributes: Vec<(String, StreamBuilder)>,
    additional: Vec<StreamBuilder>,
    struct_type: StructType,
}

impl<'a, P: Parent> GenStruct<'a, P> {
    pub(crate) fn new(parent: &'a mut P, name: impl Into<String>) -> Self {
        Self {
            parent,
            name: Ident::new(name.into().as_str(), Span::call_site()),
            visibility: Visibility::Default,
            fields: Vec::new(),
            derives: Vec::new(),
            attributes: Vec::new(),
            additional: Vec::new(),
            struct_type: StructType::Named,
        }
    }

    /// Make the struct a zero-sized type (no fields)
    ///
    /// Any fields will be ignored
    ///
    /// ```
    /// # use virtue::prelude::Generator;
    /// # let mut generator = Generator::with_name("Fooz");
    /// generator
    ///     .generate_struct("Foo")
    ///     .make_zst()
    ///     .add_field("bar", "u16")
    ///     .add_field("baz", "String");
    /// # generator.assert_eq("struct Foo ;");
    /// # Ok::<_, virtue::Error>(())
    /// ```
    ///
    /// Generates:
    /// ```
    /// struct Foo;
    /// ```
    pub fn make_zst(&mut self) -> &mut Self {
        self.struct_type = StructType::Zst;
        self
    }

    /// Make the struct fields unnamed
    ///
    /// The names of any field will be ignored
    ///
    /// ```
    /// # use virtue::prelude::Generator;
    /// # let mut generator = Generator::with_name("Fooz");
    /// generator
    ///     .generate_struct("Foo")
    ///     .make_tuple()
    ///     .add_field("bar", "u16")
    ///     .add_field("baz", "String");
    /// # generator.assert_eq("struct Foo (u16 , String ,) ;");
    /// # Ok::<_, virtue::Error>(())
    /// ```
    ///
    /// Generates:
    /// ```
    /// struct Foo(u16, String);
    /// ```
    pub fn make_tuple(&mut self) -> &mut Self {
        self.struct_type = StructType::Unnamed;
        self
    }

    /// Make the struct `pub`. By default the struct will have no visibility modifier and will only be visible in the current scope.
    pub fn make_pub(&mut self) -> &mut Self {
        self.visibility = Visibility::Pub;
        self
    }

    /// Add a derive macro to the struct.
    ///
    /// ```
    /// # use virtue::prelude::Generator;
    /// # let mut generator = Generator::with_name("Bar");
    /// generator
    ///     .generate_struct("Foo")
    ///     .with_derive("Clone")
    ///     .with_derive("Default");
    /// # generator.assert_eq("# [derive (Clone , Default)] struct Foo { }");
    /// # Ok::<_, virtue::Error>(())
    /// ```
    ///
    /// Generates:
    /// ```ignore
    /// #[derive(Clone, Default)]
    /// struct Foo { }
    pub fn with_derive(&mut self, derive: impl Into<StringOrIdent>) -> &mut Self {
        AttributeContainer::with_derive(self, derive)
    }

    /// Add derive macros to the struct.
    ///
    /// ```
    /// # use virtue::prelude::Generator;
    /// # let mut generator = Generator::with_name("Bar");
    /// generator
    ///     .generate_struct("Foo")
    ///     .with_derives(["Clone".into(), "Default".into()]);
    /// # generator.assert_eq("# [derive (Clone , Default)] struct Foo { }");
    /// # Ok::<_, virtue::Error>(())
    /// ```
    ///
    /// Generates:
    /// ```ignore
    /// #[derive(Clone, Default)]
    /// struct Foo { }
    pub fn with_derives(&mut self, derives: impl IntoIterator<Item = StringOrIdent>) -> &mut Self {
        AttributeContainer::with_derives(self, derives)
    }

    /// Add an attribute to the struct. For `#[derive(...)]`, use [`with_derive`](Self::with_derive)
    /// instead.
    ///
    /// ```
    /// # use virtue::prelude::Generator;
    /// # let mut generator = Generator::with_name("Bar");
    /// generator
    ///     .generate_struct("Foo")
    ///     .with_attribute("serde", |b| {
    ///         b.push_parsed("(rename_all = \"camelCase\")")?;
    ///         Ok(())
    ///     })?;
    /// # generator.assert_eq("# [serde (rename_all = \"camelCase\")] struct Foo { }");
    /// # Ok::<_, virtue::Error>(())
    /// ```
    ///
    /// Generates:
    /// ```ignore
    /// #[serde(rename_all = "camelCase")]
    /// struct Foo { }
    /// ```
    pub fn with_attribute<T>(
        &mut self,
        name: impl Into<String>,
        value: T,
    ) -> crate::Result<&mut Self>
    where
        T: FnOnce(&mut StreamBuilder) -> crate::Result,
    {
        AttributeContainer::with_attribute(self, name, value)
    }

    /// Add a *private* field to the struct. For adding a public field, see `add_pub_field`
    ///
    /// Names are ignored when the Struct's fields are unnamed
    ///
    /// ```
    /// # use virtue::prelude::Generator;
    /// # let mut generator = Generator::with_name("Fooz");
    /// generator
    ///     .generate_struct("Foo")
    ///     .add_field("bar", "u16")
    ///     .add_field("baz", "String");
    /// # generator.assert_eq("struct Foo { bar : u16 , baz : String , }");
    /// # Ok::<_, virtue::Error>(())
    /// ```
    ///
    /// Generates:
    /// ```
    /// struct Foo {
    ///     bar: u16,
    ///     baz: String,
    /// };
    /// ```
    pub fn add_field(&mut self, name: impl Into<String>, ty: impl Into<String>) -> &mut Self {
        self.fields.push(Field::new(name, Visibility::Default, ty));
        self
    }

    /// Add a *private* field with an attribute to the struct. For adding a public field, see `add_pub_field_with_attribute`
    ///
    /// Names are ignored when the Struct's fields are unnamed
    ///
    /// ```
    /// # use virtue::prelude::Generator;
    /// # let mut generator = Generator::with_name("Bar");
    /// generator
    ///     .generate_struct("Foo")
    ///     .add_field_with_attribute("bar", "u16", "serde", |b| {
    ///         b.push_parsed("(default)")?;
    ///         Ok(())
    ///     })?;
    /// # generator.assert_eq("struct Foo { # [serde (default)] bar : u16 , }");
    /// # Ok::<_, virtue::Error>(())
    /// ```
    ///
    /// Generates:
    /// ```ignore
    /// struct Foo {
    ///     #[serde(default)]
    ///     bar: u16
    /// }
    /// ```
    pub fn add_field_with_attribute<T>(
        &mut self,
        name: impl Into<String>,
        ty: impl Into<String>,
        attribute_name: impl Into<String>,
        attribute_value: T,
    ) -> crate::Result<&mut Self>
    where
        T: FnOnce(&mut StreamBuilder) -> crate::Result,
    {
        FieldContainer::add_field_with_attribute(
            self,
            name,
            ty,
            Visibility::Default,
            attribute_name,
            attribute_value,
        )
    }

    /// Add a *public* field to the struct. For adding a private field, see `add_field`
    ///
    /// Names are ignored when the Struct's fields are unnamed
    pub fn add_pub_field(&mut self, name: impl Into<String>, ty: impl Into<String>) -> &mut Self {
        self.fields.push(Field::new(name, Visibility::Pub, ty));
        self
    }

    /// Add a *public* field with an attribute to the struct. For adding a private field, see `add_field_with_attribute`
    ///
    /// Names are ignored when the Struct's fields are unnamed
    ///
    /// ```
    /// # use virtue::prelude::Generator;
    /// # let mut generator = Generator::with_name("Bar");
    /// generator
    ///     .generate_struct("Foo")
    ///     .add_pub_field_with_attribute("bar", "u16", "serde", |b| {
    ///         b.push_parsed("(default)")?;
    ///         Ok(())
    ///     })?;
    /// # generator.assert_eq("struct Foo { # [serde (default)] pub bar : u16 , }");
    /// # Ok::<_, virtue::Error>(())
    /// ```
    ///
    /// Generates:
    /// ```ignore
    /// struct Foo {
    ///     #[serde(default)]
    ///     pub bar: u16
    /// }
    /// ```
    pub fn add_pub_field_with_attribute<T>(
        &mut self,
        name: impl Into<String>,
        ty: impl Into<String>,
        attribute_name: impl Into<String>,
        attribute_value: T,
    ) -> crate::Result<&mut Self>
    where
        T: FnOnce(&mut StreamBuilder) -> crate::Result,
    {
        FieldContainer::add_field_with_attribute(
            self,
            name,
            ty,
            Visibility::Pub,
            attribute_name,
            attribute_value,
        )
    }

    /// Add an `impl <name> for <struct>`
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

impl<P: Parent> AttributeContainer for GenStruct<'_, P> {
    fn derives(&mut self) -> &mut Vec<StringOrIdent> {
        &mut self.derives
    }

    fn attributes(&mut self) -> &mut Vec<(String, StreamBuilder)> {
        &mut self.attributes
    }
}

impl<P: Parent> FieldContainer for GenStruct<'_, P> {
    fn fields(&mut self) -> &mut Vec<Field> {
        &mut self.fields
    }
}

impl<'a, P: Parent> Parent for GenStruct<'a, P> {
    fn append(&mut self, builder: StreamBuilder) {
        self.additional.push(builder);
    }

    fn name(&self) -> &Ident {
        &self.name
    }

    fn generics(&self) -> Option<&crate::parse::Generics> {
        None
    }

    fn generic_constraints(&self) -> Option<&crate::parse::GenericConstraints> {
        None
    }
}

impl<'a, P: Parent> Drop for GenStruct<'a, P> {
    fn drop(&mut self) {
        use std::mem::take;
        let mut builder = StreamBuilder::new();

        self.build_derives(&mut builder)
            .build_attributes(&mut builder);

        if self.visibility == Visibility::Pub {
            builder.ident_str("pub");
        }
        builder.ident_str("struct").ident(self.name.clone());

        match self.struct_type {
            StructType::Named => builder
                .group(Delimiter::Brace, |b| {
                    for field in self.fields.iter_mut() {
                        field.build_attributes(b);
                        if field.vis == Visibility::Pub {
                            b.ident_str("pub");
                        }
                        b.ident_str(&field.name)
                            .punct(':')
                            .push_parsed(&field.ty)?
                            .punct(',');
                    }
                    Ok(())
                })
                .expect("Could not build struct"),
            StructType::Unnamed => builder
                .group(Delimiter::Parenthesis, |b| {
                    for field in self.fields.iter_mut() {
                        field.build_attributes(b);
                        if field.vis == Visibility::Pub {
                            b.ident_str("pub");
                        }
                        b.push_parsed(&field.ty)?.punct(',');
                    }
                    Ok(())
                })
                .expect("Could not build struct")
                .punct(';'),
            StructType::Zst => builder.punct(';'),
        };

        for additional in take(&mut self.additional) {
            builder.append(additional);
        }
        self.parent.append(builder);
    }
}

enum StructType {
    Named,
    Unnamed,
    Zst,
}
