enum OneLine { Leaf { content: String, id: usize, size: usize, }, Hardline { content: String, id: usize }, Space, }

enum ExpandEnum { 
    Leaf { content: String, id: usize, size: usize, }, Hardline { content: String, id: usize },
    Space, 
}

enum NoFinalComma { 
    Leaf { content: String, id: usize, size: usize, }, Hardline { content: String, id: usize },
    Space
}

enum WithTrailingComma {
    One,
    Two,
} 

enum WithoutTrailingComma {
    One,
    Two
} 

enum ExpandTwoLevels { 
    Leaf { 
        content: String, 
        id: usize,
        size: usize,
    },
    Hardline { content: String, id: usize },
    Space, 
}

