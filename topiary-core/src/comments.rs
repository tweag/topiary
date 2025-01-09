use std::collections::HashSet;

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
    comment_ids: &HashSet<usize>,
    comments: &mut Vec<(InputEdit, AnchoredComment)>,
) -> FormatterResult<()> {
    if comment_ids.contains(&node.id()) {
        let commented = find_anchor(&node, input, comment_ids)?;
        // Build the corresponding InputEdit:
        // - If the comment is not alone on its line, return its bounds
        // - If the comment is alone on its line, return the bounds of all its line
        //   (we don't want to create undue blank lines)
        let prev = previous_disjoint_node(&node);
        let next = next_disjoint_node(&node);
        let is_alone_before = prev
            .as_ref()
            .map(|n| n.start_position().row() < node.start_position().row());
        let is_alone_after = next
            .as_ref()
            .map(|n| node.end_position().row() < n.start_position().row());

        // The logic is a bit complex here. Each case gives an example of a comment it would match
        let edit: InputEdit = match (is_alone_before, is_alone_after) {
            // /* define a foo */ let _ = foo
            (Some(false), _) |
            // let _ = foo /* we defined a foo */
            (_, Some(false)) |
            // /* this file has a comment and nothing else */
            (None, None) => {
                into_edit(&node)
            },
            // let _ = foo
            // /* This comment is alone on its line, but has stuff before and after */
            // let _ = bar
            (Some(true), Some(true)) |
            // /* This is the first line of the file */
            // let _ = foo
            (None, Some(true)) => {
                InputEdit::new(
                    node.start_byte(),
                    next.as_ref().unwrap().start_byte(),
                    node.start_byte(),
                    &node.start_position(),
                    &next.as_ref().unwrap().start_position(),
                    &node.start_position()
                )
            },
            // let _ = foo
            // /* This is the last line of the file */
            (Some(true), None) => {
                InputEdit::new(
                    prev.as_ref().unwrap().end_byte(),
                    node.end_byte(),
                    prev.as_ref().unwrap().end_byte(),
                    &prev.as_ref().unwrap().end_position(),
                    &node.end_position(),
                    &prev.as_ref().unwrap().end_position(),
                )
            }
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
            find_comments(child, input, comment_ids, comments)?;
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
    CommentedAfter {
        section: InputSection,
        /// Whether or not there is a blank line before/after the comment, as in:
        /// ```
        /// // The previous section was tiring, let's rest a little
        ///
        /// // Alright, back to the code:
        /// let foo = 1;
        /// ```
        blank_line_after: bool,
        blank_line_before: bool,
    },
}

impl Diff<InputSection> for Commented {
    type ErrorType = FormatterError;

    fn subtract(&mut self, other: InputSection) -> FormatterResult<()> {
        match self {
            Commented::CommentedBefore(section) => section.subtract(other),
            Commented::CommentedAfter { section, .. } => section.subtract(other),
        }
    }
}

/// A comment, as part of Topiary's output.
/// We forget node information here, because the struct
/// is supposed to be attached to the node it comments.
#[derive(Debug)]
pub struct Comment {
    pub content: String,
    pub original_column: i32,
    pub blank_line_after: bool,
    pub blank_line_before: bool,
}

impl From<&AnchoredComment> for Comment {
    fn from(value: &AnchoredComment) -> Self {
        Comment {
            content: value.comment_text.clone(),
            original_column: value.original_column,
            blank_line_after: match value.commented {
                Commented::CommentedBefore(_) => false,
                Commented::CommentedAfter {
                    blank_line_after, ..
                } => blank_line_after,
            },
            blank_line_before: match value.commented {
                Commented::CommentedBefore(_) => false,
                Commented::CommentedAfter {
                    blank_line_before, ..
                } => blank_line_before,
            },
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
fn next_non_comment_leaf<'tree>(
    starting_node: Node<'tree>,
    comment_ids: &HashSet<usize>,
) -> Option<Node<'tree>> {
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
                    if comment_ids.contains(&node.id()) {
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
            if comment_ids.contains(&child.id()) {
                break;
            } else {
                node = child
            }
        }
        // check if the leaf is a comment. If it is not, start over again.
        if comment_ids.contains(&node.id()) {
            continue;
        } else {
            return Some(node);
        }
    }
}

// TODO: if performance is an issue, use TreeCursor to navigate the tree
fn previous_non_comment_leaf<'tree>(
    starting_node: Node<'tree>,
    comment_ids: &HashSet<usize>,
) -> Option<Node<'tree>> {
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
                    if comment_ids.contains(&node.id()) {
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
            if comment_ids.contains(&child.id()) {
                break;
            } else {
                node = child
            }
        }
        // check if the leaf is a comment. If it is not, start over again.
        if comment_ids.contains(&node.id()) {
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
fn find_anchor<'tree>(
    node: &'tree Node<'tree>,
    input: &str,
    comment_ids: &HashSet<usize>,
) -> FormatterResult<Commented> {
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
        if let Some(anchor) = next_non_comment_leaf(node.clone(), comment_ids) {
            let prev = previous_disjoint_node(node);
            let next = next_disjoint_node(node);
            Ok(Commented::CommentedAfter {
                section: (&anchor).into(),
                blank_line_after: next
                    .map(|next| next.start_position().row() > node.end_position().row() + 1)
                    .unwrap_or(false),
                blank_line_before: prev
                    .map(|prev| prev.end_position().row() + 1 < node.start_position().row())
                    .unwrap_or(false),
            })
        } else if let Some(anchor) = previous_non_comment_leaf(node.clone(), comment_ids) {
            Ok(Commented::CommentedBefore((&anchor).into()))
        } else {
            Err(FormatterError::CommentOrphaned(
                node.utf8_text(input.as_bytes())?.to_string(),
            ))
        }
    } else {
        if let Some(anchor) = previous_non_comment_leaf(node.clone(), comment_ids) {
            Ok(Commented::CommentedBefore((&anchor).into()))
        } else if let Some(anchor) = next_non_comment_leaf(node.clone(), comment_ids) {
            let prev = previous_disjoint_node(node);
            let next = next_disjoint_node(node);
            Ok(Commented::CommentedAfter {
                section: (&anchor).into(),
                blank_line_after: next
                    .map(|next| next.start_position().row() > node.end_position().row() + 1)
                    .unwrap_or(false),
                blank_line_before: prev
                    .map(|prev| prev.end_position().row() + 1 < node.start_position().row())
                    .unwrap_or(false),
            })
        } else {
            Err(FormatterError::CommentOrphaned(
                node.utf8_text(input.as_bytes())?.to_string(),
            ))
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnchoredComment {
    pub comment_text: String,
    // We need to keep track of the column for indentation purposes
    pub original_column: i32,
    pub commented: Commented,
}

impl From<&AnchoredComment> for InputSection {
    fn from(value: &AnchoredComment) -> Self {
        match value.commented {
            Commented::CommentedBefore(section) => section,
            Commented::CommentedAfter { section, .. } => section,
        }
    }
}

pub struct SeparatedInput {
    pub input_tree: Tree,
    pub input_string: String,
    pub comments: Vec<AnchoredComment>,
}

pub fn extract_comments<'a>(
    tree: &'a Tree,
    input: &'a str,
    comment_ids: HashSet<usize>,
    grammar: &Language,
    tolerate_parsing_errors: bool,
) -> FormatterResult<SeparatedInput> {
    let mut anchors: Vec<(InputEdit, AnchoredComment)> = Vec::new();
    let mut anchored_comments: Vec<AnchoredComment> = Vec::new();
    let mut new_input: String = input.to_string();
    let mut new_tree: Tree = tree.clone();
    find_comments(tree.root_node(), input, &comment_ids, &mut anchors)?;
    anchors.sort_by_key(|(node, _)| node.start_byte());
    let mut edits: Vec<InputEdit> = Vec::new();
    // for each (comment, anchor) pair in reverse order, we:
    // 1) remove the comment from the input,
    // 2) register an InputEdit to modify the tree,
    // 3) edit all anchors to account for the removed comment.
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
        edits.push(edit);
        anchored_comments.push(anchored_comment);
    }
    for edit in edits {
        new_tree.edit(&edit);
        // 3)
        let section: InputSection = (&edit).into();
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
