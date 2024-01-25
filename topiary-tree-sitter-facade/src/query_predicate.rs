#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::borrow::Cow;

    pub struct QueryPredicate<'query> {
        pub(crate) inner: &'query tree_sitter::QueryPredicate,
    }

    impl<'query> QueryPredicate<'query> {
        #[inline]
        pub fn operator(&self) -> Cow<str> {
            Cow::Borrowed(&self.inner.operator)
        }

        #[inline]
        pub fn args(&self) -> Vec<String> {
            let args: Vec<_> = self
                .inner
                .args
                .iter()
                .map(|s| match s {
                    tree_sitter::QueryPredicateArg::String(s) => s.to_string(),
                    _ => {
                        unimplemented!("Only string predicate arguments are currently implemented.")
                    }
                })
                .collect();

            args
        }
    }

    impl<'query> std::fmt::Debug for QueryPredicate<'query> {
        fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            std::fmt::Debug::fmt(&self.inner, fmt)
        }
    }

    impl<'query> From<&'query tree_sitter::QueryPredicate> for QueryPredicate<'query> {
        #[inline]
        fn from(inner: &'query tree_sitter::QueryPredicate) -> Self {
            Self { inner }
        }
    }

    impl<'query> std::panic::RefUnwindSafe for QueryPredicate<'query> {}

    impl<'query> Unpin for QueryPredicate<'query> {}

    impl<'query> std::panic::UnwindSafe for QueryPredicate<'query> {}
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use std::borrow::Cow;
    use wasm_bindgen::JsCast;

    pub struct QueryPredicate {
        pub(crate) inner: topiary_web_tree_sitter_sys::QueryPredicate,
    }

    impl QueryPredicate {
        #[inline]
        pub fn operator(&self) -> Cow<str> {
            Cow::Owned(self.inner.operator().as_string().unwrap())
        }

        #[inline]
        pub fn args(&self) -> Vec<String> {
            let args: Vec<_> = self
                .inner
                .operands()
                .iter()
                .cloned()
                .map(|value| {
                    let arg =
                        value.unchecked_into::<topiary_web_tree_sitter_sys::QueryPredicateArg>();
                    arg.value().as_string().unwrap()
                })
                .collect();

            args
        }
    }

    impl std::fmt::Debug for QueryPredicate {
        fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            std::fmt::Debug::fmt(&self.inner, fmt)
        }
    }

    impl From<topiary_web_tree_sitter_sys::QueryPredicate> for QueryPredicate {
        #[inline]
        fn from(inner: topiary_web_tree_sitter_sys::QueryPredicate) -> Self {
            Self { inner }
        }
    }

    impl std::panic::RefUnwindSafe for QueryPredicate {}

    impl Unpin for QueryPredicate {}

    impl std::panic::UnwindSafe for QueryPredicate {}
}

#[cfg(target_arch = "wasm32")]
pub use wasm::*;
