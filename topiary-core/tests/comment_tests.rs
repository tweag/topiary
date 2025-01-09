use topiary_core::{
    comments::{
        extract_comments, AnchoredComment,
        Commented::{CommentedAfter, CommentedBefore},
        SeparatedInput,
    },
    common::{parse, InputSection},
    Position,
};

const OCAML_WITH_COMMENTS: &str = r#"(* starting comment *)
fun (* fun comment *) x (* var comment *) ->
  (** multi-lined
    * body comment
    *)
  body
(* final comment *)
"#;

const OCAML_WITHOUT_COMMENTS: &str = r#"fun  x  ->
  body
"#;

// The section corresponding to `fun` in the curated code
const FUN_SECTION: InputSection = InputSection {
    start: Position { row: 1, column: 1 },
    end: Position { row: 1, column: 4 },
};

// The section corresponding to `x` in the curated code
const VAR_SECTION: InputSection = InputSection {
    start: Position { row: 1, column: 6 },
    end: Position { row: 1, column: 7 },
};

// The section corresponding to `body` in the curated code
const BODY_SECTION: InputSection = InputSection {
    start: Position { row: 2, column: 3 },
    end: Position { row: 2, column: 7 },
};

#[test]
fn test_extract_comments() {
    let input = OCAML_WITH_COMMENTS;
    let ocaml = tree_sitter_ocaml::LANGUAGE_OCAML;

    let tree = parse(input, &ocaml.into(), false, None).unwrap();

    let SeparatedInput {
        input_tree: _,
        input_string: new_input_string,
        mut comments,
    } = extract_comments(&tree, input, &ocaml.into(), false).unwrap();

    let mut expected_comments: Vec<AnchoredComment> = vec![
        AnchoredComment {
            comment_text: "(* starting comment *)".into(),
            commented: CommentedAfter {
                section: FUN_SECTION,
                blank_line_after: false,
                blank_line_before: false,
            },
            original_column: 0,
        },
        AnchoredComment {
            comment_text: "(* fun comment *)".into(),
            commented: CommentedBefore(FUN_SECTION),
            original_column: 4,
        },
        AnchoredComment {
            comment_text: "(* var comment *)".into(),
            commented: CommentedBefore(VAR_SECTION),
            original_column: 24,
        },
        AnchoredComment {
            comment_text: "(** multi-lined\n    * body comment\n    *)".into(),
            commented: CommentedAfter {
                section: BODY_SECTION,
                blank_line_after: false,
                blank_line_before: false,
            },
            original_column: 2,
        },
        AnchoredComment {
            comment_text: "(* final comment *)".into(),
            commented: CommentedBefore(BODY_SECTION),
            original_column: 0,
        },
    ];

    // sort the comments so that we're order-independent
    comments.sort_by_key(|comment| comment.comment_text.clone());
    expected_comments.sort_by_key(|comment| comment.comment_text.clone());

    assert_eq!(new_input_string, OCAML_WITHOUT_COMMENTS);

    assert_eq!(comments.len(), 5);
    for (comment, expected_comment) in comments.iter().zip(expected_comments.iter()) {
        assert_eq!(comment, expected_comment)
    }
}
