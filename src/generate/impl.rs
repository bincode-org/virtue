use super::{generate_fn::FnParent, FnBuilder, Generator, StreamBuilder};
use crate::prelude::{Delimiter, Result};

#[must_use]
/// A helper struct for implementing functions for a given struct or enum.
pub struct Impl<'a> {
    pub(super) generator: &'a mut Generator,
    pub(super) group: StreamBuilder,
}

impl<'a> Impl<'a> {
    pub(super) fn new(generator: &'a mut Generator) -> Self {
        let mut builder = StreamBuilder::new();
        builder.ident_str("impl");

        if let Some(generics) = &generator.generics {
            builder.append(generics.impl_generics());
        }
        builder.ident(generator.name.clone());

        if let Some(generics) = &generator.generics {
            builder.append(generics.type_generics());
        }
        if let Some(generic_constraints) = &generator.generic_constraints {
            builder.append(generic_constraints.where_clause());
        }
        generator.stream.append(builder);

        let group = StreamBuilder::new();
        Self { generator, group }
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
}

impl<'a> FnParent for Impl<'a> {
    fn append(&mut self, fn_definition: StreamBuilder, fn_body: StreamBuilder) -> Result {
        self.group.append(fn_definition);
        self.group.group(Delimiter::Brace, |body| {
            *body = fn_body;
            Ok(())
        })
    }
}

impl Drop for Impl<'_> {
    fn drop(&mut self) {
        let stream = std::mem::take(&mut self.group);
        self.generator
            .stream
            .group(Delimiter::Brace, |builder| {
                builder.append(stream);
                Ok(())
            })
            .unwrap();
    }
}
