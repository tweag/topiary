use std::path::PathBuf;

fn main() {
    let src_dir = PathBuf::from("src");

    cc::Build::new()
        .include(&src_dir)
        .file(src_dir.join("parser.c"))
        .compile("tree-sitter-shebang");
}
