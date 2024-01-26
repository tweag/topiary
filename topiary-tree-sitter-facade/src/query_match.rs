#[cfg(not(target_arch = "wasm32"))]
mod native {
    use crate::query_capture::QueryCapture;
    use std::convert::TryFrom;

    pub struct QueryMatch<'tree> {
        pub(crate) inner: tree_sitter::QueryMatch<'tree, 'tree>,
    }

    impl<'tree> QueryMatch<'tree> {
        #[inline]
        pub fn pattern_index(&self) -> u32 {
            u32::try_from(self.inner.pattern_index).unwrap()
        }

        #[inline]
        pub fn captures(&self) -> impl ExactSizeIterator<Item = QueryCapture<'tree>> {
            self.inner.captures.iter().map(Into::into)
        }
    }

    impl<'tree> From<tree_sitter::QueryMatch<'tree, 'tree>> for QueryMatch<'tree> {
        #[inline]
        fn from(inner: tree_sitter::QueryMatch<'tree, 'tree>) -> Self {
            Self { inner }
        }
    }

    impl<'tree> std::panic::RefUnwindSafe for QueryMatch<'tree> {}

    impl<'tree> Unpin for QueryMatch<'tree> {}

    impl<'tree> std::panic::UnwindSafe for QueryMatch<'tree> {}
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use crate::query_capture::QueryCapture;
    use wasm_bindgen::JsCast;

    #[derive(Clone)]
    pub struct QueryMatch<'tree> {
        pub(crate) inner: topiary_web_tree_sitter_sys::QueryMatch,
        pub(crate) phantom: std::marker::PhantomData<&'tree ()>,
    }

    impl<'tree> QueryMatch<'tree> {
        #[inline]
        pub fn pattern_index(&self) -> u32 {
            self.inner.pattern()
        }

        #[inline]
        pub fn captures(&self) -> impl ExactSizeIterator<Item = QueryCapture<'tree>> + 'tree {
            self.inner.captures().into_vec().into_iter().map(|value| {
                value
                    .unchecked_into::<topiary_web_tree_sitter_sys::QueryCapture>()
                    .into()
            })
        }
    }

    impl<'tree> From<topiary_web_tree_sitter_sys::QueryMatch> for QueryMatch<'tree> {
        #[inline]
        fn from(inner: topiary_web_tree_sitter_sys::QueryMatch) -> Self {
            let phantom = std::marker::PhantomData;
            Self { inner, phantom }
        }
    }

    impl<'tree> std::panic::RefUnwindSafe for QueryMatch<'tree> {}

    impl<'tree> Unpin for QueryMatch<'tree> {}

    impl<'tree> std::panic::UnwindSafe for QueryMatch<'tree> {}
}

#[cfg(target_arch = "wasm32")]
pub use wasm::*;
