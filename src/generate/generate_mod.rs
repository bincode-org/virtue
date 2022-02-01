use super::{GenStruct, Impl, Parent, StreamBuilder};
use crate::{
    parse::Visibility,
    prelude::{Delimiter, Ident, Span},
};

/// Builder for generating a module with its contents.
pub struct GenerateMod<'a, P: Parent> {
    parent: &'a mut P,
    name: Ident,
    vis: Visibility,
    content: StreamBuilder,
}

impl<'a, P: Parent> GenerateMod<'a, P> {
    pub(crate) fn new(parent: &'a mut P, name: impl Into<String>) -> Self {
        Self {
            parent,
            name: Ident::new(name.into().as_str(), Span::call_site()),
            vis: Visibility::Default,
            content: StreamBuilder::new(),
        }
    }

    /// Generate a struct with the given name.
    pub fn generate_struct(&mut self, name: impl Into<String>) -> GenStruct<Self> {
        GenStruct::new(self, name)
    }

    /// Generate an `impl <name>` implementation. See [`Impl`] for more information.
    pub fn r#impl(&mut self, name: impl Into<String>) -> Impl<Self> {
        Impl::new(self, name)
    }

    /// Generate an `impl <name>` implementation. See [`Impl`] for more information.
    ///
    /// Alias for [`impl`] which doesn't need a `r#` prefix.
    ///
    /// [`impl`]: #method.impl
    pub fn generate_impl(&mut self, name: impl Into<String>) -> Impl<Self> {
        Impl::new(self, name)
    }
}

impl<'a, P: Parent> Drop for GenerateMod<'a, P> {
    fn drop(&mut self) {
        let mut builder = StreamBuilder::new();
        if self.vis == Visibility::Pub {
            builder.ident_str("pub");
        }
        builder
            .ident_str("mod")
            .ident(self.name.clone())
            .group(Delimiter::Brace, |group| {
                *group = std::mem::take(&mut self.content);
                Ok(())
            })
            .unwrap();

        self.parent.append(builder);
    }
}

impl<P: Parent> Parent for GenerateMod<'_, P> {
    fn append(&mut self, builder: StreamBuilder) {
        self.content.append(builder);
    }

    fn name(&self) -> &crate::prelude::Ident {
        &self.name
    }

    fn generics(&self) -> Option<&crate::parse::Generics> {
        None
    }

    fn generic_constraints(&self) -> Option<&crate::parse::GenericConstraints> {
        None
    }
}
