use topiary_tree_sitter_facade::{InputEdit, Language, Node, Parser, Tree};

use crate::{
    error::FormatterError,
    types::{Diff, InputSection, Position},
    FormatterResult,
};

/// When you remove a block of text from the input, it changes the positions of every subsequent character.
/// This is what this Diff instance does.
impl Diff<InputSection> for Position {
    type ErrorType = FormatterError;

    fn subtract(self: &mut Self, other: InputSection) -> FormatterResult<()> {
        if *self <= other.start {
            // The point is before the removed block: nothing happens.
            Ok(())
        } else if other.end <= *self {
            // The point is after the removed block: its new coordinates depend on whether it was
            // on the same row as the last point of the removed block.
            //
            // See in the following example how the positions of characters `a` and `b`
            // change when the bracketed block is removed:
            //
            // Before:
            // ..........
            // ...[---------
            // ---------
            // -------]...a..
            // ...b......
            // .............
            //
            // After:
            // ..........
            // ......a..
            // ...b......
            // .............
            let mut row = self.row;
            let mut column = self.column;
            if row == other.end.row {
                column = column + other.start.column - other.end.column
            }
            row = row + other.start.row - other.end.row;
            *self = Position { row, column };
            Ok(())
        } else {
            // The point is within the removed block:
            // fail, because the point can't be accessed after the subtraction
            Err(FormatterError::Internal(
                "Tried to remove a section from a point it contains".into(),
                None,
            ))
        }
    }
}

impl Diff<InputSection> for InputSection {
    type ErrorType = FormatterError;

    fn subtract(self: &mut Self, other: Self) -> FormatterResult<()> {
        self.start.subtract(other)?;
        self.end.subtract(other)
    }
}

// TODO: allow the users to manually identify comments. Maybe with a separate query file?
fn is_comment(node: &Node) -> bool {
    node.is_extra() && node.kind().to_string().contains("comment")
}

fn find_comments<'a>(
    node: Node<'a>,
    input: &str,
    comments: &mut Vec<(Node<'a>, AnchoredComment)>,
) -> FormatterResult<()> {
    if is_comment(&node) {
        let commented = find_anchor(&node, input)?;
        comments.push((
            node.clone(),
            AnchoredComment {
                comment_text: node.utf8_text(input.as_bytes())?.to_string(),
                commented,
            },
        ));
        Ok(())
    } else {
        let mut walker = node.walk();
        for child in node.children(&mut walker) {
            find_comments(child, input, comments)?;
        }
        Ok(())
    }
}

/// The section of code to which a comment refers. We also remember whether the comment
/// is positioned before or after the section.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Commented {
    /// The code section is before the comment, as in:
    /// ```
    /// struct Foo {
    ///     baz: usize, // this is baz
    ///     quz: usize, // this is qux
    /// }
    /// ```
    CommentedBefore(InputSection),
    /// The code section is after the comment, as in:
    /// ```
    /// struct Foo {
    ///     // let's have a baz
    ///     baz: usize,
    ///     // and a qux
    ///     qux: usize,
    /// }
    /// ```
    CommentedAfter(InputSection),
}

impl Diff<InputSection> for Commented {
    type ErrorType = FormatterError;

    fn subtract(self: &mut Self, other: InputSection) -> FormatterResult<()> {
        match self {
            Commented::CommentedBefore(section) => section.subtract(other),
            Commented::CommentedAfter(section) => section.subtract(other),
        }
    }
}

// TODO: if performance is an issue, use TreeCursor to navigate the tree
fn next_non_comment_leaf<'tree>(starting_node: Node<'tree>) -> Option<Node<'tree>> {
    let mut node: Node<'tree> = starting_node;
    loop {
        // get the next leaf:
        // 1) move up until we find a next sibling
        loop {
            match node.next_sibling() {
                None => {
                    if let Some(parent) = node.parent() {
                        node = parent
                    } else {
                        return None;
                    } // we've reached the root and found nothing
                }
                Some(sibling) => {
                    node = sibling;
                    if is_comment(&node) {
                        // get the following sibling
                        continue;
                    } else {
                        break;
                    }
                }
            }
        }
        // 2) get the leftmost leaf of the sibling.
        // If we encounter a comment, we stop. We'll get back to 1) after the loop
        while let Some(child) = node.child(0) {
            if is_comment(&child) {
                break;
            } else {
                node = child
            }
        }
        // check if the leaf is a comment. If it is not, start over again.
        if is_comment(&node) {
            continue;
        } else {
            return Some(node);
        }
    }
}

