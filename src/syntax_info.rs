use crate::{error::FormatterError, Atom, FormatterResult};
use std::collections::HashSet;
use tree_sitter::Node;

pub struct SyntaxInfo {
    pub multi_line_nodes: HashSet<usize>,
    pub blank_lines_before: HashSet<usize>,
    pub line_break_before: HashSet<usize>,
    pub line_break_after: HashSet<usize>,
}

impl SyntaxInfo {
    pub fn detect(root: Node) -> SyntaxInfo {
        // Detect user specified line breaks
        let multi_line_nodes = detect_multi_line_nodes(root);
        let blank_lines_before = detect_blank_lines_before(root);
        let (line_break_before, line_break_after) = detect_line_break_before_and_after(root);

        SyntaxInfo {
            multi_line_nodes,
            blank_lines_before,
            line_break_before,
            line_break_after,
        }
    }

    pub fn resolve_capture(
        &self,
        name: String,
        atoms: &mut Vec<Atom>,
        node: Node,
        delimiter: Option<&str>,
    ) -> FormatterResult<()> {
        log::debug!("Resolving {name}");

        match name.as_ref() {
            "allow_blank_line_before" => {
                if self.blank_lines_before.contains(&node.id()) {
                    self.atoms_prepend(Atom::Blankline, node, atoms);
                }
            }
            "append_delimiter" => self.atoms_append(
                Atom::Literal(
                    delimiter
                        .ok_or_else(|| {
                            FormatterError::Query(
                                "@append_delimiter requires a #delimiter! predicate".into(),
                                None,
                            )
                        })?
                        .to_string(),
                ),
                node,
                atoms,
            ),
            "append_empty_softline" => {
                self.atoms_append(Atom::Softline { spaced: false }, node, atoms)
            }
            "append_hardline" => self.atoms_append(Atom::Hardline, node, atoms),
            "append_indent_start" => self.atoms_append(Atom::IndentStart, node, atoms),
            "append_indent_end" => self.atoms_append(Atom::IndentEnd, node, atoms),
            "append_input_softline" => {
                let space = if self.line_break_after.contains(&node.id()) {
                    Atom::Hardline
                } else {
                    Atom::Space
                };

                self.atoms_append(space, node, atoms);
            }
            "append_space" => self.atoms_append(Atom::Space, node, atoms),
            "append_spaced_softline" => {
                self.atoms_append(Atom::Softline { spaced: true }, node, atoms)
            }
            "prepend_empty_softline" => {
                self.atoms_prepend(Atom::Softline { spaced: false }, node, atoms)
            }
            "prepend_indent_start" => self.atoms_prepend(Atom::IndentStart, node, atoms),
            "prepend_indent_end" => self.atoms_prepend(Atom::IndentEnd, node, atoms),
            "prepend_input_softline" => {
                let space = if self.line_break_before.contains(&node.id()) {
                    Atom::Hardline
                } else {
                    Atom::Space
                };

                self.atoms_prepend(space, node, atoms);
            }
            "prepend_space" => self.atoms_prepend(Atom::Space, node, atoms),
            "prepend_spaced_softline" => {
                self.atoms_prepend(Atom::Softline { spaced: true }, node, atoms)
            }
            // Skip over leafs
            _ => {}
        }

        Ok(())
    }

    fn atoms_append(&self, atom: Atom, node: Node, atoms: &mut Vec<Atom>) {
        if let Some(atom) = self.expand_softline(atom, node) {
            let target_node = last_leaf(node);
            let index = find_node(target_node, atoms);
            if index > atoms.len() {
                atoms.push(atom);
            } else {
                atoms.insert(index + 1, atom);
            }
        }
    }

    fn atoms_prepend(&self, atom: Atom, node: Node, atoms: &mut Vec<Atom>) {
        if let Some(atom) = self.expand_softline(atom, node) {
            let target_node = first_leaf(node);
            let index = find_node(target_node, atoms);
            atoms.insert(index, atom);
        }
    }

    fn expand_softline(&self, atom: Atom, node: Node) -> Option<Atom> {
        if let Atom::Softline { spaced } = atom {
            if let Some(parent) = node.parent() {
                let parent_id = parent.id();

                if self.multi_line_nodes.contains(&parent_id) {
                    log::debug!(
                        "Expanding softline to hardline in node {:?} with parent {}: {:?}",
                        node,
                        parent_id,
                        parent
                    );
                    Some(Atom::Hardline)
                } else if spaced {
                    log::debug!(
                        "Expanding softline to space in node {:?} with parent {}: {:?}",
                        node,
                        parent_id,
                        parent
                    );
                    Some(Atom::Space)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            Some(atom)
        }
    }
}

fn detect_multi_line_nodes(node: Node) -> HashSet<usize> {
    let mut ids = HashSet::new();

    for child in node.children(&mut node.walk()) {
        ids.extend(detect_multi_line_nodes(child));
    }

    let start_line = node.start_position().row;
    let end_line = node.end_position().row;

    if end_line > start_line {
        let id = node.id();
        ids.insert(id);
        log::debug!("Multi-line node {}: {:?}", id, node,);
    }

    ids
}

fn detect_blank_lines_before(node: Node) -> HashSet<usize> {
    detect_line_breaks_inner(node, 2, &mut None).0
}

fn detect_line_break_before_and_after(node: Node) -> (HashSet<usize>, HashSet<usize>) {
    detect_line_breaks_inner(node, 1, &mut None)
}

fn detect_line_breaks_inner<'a>(
    node: Node<'a>,
    minimum_line_breaks: usize,
    previous_node: &mut Option<Node<'a>>,
) -> (HashSet<usize>, HashSet<usize>) {
    let mut nodes_with_breaks_before = HashSet::new();
    let mut nodes_with_breaks_after = HashSet::new();

    if let Some(previous_node) = previous_node {
        let previous_end = previous_node.end_position().row;
        let current_start = node.start_position().row;

        if current_start >= previous_end + minimum_line_breaks {
            nodes_with_breaks_before.insert(node.id());
            nodes_with_breaks_after.insert(previous_node.id());

            log::debug!(
                "There are at least {} blank lines between {:?} and {:?}",
                minimum_line_breaks,
                previous_node,
                node
            );
        }
    }

    *previous_node = Some(node);

    for child in node.children(&mut node.walk()) {
        let (before, after) = detect_line_breaks_inner(child, minimum_line_breaks, previous_node);
        nodes_with_breaks_before.extend(before);
        nodes_with_breaks_after.extend(after);
    }

    (nodes_with_breaks_before, nodes_with_breaks_after)
}

/// Given a node, returns the id of the first leaf in the subtree.
fn first_leaf(node: Node) -> Node {
    if node.child_count() == 0 {
        node
    } else {
        first_leaf(node.child(0).unwrap())
    }
}

/// Given a node, returns the id of the last leaf in the subtree.
fn last_leaf(node: Node) -> Node {
    let nr_children = node.child_count();
    if nr_children == 0 {
        node
    } else {
        last_leaf(node.child(nr_children - 1).unwrap())
    }
}

/// Finds the matching node in the atoms and returns the index
/// TODO: Error
fn find_node(node: Node, atoms: &mut [Atom]) -> usize {
    let mut target_node = node;
    loop {
        for (i, n) in atoms.iter().enumerate() {
            match n {
                Atom::Leaf { id, .. } => {
                    if *id == target_node.id() {
                        return i;
                    }
                }
                _ => continue,
            }
        }

        target_node = match target_node.parent() {
            Some(p) => p,
            None => unreachable!(),
        }
    }
}
