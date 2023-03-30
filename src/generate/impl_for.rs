use super::{generate_fn::FnParent, FnBuilder, Parent, StreamBuilder};
use crate::{
    parse::{GenericConstraints, Generics},
    prelude::{Delimiter, Result},
};

#[must_use]
/// A helper struct for implementing a trait for a given struct or enum.
pub struct ImplFor<'a, P: Parent> {
    generator: &'a mut P,
    trait_name: String,
    lifetimes: Option<Vec<String>>,
    consts: Vec<StreamBuilder>,
    custom_generic_constraints: Option<GenericConstraints>,
    impl_types: Vec<StreamBuilder>,
    fns: Vec<(StreamBuilder, StreamBuilder)>,
}

impl<'a, P: Parent> ImplFor<'a, P> {
    pub(super) fn new(generator: &'a mut P, trait_name: impl Into<String>) -> Self {
        Self {
            generator,
            trait_name: trait_name.into(),
            lifetimes: None,
            consts: Vec::new(),
            custom_generic_constraints: None,
            impl_types: Vec::new(),
            fns: Vec::new(),
        }
    }

    pub(super) fn new_with_lifetimes<ITER, T>(
        generator: &'a mut P,
        trait_name: T,
        lifetimes: ITER,
    ) -> Self
    where
        ITER: IntoIterator,
        ITER::Item: Into<String>,
        T: Into<String>,
    {
        Self {
            generator,
            trait_name: trait_name.into(),
            lifetimes: Some(lifetimes.into_iter().map(Into::into).collect()),
            consts: Vec::new(),
            custom_generic_constraints: None,
            impl_types: Vec::new(),
            fns: Vec::new(),
        }
    }

    /// Add a const to the trait implementation
    /// ```no_run
    /// # use virtue::prelude::Generator;
    /// # let mut generator: Generator = unsafe { std::mem::zeroed() };
    /// generator.impl_for("Foo")
    ///          .generate_const("BAR", "u8")
    ///          .with_value(|b| {
    ///             b.push_parsed("5")?;
    ///             Ok(())
    ///          })?;
    /// # Ok::<_, virtue::Error>(())
    /// ```
    ///
    /// Generates:
    /// ```ignore
    /// impl Foo for <struct or enum> {
    ///     const BAR: u8 = 5;
    /// }
    pub fn generate_const<'s>(
        &'s mut self,
        name: impl Into<String>,
        ty: impl Into<String>,
    ) -> GenConst<'s, 'a, P> {
        GenConst {
            parent: self,
            name: name.into(),
            ty: ty.into(),
        }
    }

    /// Add a function to the trait implementation.
    ///
    /// `generator.impl_for("Foo").generate_fn("bar")` results in code like:
    ///
    /// ```ignore
    /// impl Foo for <struct or enum> {
    ///     fn bar() {}
    /// }
    /// ```
    ///
    /// See [`FnBuilder`] for more options, as well as information on how to fill the function body.
    pub fn generate_fn(&mut self, name: impl Into<String>) -> FnBuilder<ImplFor<'a, P>> {
        FnBuilder::new(self, name)
    }

    /// Add a type to the impl
    ///
    /// `generator.impl_for("Foo").impl_type("Bar", "u8")` results in code like:
    ///
    /// ```ignore
    /// impl Foo for <struct or enum> {
    ///     type Bar = u8;
    /// }
    /// ```
    pub fn impl_type(&mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Result {
        let mut builder = StreamBuilder::new();
        builder
            .ident_str("type")
            .push_parsed(name)?
            .punct('=')
            .push_parsed(value)?
            .punct(';');
        self.impl_types.push(builder);
        Ok(())
    }

    ///
    /// Modify the generic constraints of a type.
    /// This can be used to add additional type constraints to your implementation.
    ///
    /// ```ignore
    /// // Your derive:
    /// #[derive(YourTrait)]
    /// pub struct Foo<B> {
    ///     ...
    /// }
    ///
    /// // With this code:
    /// generator
    ///     .impl_for("YourTrait")
    ///     .modify_generic_constraints(|generics, constraints| {
    ///         for g in generics.iter_generics() {
    ///             constraints.push_generic(g, "YourTrait");
    ///         }
    ///     })
    ///
    /// // will generate:
    /// impl<B> YourTrait for Foo<B>
    ///     where B: YourTrait // <-
    /// {
    /// }
    /// ```
    ///
    pub fn modify_generic_constraints<CB>(&mut self, cb: CB) -> Result<&mut Self>
    where
        CB: FnOnce(&Generics, &mut GenericConstraints) -> Result,
    {
        if let Some(generics) = self.generator.generics() {
            let mut constraints = self
                .generator
                .generic_constraints()
                .cloned()
                .unwrap_or_default();
            cb(generics, &mut constraints)?;
            self.custom_generic_constraints = Some(constraints)
        }
        Ok(self)
    }
}