// TODO: if performance is an issue, use TreeCursor to navigate the tree
fn previous_non_comment_leaf<'tree>(starting_node: Node<'tree>) -> Option<Node<'tree>> {
    let mut node: Node<'tree> = starting_node;
    loop {
        // get the previous leaf:
        // 1) move up until we find a previous sibling
        loop {
            match node.prev_sibling() {
                None => {
                    if let Some(parent) = node.parent() {
                        node = parent
                    } else {
                        // we've reached the root and found nothing
                        return None;
                    }
                }
                Some(sibling) => {
                    node = sibling;
                    if is_comment(&node) {
                        // get the previous sibling
                        continue;
                    } else {
                        break;
                    }
                }
            }
        }
        // 2) get the rightmost leaf of the sibling.
        // If we encounter a comment, we stop. We'll get back to 1) after the loop
        while let Some(child) = {
            if node.child_count() == 0 {
                None
            } else {
                node.child(node.child_count() - 1)
            }
        } {
            if is_comment(&child) {
                break;
            } else {
                node = child
            }
        }
        // check if the leaf is a comment. If it is not, start over again.
        if is_comment(&node) {
            continue;
        } else {
            return Some(node);
        }
    }
}

fn reparse(
    old_tree: Tree,
    content: &str,
    grammar: &topiary_tree_sitter_facade::Language,
) -> FormatterResult<Tree> {
    let mut parser = Parser::new()?;
    parser.set_language(grammar)?;
    let tree = parser
        .parse(content, Some(&old_tree))?
        .ok_or_else(|| FormatterError::Internal("Could not parse input".into(), None))?;
    Ok(tree)
}

// Use the following heuristics to find a comment's anchor:
// If the comment is only prefixed by blank symbols on its line, then the anchor is the
// next non-comment sibling node.
// Otherwise, the anchor is the previous non-comment sibling node.
// If there is no such node, we anchor to the first non-comment sibling node
// in the other direction.
fn find_anchor<'tree>(node: &'tree Node<'tree>, input: &str) -> FormatterResult<Commented> {
    let point = node.start_position();
    let mut lines = input.lines();
    let prefix = lines
        .nth(point.row() as usize)
        .map(|line| &line[..point.column() as usize])
        .ok_or_else(|| {
            FormatterError::Internal(
                format!(
                    "Trying to access nonexistent line {} in text:\n{}",
                    point.row(),
                    input,
                ),
                None,
            )
        })?;
    if prefix.trim_start() == "" {
        if let Some(anchor) = next_non_comment_leaf(node.clone()) {
            return Ok(Commented::CommentedAfter(anchor.into()));
        } else if let Some(anchor) = previous_non_comment_leaf(node.clone()) {
            return Ok(Commented::CommentedBefore(anchor.into()));
        } else {
            return Err(FormatterError::Internal(
                format!("Could find no anchor for comment {node:?}",),
                None,
            ));
        }
    } else {
        if let Some(anchor) = previous_non_comment_leaf(node.clone()) {
            return Ok(Commented::CommentedBefore(anchor.into()));
        } else if let Some(anchor) = next_non_comment_leaf(node.clone()) {
            return Ok(Commented::CommentedAfter(anchor.into()));
        } else {
            return Err(FormatterError::Internal(
                format!("Could find no anchor for comment {node:?}",),
                None,
            ));
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnchoredComment {
    pub comment_text: String,
    pub commented: Commented,
}

pub struct SeparatedInput {
    pub input_tree: Tree,
    pub input_string: String,
    pub comments: Vec<AnchoredComment>,
}

pub fn extract_comments<'a>(
    tree: &'a Tree,
    input: &'a str,
    grammar: &Language,
) -> FormatterResult<SeparatedInput> {
    let mut anchors: Vec<(Node, AnchoredComment)> = Vec::new();
    let mut anchored_comments: Vec<AnchoredComment> = Vec::new();
    let mut new_input: String = input.to_string();
    let mut new_tree: Tree = tree.clone();
    find_comments(tree.root_node(), input, &mut anchors)?;
    anchors.sort_by_key(|(node, _)| node.start_byte());
    let mut edits: Vec<InputEdit> = Vec::new();
    // for each (comment, anchor) pair in reverse order, we:
    // 1) remove the comment from the input,
    // 2) register an InputEdit to modify the tree,
    // 3) edit the following anchors to account for the removed comment.
    //
    // The order is reversed so that all InputEdits can be applied in succession:
    // one will not affect the others.
    while let Some((comment, anchored_comment)) = anchors.pop() {
        // 1)
        new_input.replace_range(
            (comment.start_byte() as usize)..(comment.end_byte() as usize),
            "",
        );
        // 2)
        let edit = InputEdit::new(
            comment.start_byte(),
            comment.end_byte(),
            comment.start_byte(),
            &comment.start_position(),
            &comment.end_position(),
            &comment.start_position(),
        );
        edits.push(edit);
        // 3)
        anchored_comments.push(anchored_comment);
        anchored_comments = anchored_comments
            .iter()
            .map(
                |AnchoredComment {
                     mut commented,
                     comment_text,
                 }|
                 -> FormatterResult<AnchoredComment> {
                    commented.subtract(comment.clone().into())?;
                    Ok(AnchoredComment {
                        commented,
                        comment_text: comment_text.to_string(),
                    })
                },
            )
            .collect::<FormatterResult<Vec<_>>>()?;
    }
    for edit in edits {
        new_tree.edit(&edit);
    }
    new_tree = reparse(new_tree, new_input.as_str(), grammar)?;
    Ok(SeparatedInput {
        input_tree: new_tree,
        input_string: new_input,
        comments: anchored_comments,
    })
}
