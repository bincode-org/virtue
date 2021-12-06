use super::StreamBuilder;
use crate::prelude::{Delimiter, Result};

/// A builder for functions.
pub struct FnBuilder<'a, P> {
    parent: &'a mut P,
    name: &'a str,

    lifetimes: Vec<(&'a str, Vec<&'a str>)>,
    generics: Vec<(&'a str, Vec<&'a str>)>,
    self_arg: FnSelfArg,
    args: Vec<(&'a str, &'a str)>,
    return_type: Option<&'a str>,
}

impl<'a, P: FnParent> FnBuilder<'a, P> {
    pub(super) fn new(parent: &'a mut P, name: &'a str) -> Self {
        Self {
            parent,
            name,
            lifetimes: Vec::new(),
            generics: Vec::new(),
            self_arg: FnSelfArg::None,
            args: Vec::new(),
            return_type: None,
        }
    }

    /// Add a lifetime parameter.
    ///
    /// `dependencies` are the optional lifetime dependencies of the given lifetime.
    ///
    /// ```no_run
    /// # use virtue::prelude::Generator;
    /// # let mut generator: Generator = unsafe { std::mem::zeroed() };
    /// generator
    ///     .r#impl()
    ///     .generate_fn("foo") // fn foo()
    ///     .with_lifetime("a", None) // fn foo<'a>()
    ///     .with_lifetime("b", ["a"]); // fn foo<'a, 'b: 'a>();
    /// ```
    pub fn with_lifetime<DEP>(mut self, name: &'a str, dependencies: DEP) -> Self
    where
        DEP: IntoIterator<Item = &'a str>,
    {
        self.lifetimes
            .push((name, dependencies.into_iter().collect()));
        self
    }
    /// Add a generic parameter. Keep in mind that will *not* work for lifetimes.
    ///
    /// `dependencies` are the optional dependencies of the parameter.
    ///
    /// ```no_run
    /// # use virtue::prelude::Generator;
    /// # let mut generator: Generator = unsafe { std::mem::zeroed() };
    /// generator
    ///     .r#impl()
    ///     .generate_fn("foo") // fn foo()
    ///     .with_generic("D", None) // fn foo<D>()
    ///     .with_generic("E", ["Encodable"]); // fn foo<D, E: Encodable>();
    /// ```
    pub fn with_generic<DEP>(mut self, name: &'a str, dependencies: DEP) -> Self
    where
        DEP: IntoIterator<Item = &'a str>,
    {
        self.generics
            .push((name, dependencies.into_iter().collect()));
        self
    }

    /// Set the value for `self`. See [FnSelfArg] for more information.
    ///
    /// ```no_run
    /// # use virtue::prelude::{Generator, FnSelfArg};
    /// # let mut generator: Generator = unsafe { std::mem::zeroed() };
    /// generator
    ///     .r#impl()
    ///     .generate_fn("foo") // fn foo()
    ///     .with_self_arg(FnSelfArg::RefSelf); // fn foo(&self)
    /// ```
    pub fn with_self_arg(mut self, self_arg: FnSelfArg) -> Self {
        self.self_arg = self_arg;
        self
    }

    /// Add an argument with a `name` and a `ty`.
    ///
    /// ```no_run
    /// # use virtue::prelude::Generator;
    /// # let mut generator: Generator = unsafe { std::mem::zeroed() };
    /// generator
    ///     .r#impl()
    ///     .generate_fn("foo") // fn foo()
    ///     .with_arg("a", "u32") // fn foo(a: u32)
    ///     .with_arg("b", "u32"); // fn foo(a: u32, b: u32)
    /// ```
    pub fn with_arg(mut self, name: &'a str, ty: &'a str) -> Self {
        self.args.push((name, ty));
        self
    }

    /// Set the return type for the function. By default the function will have no return type.
    ///
    /// ```no_run
    /// # use virtue::prelude::Generator;
    /// # let mut generator: Generator = unsafe { std::mem::zeroed() };
    /// generator
    ///     .r#impl()
    ///     .generate_fn("foo") // fn foo()
    ///     .with_return_type("u32"); // fn foo() -> u32
    /// ```
    pub fn with_return_type(mut self, ret_type: &'a str) -> Self {
        self.return_type = Some(ret_type);
        self
    }

    /// Complete the function definition. This function takes a callback that will form the body of the function.
    ///
    /// ```no_run
    /// # use virtue::prelude::Generator;
    /// # let mut generator: Generator = unsafe { std::mem::zeroed() };
    /// generator
    ///     .r#impl()
    ///     .generate_fn("foo") // fn foo()
    ///     .body(|b| {
    ///         b.push_parsed("println!(\"hello world\");")
    ///     })
    ///     .unwrap();
    /// // fn foo() {
    /// //     println!("Hello world");
    /// // }
    /// ```
    pub fn body(
        self,
        body_builder: impl FnOnce(&mut StreamBuilder) -> crate::Result,
    ) -> crate::Result {
        let FnBuilder {
            parent,
            name,
            lifetimes,
            generics,
            self_arg,
            args,
            return_type,
        } = self;

        let mut builder = StreamBuilder::new();

        // function name; `fn name`
        builder.ident_str("fn");
        builder.ident_str(name);

        // lifetimes; `<'a: 'b, D: Display>`
        if !lifetimes.is_empty() || !generics.is_empty() {
            builder.punct('<');
            let mut is_first = true;
            for (lifetime, dependencies) in lifetimes {
                if is_first {
                    is_first = false;
                } else {
                    builder.punct(',');
                }
                builder.lifetime_str(lifetime);
                if !dependencies.is_empty() {
                    for (idx, dependency) in dependencies.into_iter().enumerate() {
                        builder.punct(if idx == 0 { ':' } else { '+' });
                        builder.lifetime_str(dependency);
                    }
                }
            }
            for (generic, dependencies) in generics {
                if is_first {
                    is_first = false;
                } else {
                    builder.punct(',');
                }
                builder.ident_str(&generic);
                if !dependencies.is_empty() {
                    for (idx, dependency) in dependencies.into_iter().enumerate() {
                        builder.punct(if idx == 0 { ':' } else { '+' });
                        builder.push_parsed(&dependency)?;
                    }
                }
            }
            builder.punct('>');
        }

        // Arguments; `(&self, foo: &Bar)`
        builder.group(Delimiter::Parenthesis, |arg_stream| {
            if let Some(self_arg) = self_arg.into_token_tree() {
                arg_stream.append(self_arg);
                arg_stream.punct(',');
            }
            for (idx, (arg_name, arg_ty)) in args.into_iter().enumerate() {
                if idx != 0 {
                    arg_stream.punct(',');
                }
                arg_stream.push_parsed(&arg_name)?;
                arg_stream.punct(':');
                arg_stream.push_parsed(&arg_ty)?;
            }
            Ok(())
        })?;

        // Return type: `-> ResultType`
        if let Some(return_type) = return_type {
            builder.puncts("->");
            builder.push_parsed(&return_type)?;
        }

        let mut body_stream = StreamBuilder::new();
        body_builder(&mut body_stream)?;

        parent.append(builder, body_stream)
    }
}

pub trait FnParent {
    fn append(&mut self, fn_definition: StreamBuilder, fn_body: StreamBuilder) -> Result;
}

/// The `self` argument of a function
#[allow(dead_code)]
#[non_exhaustive]
pub enum FnSelfArg {
    /// No `self` argument. The function will be a static function.
    None,

    /// `self`. The function will consume self.
    TakeSelf,

    /// `&self`. The function will take self by reference.
    RefSelf,

    /// `&mut self`. The function will take self by mutable reference.
    MutSelf,
}

impl FnSelfArg {
    fn into_token_tree(self) -> Option<StreamBuilder> {
        let mut builder = StreamBuilder::new();
        match self {
            Self::None => return None,
            Self::TakeSelf => {
                builder.ident_str("self");
            }
            Self::RefSelf => {
                builder.punct('&');
                builder.ident_str("self");
            }
            Self::MutSelf => {
                builder.punct('&');
                builder.ident_str("mut");
                builder.ident_str("self");
            }
        }
        Some(builder)
    }
}