impl<'a, P: Parent> FnParent for ImplFor<'a, P> {
    fn append(&mut self, fn_definition: StreamBuilder, fn_body: StreamBuilder) -> Result {
        self.fns.push((fn_definition, fn_body));
        Ok(())
    }
}

impl<P: Parent> Drop for ImplFor<'_, P> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            return;
        }
        let mut builder = StreamBuilder::new();
        self.generate_fn_definition(&mut builder);

        builder
            .group(Delimiter::Brace, |builder| {
                for ty in std::mem::take(&mut self.impl_types) {
                    builder.append(ty);
                }
                for r#const in std::mem::take(&mut self.consts) {
                    builder.append(r#const);
                }
                for (fn_def, fn_body) in std::mem::take(&mut self.fns) {
                    builder.append(fn_def);
                    builder
                        .group(Delimiter::Brace, |body| {
                            *body = fn_body;
                            Ok(())
                        })
                        .unwrap();
                }
                Ok(())
            })
            .unwrap();

        self.generator.append(builder);
    }
}

impl<P: Parent> ImplFor<'_, P> {
    fn generate_fn_definition(&mut self, builder: &mut StreamBuilder) {
        builder.ident_str("impl");
        if let Some(lifetimes) = &self.lifetimes {
            if let Some(generics) = self.generator.generics() {
                builder.append(generics.impl_generics_with_additional_lifetimes(lifetimes));
            } else {
                append_lifetimes(builder, lifetimes);
            }
        } else if let Some(generics) = self.generator.generics() {
            builder.append(generics.impl_generics());
        }
        builder.push_parsed(&self.trait_name).unwrap();
        if let Some(lifetimes) = &self.lifetimes {
            append_lifetimes(builder, lifetimes);
        }
        builder.ident_str("for");
        builder.ident(self.generator.name().clone());
        if let Some(generics) = &self.generator.generics() {
            builder.append(generics.type_generics());
        }
        if let Some(generic_constraints) = self.custom_generic_constraints.take() {
            builder.append(generic_constraints.where_clause());
        } else if let Some(generic_constraints) = &self.generator.generic_constraints() {
            builder.append(generic_constraints.where_clause());
        }
    }
}

fn append_lifetimes(builder: &mut StreamBuilder, lifetimes: &[String]) {
    for (idx, lt) in lifetimes.iter().enumerate() {
        builder.punct(if idx == 0 { '<' } else { ',' });
        builder.lifetime_str(lt);
    }
    builder.punct('>');
}

/// A builder for constants.
pub struct GenConst<'a, 'b, P: Parent> {
    parent: &'a mut ImplFor<'b, P>,
    name: String,
    ty: String,
}
impl<'a, 'b, P: Parent> GenConst<'a, 'b, P> {
    /// Complete the constant definition. This function takes a callback that will form the value of the constant.
    ///
    /// ```no_run
    /// # use virtue::prelude::Generator;
    /// # let mut generator: Generator = unsafe { std::mem::zeroed() };
    /// generator.impl_for("Foo")
    ///          .generate_const("BAR", "u8")
    ///          .with_value(|b| {
    ///             b.push_parsed("5")?;
    ///             Ok(())
    ///          })?;
    /// # Ok::<_, virtue::Error>(())
    /// ```
    ///
    /// Generates:
    /// ```ignore
    /// impl Foo for <struct or enum> {
    ///     const BAR: u8 = 5;
    /// }
    /// ```
    pub fn with_value<F>(self, f: F) -> Result
    where
        F: FnOnce(&mut StreamBuilder) -> Result,
    {
        let mut builder = StreamBuilder::new();
        builder
            .ident_str("const")
            .push_parsed(self.name)?
            .punct(':')
            .push_parsed(self.ty)?
            .punct('=');
        f(&mut builder)?;
        builder.punct(';');
        self.parent.consts.push(builder);
        Ok(())
    }
}
