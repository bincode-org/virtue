use super::{generate_fn::FnParent, FnBuilder, Generator, StreamBuilder};
use crate::prelude::{Delimiter, Result};

#[must_use]
/// A helper struct for implementing a trait for a given struct or enum.
pub struct ImplFor<'a> {
    pub(super) generator: &'a mut Generator,
    pub(super) group: StreamBuilder,
}

impl<'a> ImplFor<'a> {
    pub(super) fn new(generator: &'a mut Generator, trait_name: &str) -> Result<Self> {
        let mut builder = StreamBuilder::new();
        builder.ident_str("impl");

        if let Some(generics) = &generator.generics {
            builder.append(generics.impl_generics());
        }
        builder.push_parsed(trait_name)?;
        builder.ident_str("for");
        builder.ident(generator.name.clone());

        if let Some(generics) = &generator.generics {
            builder.append(generics.type_generics());
        }
        if let Some(generic_constraints) = &generator.generic_constraints {
            builder.append(generic_constraints.where_clause());
        }
        generator.stream.append(builder);

        let group = StreamBuilder::new();
        Ok(Self { generator, group })
    }

    pub(super) fn new_with_lifetimes(
        generator: &'a mut Generator,
        trait_name: &str,
        lifetimes: &[&str],
    ) -> Result<Self> {
        let mut builder = StreamBuilder::new();
        builder.ident_str("impl");

        if let Some(generics) = &generator.generics {
            builder.append(generics.impl_generics_with_additional_lifetimes(lifetimes));
        } else {
            append_lifetimes(&mut builder, lifetimes);
        }

        builder.push_parsed(trait_name)?;
        append_lifetimes(&mut builder, lifetimes);
        builder.ident_str("for");
        builder.ident(generator.name.clone());

        if let Some(generics) = &generator.generics {
            builder.append(generics.type_generics());
        }

        if let Some(generic_constraints) = &generator.generic_constraints {
            builder.append(generic_constraints.where_clause());
        }
        generator.stream.append(builder);

        let group = StreamBuilder::new();
        Ok(Self { generator, group })
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
    pub fn generate_fn<'b>(&'b mut self, name: &'a str) -> FnBuilder<'b, ImplFor<'a>> {
        FnBuilder::new(self, name)
    }
}

impl<'a> FnParent for ImplFor<'a> {
    fn append(&mut self, fn_definition: StreamBuilder, fn_body: StreamBuilder) -> Result {
        self.group.append(fn_definition);
        self.group.group(Delimiter::Brace, |body| {
            *body = fn_body;
            Ok(())
        })
    }
}

impl Drop for ImplFor<'_> {
    fn drop(&mut self) {
        let stream = std::mem::take(&mut self.group);
        self.generator
            .stream
            .group(Delimiter::Brace, |builder| {
                builder.append(stream);
                Ok(())
            })
            .unwrap()
    }
}

fn append_lifetimes(builder: &mut StreamBuilder, lifetimes: &[&str]) {
    for (idx, lt) in lifetimes.iter().enumerate() {
        builder.punct(if idx == 0 { '<' } else { ',' });
        builder.lifetime_str(lt);
    }
    builder.punct('>');
}
