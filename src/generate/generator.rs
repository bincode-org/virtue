use super::{Impl, ImplFor, StreamBuilder};
use crate::parse::{GenericConstraints, Generics};
use crate::prelude::{Ident, Result, TokenStream};

#[must_use]
/// The generator is used to generate code.
///
/// Often you will want to use [`impl_for`] to generate an `impl <trait_name> for <target_name()>`.
///
/// [`impl_for`]: #method.impl_for
pub struct Generator {
    pub(crate) name: Ident,
    pub(crate) generics: Option<Generics>,
    pub(crate) generic_constraints: Option<GenericConstraints>,
    pub(crate) stream: StreamBuilder,
}

impl Generator {
    pub(crate) fn new(
        name: Ident,
        generics: Option<Generics>,
        generic_constraints: Option<GenericConstraints>,
    ) -> Self {
        Self {
            name,
            generics,
            generic_constraints,
            stream: StreamBuilder::new(),
        }
    }

    /// Return the name for the struct or enum that this is going to be implemented on.
    pub fn target_name(&self) -> &Ident {
        &self.name
    }

    /// Generate an `impl <target_name>` implementation. See [Impl] for more information.
    pub fn r#impl(&mut self) -> Impl {
        Impl::new(self)
    }

    /// Generate an `impl <target_name>` implementation. See [Impl] for more information.
    ///
    /// Alias for [`impl`] which doesn't need a `r#` prefix.
    pub fn generate_impl(&mut self) -> Impl {
        Impl::new(self)
    }

    /// Generate an `for <trait_name> for <target_name>` implementation. See [ImplFor] for more information.
    pub fn impl_for<'a>(&'a mut self, trait_name: &str) -> Result<ImplFor<'a>> {
        ImplFor::new(self, trait_name)
    }

    /// Generate an `for <..lifetimes> <trait_name> for <target_name>` implementation. See [ImplFor] for more information.
    ///
    /// Note:
    /// - Lifetimes should _not_ have the leading apostrophe.
    /// - The lifetimes passed to this function will automatically depend on any other lifetime this struct or enum may have. Example:
    ///   - The struct is `struct Foo<'a> {}`
    ///   - You call `generator.impl_for_with_lifetime("Bar", &["b"])
    ///   - The code will be `impl<'a, 'b: 'a> Bar<'b> for Foo<'a> {}`
    /// - `trait_name` should _not_ have custom lifetimes. These will be added automatically.
    ///
    /// ```no_run
    /// # use virtue::prelude::*;
    /// # let mut generator: Generator = unsafe { std::mem::zeroed() };
    /// generator.impl_for_with_lifetimes("Foo", &["a", "b"]);
    ///
    /// // will output:
    /// // impl<'a, 'b> Foo<'a, 'b> for StructOrEnum { }
    /// ```
    pub fn impl_for_with_lifetimes<'a>(
        &'a mut self,
        trait_name: &str,
        lifetimes: &[&str],
    ) -> Result<ImplFor<'a>> {
        ImplFor::new_with_lifetimes(self, trait_name, lifetimes)
    }

    /// Consume the contents of this generator. This *must* be called, or else the generator will panic on drop.
    pub fn finish(mut self) -> crate::prelude::Result<TokenStream> {
        Ok(std::mem::take(&mut self.stream).stream)
    }
}

impl Drop for Generator {
    fn drop(&mut self) {
        if !self.stream.stream.is_empty() && !std::thread::panicking() {
            panic!("Generator dropped but the stream is not empty. Please call `.take_stream()` on the generator");
        }
    }
}
