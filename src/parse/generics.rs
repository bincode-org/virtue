use super::utils::*;
use crate::generate::StreamBuilder;
use crate::prelude::{Ident, TokenTree};
use crate::{Error, Result};
use std::iter::Peekable;
use std::ops::{Deref, DerefMut};

/// A generic parameter for a struct or enum.
///
/// ```
/// use std::marker::PhantomData;
/// use std::fmt::Display;
///
/// // Generic will be `Generic::Generic("F")`
/// struct Foo<F> {
///     f: PhantomData<F>
/// }
/// // Generics will be `Generic::Generic("F: Display")`
/// struct Bar<F: Display> {
///     f: PhantomData<F>
/// }
/// // Generics will be `[Generic::Lifetime("a"), Generic::Generic("F: Display")]`
/// struct Baz<'a, F> {
///     f: PhantomData<&'a F>
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Generics(pub Vec<Generic>);

impl Generics {
    pub(crate) fn try_take(
        input: &mut Peekable<impl Iterator<Item = TokenTree>>,
    ) -> Result<Option<Generics>> {
        let maybe_punct = input.peek();
        if let Some(TokenTree::Punct(punct)) = maybe_punct {
            if punct.as_char() == '<' {
                let punct = assume_punct(input.next(), '<');
                let mut result = Generics(Vec::new());
                loop {
                    match input.peek() {
                        Some(TokenTree::Punct(punct)) if punct.as_char() == '\'' => {
                            result.push(Lifetime::take(input)?.into());
                            consume_punct_if(input, ',');
                        }
                        Some(TokenTree::Punct(punct)) if punct.as_char() == '>' => {
                            assume_punct(input.next(), '>');
                            break;
                        }
                        Some(TokenTree::Ident(ident)) if ident_eq(ident, "const") => {
                            result.push(ConstGeneric::take(input)?.into());
                            consume_punct_if(input, ',');
                        }
                        Some(TokenTree::Ident(_)) => {
                            result.push(SimpleGeneric::take(input)?.into());
                            consume_punct_if(input, ',');
                        }
                        x => {
                            return Err(Error::InvalidRustSyntax {
                                span: x.map(|x| x.span()).unwrap_or_else(|| punct.span()),
                                expected: format!("', > or an ident, got {:?}", x),
                            });
                        }
                    }
                }
                return Ok(Some(result));
            }
        }
        Ok(None)
    }

    /// Returns `true` if any of the generics is a [`Generic::Lifetime`]
    pub fn has_lifetime(&self) -> bool {
        self.iter().any(|lt| lt.is_lifetime())
    }

    /// Returns an iterator which contains only the simple type generics
    pub fn iter_generics(&self) -> impl Iterator<Item = &SimpleGeneric> {
        self.iter().filter_map(|g| match g {
            Generic::Generic(s) => Some(s),
            _ => None,
        })
    }

    /// Returns an iterator which contains only the lifetimes
    pub fn iter_lifetimes(&self) -> impl Iterator<Item = &Lifetime> {
        self.iter().filter_map(|g| match g {
            Generic::Lifetime(s) => Some(s),
            _ => None,
        })
    }

    /// Returns an iterator which contains only the const generics
    pub fn iter_consts(&self) -> impl Iterator<Item = &ConstGeneric> {
        self.iter().filter_map(|g| match g {
            Generic::Const(s) => Some(s),
            _ => None,
        })
    }

    pub(crate) fn impl_generics(&self) -> StreamBuilder {
        let mut result = StreamBuilder::new();
        result.punct('<');

        for (idx, generic) in self.iter().enumerate() {
            if idx > 0 {
                result.punct(',');
            }

            generic.append_to_result_with_constraints(&mut result);
        }

        result.punct('>');

        result
    }

