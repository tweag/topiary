use topiary_tree_sitter_facade::{InputEdit, Language, Node, Tree};

use crate::{
    common::{parse, Diff, InputSection, Position},
    error::FormatterError,
    FormatterResult,
};

/// When you remove a block of text from the input, it changes the positions of every subsequent character.
/// This is what this Diff instance does.
impl Diff<InputSection> for Position {
    type ErrorType = FormatterError;

    fn subtract(&mut self, other: InputSection) -> FormatterResult<()> {
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

    fn subtract(&mut self, other: Self) -> FormatterResult<()> {
        self.start.subtract(other)?;
        self.end.subtract(other)
    }
}

// TODO: allow the users to manually identify comments. Maybe with a separate query file?
fn is_comment(node: &Node) -> bool {
    node.is_extra() && node.kind().to_string().contains("comment")
}

fn into_edit(node: &Node<'_>) -> InputEdit {
    InputEdit::new(
        node.start_byte(),
        node.end_byte(),
        node.start_byte(),
        &node.start_position(),
        &node.end_position(),
        &node.start_position(),
    )
}

fn find_comments(
    node: Node,
    input: &str,
    comments: &mut Vec<(InputEdit, AnchoredComment)>,
) -> FormatterResult<()> {
    if is_comment(&node) {
        let commented = find_anchor(&node, input)?;
        // Build the corresponding InputEdit:
        // - If the comment is not alone on its line, return its bounds
        // - If the comment is alone on its line, return the bounds of all its line
        //   (we don't want to create undue blank lines)
        let prev = previous_disjoint_node(&node);
        let next = next_disjoint_node(&node);
        // Nested ifs are necessary, because `if let ...` don't play well with other clauses
        let edit: InputEdit = if let Some(prev) = prev {
            if let Some(next) = next {
                if prev.end_position().row() < node.start_position().row()
                    && node.end_position().row() < next.start_position().row()
                {
                    InputEdit::new(
                        prev.end_byte(),
                        node.end_byte(),
                        prev.end_byte(),
                        &prev.end_position(),
                        &node.end_position(),
                        &prev.end_position(),
                    )
                } else {
                    into_edit(&node)
                }
            } else {
                into_edit(&node)
            }
        } else {
            into_edit(&node)
        };
        comments.push((
            edit,
            AnchoredComment {
                comment_text: node.utf8_text(input.as_bytes())?.to_string(),
                original_column: node.start_position().column() as i32,
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
#[derive(Copy, Clone, Debug)]
pub enum Commented {
    /// The code section is before the comment, as in:
    /// ```
    /// struct Foo {
    ///     baz: Baz, // this is baz
    ///     quz: Qux, // this is qux
    /// }
    /// ```
    CommentedBefore(InputSection),
    /// The code section is after the comment, as in:
    /// ```
    /// struct Foo {
    ///     // let's have a baz
    ///     baz: Baz,
    ///     // and a qux
    ///     qux: Qux,
    /// }
    /// ```
    CommentedAfter(InputSection),
}

impl Diff<InputSection> for Commented {
    type ErrorType = FormatterError;

    fn subtract(&mut self, other: InputSection) -> FormatterResult<()> {
        match self {
            Commented::CommentedBefore(section) => section.subtract(other),
            Commented::CommentedAfter(section) => section.subtract(other),
        }
    }
}

fn next_disjoint_node<'tree>(starting_node: &'tree Node<'tree>) -> Option<Node<'tree>> {
    let mut node: Node<'tree> = starting_node.clone();
    // move up until we find a next sibling
    while node.next_sibling().is_none() {
        match node.parent() {
            None => return None,
            Some(parent) => node = parent,
        }
    }
    node.next_sibling()
}

fn previous_disjoint_node<'tree>(starting_node: &'tree Node<'tree>) -> Option<Node<'tree>> {
    let mut node: Node<'tree> = starting_node.clone();
    // move up until we find a previous sibling
    while node.prev_sibling().is_none() {
        match node.parent() {
            None => return None,
            Some(parent) => node = parent,
        }
    }
    node.prev_sibling()
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

// Use the following heuristics to find a comment's anchor:
// If the comment is only prefixed by blank symbols on its line, then the anchor is the
// next non-comment sibling node.
// Otherwise, the anchor is the previous non-comment sibling node.
// If there is no such node, we anchor to the first non-comment sibling node
// in the other direction.
#[allow(clippy::collapsible_else_if)]
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

#[derive(Clone, Debug)]
pub struct AnchoredComment {
    pub comment_text: String,
    // We need to keep track of the column for indentation purposes
    pub original_column: i32,
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
    tolerate_parsing_errors: bool,
) -> FormatterResult<SeparatedInput> {
    let mut anchors: Vec<(InputEdit, AnchoredComment)> = Vec::new();
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
    while let Some((edit, anchored_comment)) = anchors.pop() {
        // 1)
        new_input.replace_range(
            (edit.start_byte() as usize)..(edit.old_end_byte() as usize),
            "",
        );
        // 2)
        let section: InputSection = (&edit).into();
        edits.push(edit);
        // 3)
        anchored_comments.push(anchored_comment);
        anchored_comments = anchored_comments
            .iter()
            .map(
                |AnchoredComment {
                     mut commented,
                     original_column,
                     comment_text,
                 }|
                 -> FormatterResult<AnchoredComment> {
                    commented.subtract(section)?;
                    Ok(AnchoredComment {
                        commented,
                        original_column: *original_column,
                        comment_text: comment_text.to_string(),
                    })
                },
            )
            .collect::<FormatterResult<Vec<_>>>()?;
    }
    for edit in edits {
        new_tree.edit(&edit);
    }
    new_tree = parse(
        new_input.as_str(),
        grammar,
        tolerate_parsing_errors,
        Some(&new_tree),
    )?;
    Ok(SeparatedInput {
        input_tree: new_tree,
        input_string: new_input,
        comments: anchored_comments,
    })
}
