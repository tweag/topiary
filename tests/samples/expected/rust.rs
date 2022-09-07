unsafe impl Send for Language {}
unsafe impl Sync for Language {}

pub fn node_kind_for_id(&self, id: u16) -> &'static str {
    unsafe { CStr::from_ptr(ffi::ts_language_symbol_name(self.0, id)) }
    .to_str()
    .unwrap()
}

// Comments
// at the beginning.

// More comments.

enum OneLine {
    Leaf {
        content: String,
        id: usize,
        size: usize,
    },
    Hardline {
        content: String,
        id: usize,
    },
    Space,
} // End of line comment

enum Foo {
    Bar,
} // Comment
enum Next {
    Bar,
}
enum Third {
    Bar,
}

enum ExpandEnum {
    Leaf {
        content: String,
        /* Comment between fields. */ id: usize,
        size: usize,
    },
    Hardline {
        content: String,
        id: usize,
    },
    Space,
}
enum NoFinalComma {
    Space,
}

enum ExpandTwoLevels {
    Leaf {
        content: String,
         //   Comment after field declaration in enum variant.
        id: usize,

        size: usize,
    },
    Hardline {
        content: String,
        id: usize,
    },

    // comment between enum items
    Space,
}
