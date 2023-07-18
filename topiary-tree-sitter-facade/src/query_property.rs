#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::borrow::Cow;

    pub struct QueryProperty<'query> {
        pub(crate) inner: &'query tree_sitter::QueryProperty,
    }

    impl<'query> QueryProperty<'query> {
        #[inline]
        pub fn key(&self) -> Cow<str> {
            Cow::Borrowed(&self.inner.key)
        }

        #[inline]
        pub fn value(&self) -> Option<Cow<str>> {
            match &self.inner.value {
                Some(v) => Some(Cow::Borrowed(v)),
                None => None,
            }
        }

        #[inline]
        pub fn capture_id(&self) -> Option<Cow<usize>> {
            self.inner.capture_id.as_ref().map(Cow::Borrowed)
        }
    }

    impl<'query> std::fmt::Debug for QueryProperty<'query> {
        fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            std::fmt::Debug::fmt(&self.inner, fmt)
        }
    }

    impl<'query> From<&'query tree_sitter::QueryProperty> for QueryProperty<'query> {
        #[inline]
        fn from(inner: &'query tree_sitter::QueryProperty) -> Self {
            Self { inner }
        }
    }

    impl<'query> std::panic::RefUnwindSafe for QueryProperty<'query> {}

    impl<'query> Unpin for QueryProperty<'query> {}

    impl<'query> std::panic::UnwindSafe for QueryProperty<'query> {}
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

// ERIN: TODO
#[cfg(target_arch = "wasm32")]
mod wasm {
    use std::borrow::Cow;
    use wasm_bindgen::JsCast;

    pub struct QueryPredicate {
        pub(crate) inner: web_tree_sitter::QueryPredicate,
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
                    let arg = value.unchecked_into::<web_tree_sitter::QueryPredicateArg>();
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

    impl From<web_tree_sitter::QueryPredicate> for QueryPredicate {
        #[inline]
        fn from(inner: web_tree_sitter::QueryPredicate) -> Self {
            Self { inner }
        }
    }

    impl std::panic::RefUnwindSafe for QueryPredicate {}

    impl Unpin for QueryPredicate {}

    impl std::panic::UnwindSafe for QueryPredicate {}
}

#[cfg(target_arch = "wasm32")]
pub use wasm::*;
