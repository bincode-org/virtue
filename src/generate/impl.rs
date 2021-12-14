use super::{generate_fn::FnParent, FnBuilder, Generator, StreamBuilder};
use crate::{
    parse::{GenericConstraints, Generics},
    prelude::{Delimiter, Result},
};

#[must_use]
/// A helper struct for implementing functions for a given struct or enum.
pub struct Impl<'a> {
    generator: &'a mut Generator,
    // pub(super) group: StreamBuilder,
    custom_generic_constraints: Option<GenericConstraints>,
    fns: Vec<(StreamBuilder, StreamBuilder)>,
}

impl<'a> Impl<'a> {
    pub(super) fn new(generator: &'a mut Generator) -> Self {
        Self {
            generator,
            custom_generic_constraints: None,
            fns: Vec::new(),
        }
    }

    /// Add a function to the trait implementation.
    ///
    /// `generator.impl().generate_fn("bar")` results in code like:
    ///
    /// ```ignore
    /// impl <struct or enum> {
    ///     fn bar() {}
    /// }
    /// ```
    ///
    /// See [`FnBuilder`] for more options, as well as information on how to fill the function body.
    pub fn generate_fn<'b>(&'b mut self, name: &'a str) -> FnBuilder<'b, Impl<'a>> {
        FnBuilder::new(self, name)
    }

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
    ///     .r#impl()
    ///     .modify_generic_constraints(|generics, constraints| {
    ///         for g in generics.iter_generics() {
    ///             constraints.push_generic(g, "YourTrait");
    ///         }
    ///     })
    ///
    /// // will generate:
    /// impl<B> Foo<B>
    ///     where B: YourTrait // <-
    /// {
    /// }
    /// ```
    ///
    pub fn modify_generic_constraints<CB>(&mut self, cb: CB) -> &mut Self
    where
        CB: FnOnce(&Generics, &mut GenericConstraints),
    {
        if let Some(generics) = self.generator.generics.as_ref() {
            let mut constraints = self
                .generator
                .generic_constraints
                .clone()
                .unwrap_or_default();
            cb(generics, &mut constraints);
            self.custom_generic_constraints = Some(constraints)
        }
        self
    }
}

impl<'a> FnParent for Impl<'a> {
    fn append(&mut self, fn_definition: StreamBuilder, fn_body: StreamBuilder) -> Result {
        self.fns.push((fn_definition, fn_body));
        Ok(())
    }
}

impl Drop for Impl<'_> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            return;
        }
        let mut builder = StreamBuilder::new();
        builder.ident_str("impl");

        if let Some(generics) = &self.generator.generics {
            builder.append(generics.impl_generics());
        }
        builder.ident(self.generator.name.clone());

        if let Some(generics) = &self.generator.generics {
            builder.append(generics.type_generics());
        }
        if let Some(generic_constraints) = self.custom_generic_constraints.take() {
            builder.append(generic_constraints.where_clause());
        } else if let Some(generic_constraints) = &self.generator.generic_constraints {
            builder.append(generic_constraints.where_clause());
        }

        builder
            .group(Delimiter::Brace, |builder| {
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

        self.generator.stream.append(builder);
    }
}
