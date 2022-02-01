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
    custom_generic_constraints: Option<GenericConstraints>,
    fns: Vec<(StreamBuilder, StreamBuilder)>,
}

impl<'a, P: Parent> ImplFor<'a, P> {
    pub(super) fn new(generator: &'a mut P, trait_name: impl Into<String>) -> Self {
        Self {
            generator,
            trait_name: trait_name.into(),
            lifetimes: None,
            custom_generic_constraints: None,
            fns: Vec::new(),
        }
    }

    pub(super) fn new_with_lifetimes<ITER, I, T>(
        generator: &'a mut P,
        trait_name: T,
        lifetimes: ITER,
    ) -> Self
    where
        ITER: IntoIterator<Item = I>,
        I: Into<String>,
        T: Into<String>,
    {
        Self {
            generator,
            trait_name: trait_name.into(),
            lifetimes: Some(lifetimes.into_iter().map(Into::into).collect()),
            custom_generic_constraints: None,
            fns: Vec::new(),
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
    pub fn modify_generic_constraints<CB>(&mut self, cb: CB) -> &mut Self
    where
        CB: FnOnce(&Generics, &mut GenericConstraints),
    {
        if let Some(generics) = self.generator.generics() {
            let mut constraints = self
                .generator
                .generic_constraints()
                .cloned()
                .unwrap_or_default();
            cb(generics, &mut constraints);
            self.custom_generic_constraints = Some(constraints)
        }
        self
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
