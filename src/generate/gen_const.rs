use super::StreamBuilder;
use crate::{parse::Visibility, prelude::Result};

/// Builder for constants.
pub struct GenConst<'a, P> {
    parent: &'a mut P,
    name: String,
    ty: String,
    vis: Visibility,
}

impl<'a, P: ConstParent> GenConst<'a, P> {
    pub(super) fn new(parent: &'a mut P, name: impl Into<String>, ty: impl Into<String>) -> Self {
        Self {
            parent,
            name: name.into(),
            vis: Visibility::Default,
            ty: ty.into(),
        }
    }

    /// Make the const `pub`. By default the const will have no visibility modifier and will only be visible in the current scope.
    #[must_use]
    pub fn make_pub(mut self) -> Self {
        self.vis = Visibility::Pub;
        self
    }

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
        if self.vis == Visibility::Pub {
            builder.ident_str("pub");
        }
        builder
            .ident_str("const")
            .push_parsed(self.name)?
            .punct(':')
            .push_parsed(self.ty)?
            .punct('=');
        f(&mut builder)?;
        builder.punct(';');
        self.parent.append(builder)
    }
}

pub trait ConstParent {
    fn append(&mut self, builder: StreamBuilder) -> Result;
}