    pub(crate) fn impl_generics_with_additional_lifetimes(
        &self,
        lifetime: &[String],
    ) -> StreamBuilder {
        let mut result = StreamBuilder::new();
        for (idx, lt) in lifetime.iter().enumerate() {
            result.punct(if idx == 0 { '<' } else { ',' });
            result.lifetime_str(lt);

            if self.has_lifetime() {
                for (idx, lt) in self.iter().filter_map(|lt| lt.as_lifetime()).enumerate() {
                    result.punct(if idx == 0 { ':' } else { '+' });
                    result.lifetime(lt.ident.clone());
                }
            }

            for generic in self.iter() {
                result.punct(',');
                generic.append_to_result_with_constraints(&mut result);
            }
        }

        result.punct('>');

        result
    }

    pub(crate) fn type_generics(&self) -> StreamBuilder {
        let mut result = StreamBuilder::new();
        result.punct('<');

        for (idx, generic) in self.iter().enumerate() {
            if idx > 0 {
                result.punct(',');
            }
            if generic.is_lifetime() {
                result.lifetime(generic.ident());
            } else {
                result.ident(generic.ident());
            }
        }

        result.punct('>');
        result
    }
}

impl Deref for Generics {
    type Target = Vec<Generic>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Generics {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A single generic argument on a type
#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
#[non_exhaustive]
pub enum Generic {
    /// A lifetime generic
    ///
    /// ```
    /// # use std::marker::PhantomData;
    /// struct Foo<'a> { // will be Generic::Lifetime("a")
    /// #   a: PhantomData<&'a ()>,
    /// }
    /// ```
    Lifetime(Lifetime),
    /// A simple generic
    ///
    /// ```
    /// # use std::marker::PhantomData;
    /// struct Foo<F> { // will be Generic::Generic("F")
    /// #   a: PhantomData<F>,
    /// }
    /// ```
    Generic(SimpleGeneric),
    /// A const generic
    ///
    /// ```
    /// struct Foo<const N: usize> { // will be Generic::Const("N")
    /// #   a: [u8; N],
    /// }
    /// ```
    Const(ConstGeneric),
}

impl Generic {
    fn is_lifetime(&self) -> bool {
        matches!(self, Generic::Lifetime(_))
    }

    fn ident(&self) -> Ident {
        match self {
            Self::Lifetime(lt) => lt.ident.clone(),
            Self::Generic(gen) => gen.ident.clone(),
            Self::Const(gen) => gen.ident.clone(),
        }
    }

    fn as_lifetime(&self) -> Option<&Lifetime> {
        match self {
            Self::Lifetime(lt) => Some(lt),
            _ => None,
        }
    }

    fn has_constraints(&self) -> bool {
        match self {
            Self::Lifetime(lt) => !lt.constraint.is_empty(),
            Self::Generic(gen) => !gen.constraints.is_empty(),
            Self::Const(_) => true, // const generics always have a constraint
        }
    }

    fn constraints(&self) -> Vec<TokenTree> {
        match self {
            Self::Lifetime(lt) => lt.constraint.clone(),
            Self::Generic(gen) => gen.constraints.clone(),
            Self::Const(gen) => gen.constraints.clone(),
        }
    }

    fn append_to_result_with_constraints(&self, builder: &mut StreamBuilder) {
        match self {
            Self::Lifetime(lt) => builder.lifetime(lt.ident.clone()),
            Self::Generic(gen) => builder.ident(gen.ident.clone()),
            Self::Const(gen) => {
                builder.ident(gen.const_token.clone());
                builder.ident(gen.ident.clone())
            }
        };
        if self.has_constraints() {
            builder.punct(':');
            builder.extend(self.constraints());
        }
    }
}

impl From<Lifetime> for Generic {
    fn from(lt: Lifetime) -> Self {
        Self::Lifetime(lt)
    }
}

impl From<SimpleGeneric> for Generic {
    fn from(gen: SimpleGeneric) -> Self {
        Self::Generic(gen)
    }
}

impl From<ConstGeneric> for Generic {
    fn from(gen: ConstGeneric) -> Self {
        Self::Const(gen)
    }
}

#[test]
fn test_generics_try_take() {
    use crate::token_stream;

    assert!(Generics::try_take(&mut token_stream("")).unwrap().is_none());
    assert!(Generics::try_take(&mut token_stream("foo"))
        .unwrap()
        .is_none());
    assert!(Generics::try_take(&mut token_stream("()"))
        .unwrap()
        .is_none());

    let stream = &mut token_stream("struct Foo<'a, T>()");
    let (data_type, ident) = super::DataType::take(stream).unwrap();
    assert_eq!(data_type, super::DataType::Struct);
    assert_eq!(ident, "Foo");
    let generics = Generics::try_take(stream).unwrap().unwrap();
    assert_eq!(generics.len(), 2);
    assert_eq!(generics[0].ident(), "a");
    assert_eq!(generics[1].ident(), "T");

    let stream = &mut token_stream("struct Foo<A, B>()");
    let (data_type, ident) = super::DataType::take(stream).unwrap();
    assert_eq!(data_type, super::DataType::Struct);
    assert_eq!(ident, "Foo");
    let generics = Generics::try_take(stream).unwrap().unwrap();
    assert_eq!(generics.len(), 2);
    assert_eq!(generics[0].ident(), "A");
    assert_eq!(generics[1].ident(), "B");

    let stream = &mut token_stream("struct Foo<'a, T: Display>()");
    let (data_type, ident) = super::DataType::take(stream).unwrap();
    assert_eq!(data_type, super::DataType::Struct);
    assert_eq!(ident, "Foo");
    let generics = Generics::try_take(stream).unwrap().unwrap();
    dbg!(&generics);
    assert_eq!(generics.len(), 2);
    assert_eq!(generics[0].ident(), "a");
    assert_eq!(generics[1].ident(), "T");

    let stream = &mut token_stream("struct Foo<'a, T: for<'a> Bar<'a> + 'static>()");
    let (data_type, ident) = super::DataType::take(stream).unwrap();
    assert_eq!(data_type, super::DataType::Struct);
    assert_eq!(ident, "Foo");
    dbg!(&generics);
    assert_eq!(generics.len(), 2);
    assert_eq!(generics[0].ident(), "a");
    assert_eq!(generics[1].ident(), "T");

    let stream = &mut token_stream(
        "struct Baz<T: for<'a> Bar<'a, for<'b> Bar<'b, for<'c> Bar<'c, u32>>>> {}",
    );
    let (data_type, ident) = super::DataType::take(stream).unwrap();
    assert_eq!(data_type, super::DataType::Struct);
    assert_eq!(ident, "Baz");
    let generics = Generics::try_take(stream).unwrap().unwrap();
    dbg!(&generics);
    assert_eq!(generics.len(), 1);
    assert_eq!(generics[0].ident(), "T");

    let stream = &mut token_stream("struct Baz<()> {}");
    let (data_type, ident) = super::DataType::take(stream).unwrap();
    assert_eq!(data_type, super::DataType::Struct);
    assert_eq!(ident, "Baz");
    assert!(Generics::try_take(stream)
        .unwrap_err()
        .is_invalid_rust_syntax());

    let stream = &mut token_stream("struct Bar<A: FnOnce(&'static str) -> SomeStruct, B>");
    let (data_type, ident) = super::DataType::take(stream).unwrap();
    assert_eq!(data_type, super::DataType::Struct);
    assert_eq!(ident, "Bar");
    let generics = Generics::try_take(stream).unwrap().unwrap();
    dbg!(&generics);
    assert_eq!(generics.len(), 2);
    assert_eq!(generics[0].ident(), "A");
    assert_eq!(generics[1].ident(), "B");
}

/// a lifetime generic parameter, e.g. `struct Foo<'a> { ... }`
#[derive(Debug, Clone)]
pub struct Lifetime {
    ident: Ident,
    constraint: Vec<TokenTree>,
}

impl Lifetime {
    pub(crate) fn take(input: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Result<Self> {
        let start = assume_punct(input.next(), '\'');
        let ident = match input.peek() {
            Some(TokenTree::Ident(_)) => assume_ident(input.next()),
            Some(t) => return Err(Error::ExpectedIdent(t.span())),
            None => return Err(Error::ExpectedIdent(start.span())),
        };

        let mut constraint = Vec::new();
        if let Some(TokenTree::Punct(p)) = input.peek() {
            if p.as_char() == ':' {
                assume_punct(input.next(), ':');
                constraint = read_tokens_until_punct(input, &[',', '>'])?;
            }
        }

        Ok(Self { ident, constraint })
    }

    #[cfg(test)]
    fn is_ident(&self, s: &str) -> bool {
        self.ident.to_string() == s
    }
}

#[test]
fn test_lifetime_take() {
    use crate::token_stream;
    use std::panic::catch_unwind;
    assert!(Lifetime::take(&mut token_stream("'a"))
        .unwrap()
        .is_ident("a"));
    assert!(catch_unwind(|| Lifetime::take(&mut token_stream("'0"))).is_err());
    assert!(catch_unwind(|| Lifetime::take(&mut token_stream("'("))).is_err());
    assert!(catch_unwind(|| Lifetime::take(&mut token_stream("')"))).is_err());
    assert!(catch_unwind(|| Lifetime::take(&mut token_stream("'0'"))).is_err());

    let stream = &mut token_stream("'a: 'b>");
    let lifetime = Lifetime::take(stream).unwrap();
    assert_eq!(lifetime.ident, "a");
    assert_eq!(lifetime.constraint.len(), 2);
    assume_punct(stream.next(), '>');
    assert!(stream.next().is_none());
}

/// a simple generic parameter, e.g. `struct Foo<F> { .. }`
#[derive(Debug, Clone)]
pub struct SimpleGeneric {
    ident: Ident,
    constraints: Vec<TokenTree>,
}

impl SimpleGeneric {
    pub(crate) fn take(input: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Result<Self> {
        let ident = assume_ident(input.next());
        let mut constraints = Vec::new();
        if let Some(TokenTree::Punct(punct)) = input.peek() {
            if punct.as_char() == ':' {
                assume_punct(input.next(), ':');
                constraints = read_tokens_until_punct(input, &['>', ','])?;
            }
        }
        Ok(Self { ident, constraints })
    }

    /// The name of this generic, e.g. `T`
    pub fn name(&self) -> Ident {
        self.ident.clone()
    }
}

/// a const generic parameter, e.g. `struct Foo<const N: usize> { .. }`
#[derive(Debug, Clone)]
pub struct ConstGeneric {
    const_token: Ident,
    ident: Ident,
    constraints: Vec<TokenTree>,
}

impl ConstGeneric {
    pub fn take(input: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Result<Self> {
        let const_token = assume_ident(input.next());
        let ident = assume_ident(input.next());
        let mut constraints = Vec::new();
        if let Some(TokenTree::Punct(punct)) = input.peek() {
            if punct.as_char() == ':' {
                assume_punct(input.next(), ':');
                constraints = read_tokens_until_punct(input, &['>', ','])?;
            }
        }
        Ok(Self {
            const_token,
            ident,
            constraints,
        })
    }
}

/// Constraints on generic types.
///
/// ```
/// # use std::marker::PhantomData;
/// # use std::fmt::Display;
///
/// struct Foo<F>
///     where F: Display // These are `GenericConstraints`
/// {
///     f: PhantomData<F>
/// }
#[derive(Debug, Clone, Default)]
pub struct GenericConstraints {
    constraints: Vec<TokenTree>,
}

impl GenericConstraints {
    pub(crate) fn try_take(
        input: &mut Peekable<impl Iterator<Item = TokenTree>>,
    ) -> Result<Option<Self>> {
        match input.peek() {
            Some(TokenTree::Ident(ident)) => {
                if !ident_eq(ident, "where") {
                    return Ok(None);
                }
            }
            _ => {
                return Ok(None);
            }
        }
        input.next();
        let constraints = read_tokens_until_punct(input, &['{', '('])?;
        Ok(Some(Self { constraints }))
    }

    pub(crate) fn where_clause(&self) -> StreamBuilder {
        let mut result = StreamBuilder::new();
        result.ident_str("where");
        result.extend(self.constraints.clone());
        result
    }

    /// Push the given constraint onto this stream.
    ///
    /// ```ignore
    /// let mut generic_constraints = GenericConstraints::parse("T: Foo"); // imaginary function
    /// let mut generic = SimpleGeneric::new("U"); // imaginary function
    ///
    /// generic_constraints.push_constraint(&generic, "Bar");
    ///
    /// // generic_constraints is now:
    /// // `T: Foo, U: Bar`
    /// ```
    pub fn push_constraint(
        &mut self,
        generic: &SimpleGeneric,
        constraint: impl AsRef<str>,
    ) -> Result<()> {
        let mut builder = StreamBuilder::new();
        if !self.constraints.is_empty() {
            builder.punct(',');
        }
        builder.ident(generic.ident.clone());
        builder.punct(':');
        builder.push_parsed(constraint)?;
        self.constraints.extend(builder.stream.into_iter());

        Ok(())
    }

    /// Push the given constraint onto this stream.
    ///
    /// ```ignore
    /// let mut generic_constraints = GenericConstraints::parse("T: Foo"); // imaginary function
    ///
    /// generic_constraints.push_parsed_constraint("u32: SomeTrait");
    ///
    /// // generic_constraints is now:
    /// // `T: Foo, u32: SomeTrait`
    /// ```
    pub fn push_parsed_constraint(&mut self, constraint: impl AsRef<str>) -> Result<()> {
        let mut builder = StreamBuilder::new();
        if !self.constraints.is_empty() {
            builder.punct(',');
        }
        builder.push_parsed(constraint)?;
        self.constraints.extend(builder.stream.into_iter());

        Ok(())
    }

    /// Clear the constraints
    pub fn clear(&mut self) {
        self.constraints.clear();
    }
}

#[test]
fn test_generic_constraints_try_take() {
    use super::{DataType, StructBody, Visibility};
    use crate::token_stream;

    let stream = &mut token_stream("struct Foo where Foo: Bar { }");
    DataType::take(stream).unwrap();
    assert!(GenericConstraints::try_take(stream).unwrap().is_some());

    let stream = &mut token_stream("struct Foo { }");
    DataType::take(stream).unwrap();
    assert!(GenericConstraints::try_take(stream).unwrap().is_none());

    let stream = &mut token_stream("struct Foo where Foo: Bar(Foo)");
    DataType::take(stream).unwrap();
    assert!(GenericConstraints::try_take(stream).unwrap().is_some());

    let stream = &mut token_stream("struct Foo()");
    DataType::take(stream).unwrap();
    assert!(GenericConstraints::try_take(stream).unwrap().is_none());

    let stream = &mut token_stream("struct Foo()");
    assert!(GenericConstraints::try_take(stream).unwrap().is_none());

    let stream = &mut token_stream("{}");
    assert!(GenericConstraints::try_take(stream).unwrap().is_none());

    let stream = &mut token_stream("");
    assert!(GenericConstraints::try_take(stream).unwrap().is_none());

    let stream = &mut token_stream("pub(crate) struct Test<T: Encode> {}");
    assert_eq!(Visibility::Pub, Visibility::take(stream));
    let (data_type, ident) = DataType::take(stream).unwrap();
    assert_eq!(data_type, DataType::Struct);
    assert_eq!(ident, "Test");
    let constraints = Generics::try_take(stream).unwrap().unwrap();
    assert_eq!(constraints.len(), 1);
    assert_eq!(constraints[0].ident(), "T");
    let body = StructBody::take(stream).unwrap();
    assert_eq!(body.fields.len(), 0);
}
