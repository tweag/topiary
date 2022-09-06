// Comments
// at the beginning.

// More comments.

enum OneLine { Leaf { content: String, id: usize, size: usize, }, Hardline { content: String, id: usize }, Space, } // End of line comment

enum Foo { Bar } // Comment
enum Next { Bar }
enum Third { Bar }

enum ExpandEnum {
  Leaf { content: String, /* Comment between fields. */ id: usize, size: usize, },
  Hardline { content: String, id: usize },
  Space,
}
enum NoFinalComma {
  Space
}

enum ExpandTwoLevels {

  Leaf {
    content: String, //   Comment after field declaration in enum variant.
    id: usize,

    size: usize,
  },
  Hardline { content: String, id: usize },

  // comment between enum items

  Space,
}
