use std::collections::{BTreeSet, HashMap, HashSet};

use tree_sitter::Node;

use crate::{Atom, FormatterError, FormatterResult};

#[derive(Debug)]
pub struct AtomCollection {
    atoms: Vec<Atom>,
    prepend: HashMap<usize, Vec<Atom>>,
    append: HashMap<usize, Vec<Atom>>,
    specified_leaf_nodes: BTreeSet<usize>,
    multi_line_nodes: HashSet<usize>,
    blank_lines_before: HashSet<usize>,
    line_break_before: HashSet<usize>,
    line_break_after: HashSet<usize>,
}

// The purpose of this function is to identify the node which plays the role of parent,
// when a flat tree is represented as a binary tree.
// For instance, if one has a tree of the form
// (op
//   (num 1)
//   (symbol +)
//   (op
//     (num 2)
//     (symbol +)
//     (num 3)
//   )
// )
// One could like to see it as an addition with 3 terms
// rather than an addition nested under another one.
fn earliest_ancestor_of_same_type(node: Node) -> Node {
    if let Some(parent) = node.parent() {
        if parent.kind() == node.kind() {
            earliest_ancestor_of_same_type(parent)
        } else {
            node
        }
    } else {
        node
    }
}

impl AtomCollection {
    /// Use this to create an initial AtomCollection
    pub fn collect_leafs(
        root: Node,
        source: &[u8],
        specified_leaf_nodes: BTreeSet<usize>,
    ) -> FormatterResult<AtomCollection> {
        // Detect user specified line breaks
        let multi_line_nodes = detect_multi_line_nodes(root);
        let blank_lines_before = detect_blank_lines_before(root);
        let (line_break_before, line_break_after) = detect_line_break_before_and_after(root);

        let mut atoms = AtomCollection {
            atoms: Vec::new(),
            prepend: HashMap::new(),
            append: HashMap::new(),
            specified_leaf_nodes,
            multi_line_nodes,
            blank_lines_before,
            line_break_before,
            line_break_after,
        };

        atoms.collect_leafs_inner(root, source, &Vec::new(), 0)?;

        Ok(atoms)
    }

    /// This gets called a lot during query processing, and needs to be efficient.
    pub fn resolve_capture(
        &mut self,
        name: &str,
        node: Node,
        delimiter: Option<&str>,
    ) -> FormatterResult<()> {
        log::debug!("Resolving {name}");

        match name {
            "allow_blank_line_before" => {
                if self.blank_lines_before.contains(&node.id()) {
                    self.prepend(Atom::Blankline, node);
                }
            }
            "append_delimiter" => self.append(
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
            ),
            "append_empty_softline" => self.append(
                Atom::Softline {
                    spaced: false,
                    considering_ancestor: false,
                },
                node,
            ),
            "append_hardline" => self.append(Atom::Hardline, node),
            "append_indent_start" => self.append(Atom::IndentStart, node),
            "append_indent_end" => self.append(Atom::IndentEnd, node),
            "append_input_softline" => {
                let space = if self.line_break_after.contains(&node.id()) {
                    Atom::Hardline
                } else {
                    Atom::Space
                };

                self.append(space, node);
            }
            "append_space" => self.append(Atom::Space, node),
            "append_spaced_softline" => self.append(
                Atom::Softline {
                    spaced: true,
                    considering_ancestor: false,
                },
                node,
            ),
            "append_empty_soft_ancestor_line" => self.append(
                Atom::Softline {
                    spaced: false,
                    considering_ancestor: true,
                },
                node,
            ),
            "append_spaced_soft_ancestor_line" => self.append(
                Atom::Softline {
                    spaced: true,
                    considering_ancestor: true,
                },
                node,
            ),
            "prepend_delimiter" => self.prepend(
                Atom::Literal(
                    delimiter
                        .ok_or_else(|| {
                            FormatterError::Query(
                                "@prepend_delimiter requires a #delimiter! predicate".into(),
                                None,
                            )
                        })?
                        .to_string(),
                ),
                node,
            ),
            "prepend_empty_softline" => self.prepend(
                Atom::Softline {
                    spaced: false,
                    considering_ancestor: false,
                },
                node,
            ),
            "prepend_indent_start" => self.prepend(Atom::IndentStart, node),
            "prepend_indent_end" => self.prepend(Atom::IndentEnd, node),
            "prepend_input_softline" => {
                let space = if self.line_break_before.contains(&node.id()) {
                    Atom::Hardline
                } else {
                    Atom::Space
                };

                self.prepend(space, node);
            }
            "prepend_space" => self.prepend(Atom::Space, node),
            "prepend_spaced_softline" => self.prepend(
                Atom::Softline {
                    spaced: true,
                    considering_ancestor: false,
                },
                node,
            ),
            "prepend_empty_soft_ancestor_line" => self.prepend(
                Atom::Softline {
                    spaced: false,
                    considering_ancestor: true,
                },
                node,
            ),
            "prepend_spaced_soft_ancestor_line" => self.prepend(
                Atom::Softline {
                    spaced: true,
                    considering_ancestor: true,
                },
                node,
            ),
            // Skip over leafs
            "leaf" => {}

            // Return a query parsing error on unknown capture names
            unknown => {
                return Err(FormatterError::Query(
                    format!("@{unknown} is not a valid capture name"),
                    None,
                ))
            }
        }

        Ok(())
    }

    /// After query processing is done, a flattened/expanded vector of atoms can be created.
    pub fn apply_prepends_and_appends(&mut self) {
        let mut expanded: Vec<Atom> = Vec::new();

        for atom in self.atoms.iter() {
            match atom {
                Atom::Leaf { id, .. } => {
                    for prepended in self.prepend.entry(*id).or_default() {
                        expanded.push(prepended.clone());
                    }

                    expanded.push(atom.clone());

                    for appended in self.append.entry(*id).or_default() {
                        log::debug!("Applying append of {appended:?} to {atom:?}.");
                        expanded.push(appended.clone());
                    }
                }
                _ => {
                    log::debug!("Not a leaf: {atom:?}");
                    expanded.push(atom.clone());
                }
            }
        }

        self.atoms = expanded;
    }

    fn collect_leafs_inner(
        &mut self,
        node: Node,
        source: &[u8],
        parent_ids: &[usize],
        level: usize,
    ) -> FormatterResult<()> {
        let id = node.id();
        let parent_ids = [parent_ids, &[id]].concat();

        log::debug!(
            "CST node: {}{:?} - Named: {}",
            "  ".repeat(level),
            node,
            node.is_named()
        );

        if node.child_count() == 0 || self.specified_leaf_nodes.contains(&node.id()) {
            self.atoms.push(Atom::Leaf {
                content: String::from(node.utf8_text(source)?),
                id,
            });
        } else {
            for child in node.children(&mut node.walk()) {
                self.collect_leafs_inner(child, source, &parent_ids, level + 1)?;
            }
        }

        Ok(())
    }

    fn prepend(&mut self, atom: Atom, node: Node) {
        if let Some(atom) = self.expand_softline(atom, node) {
            // TODO: Pre-populate these
            let target_node = first_leaf(node);

            // If this is a child of a node that we have deemed as a leaf node
            // (e.g. a character in a string), we need to use that node id
            // instead.
            let target_node = self.parent_leaf_node(target_node);

            log::debug!("Prepending {atom:?} to node {:?}", target_node,);

            self.prepend
                .entry(target_node.id())
                .and_modify(|atoms| atoms.push(atom.clone()))
                .or_insert_with(|| vec![atom]);
        }
    }

    fn append(&mut self, atom: Atom, node: Node) {
        if let Some(atom) = self.expand_softline(atom, node) {
            let target_node = last_leaf(node);

            // If this is a child of a node that we have deemed as a leaf node
            // (e.g. a character in a string), we need to use that node id
            // instead.
            let target_node = self.parent_leaf_node(target_node);

            log::debug!("Appending {atom:?} to node {:?}", target_node,);

            self.append
                .entry(target_node.id())
                .and_modify(|atoms| atoms.push(atom.clone()))
                .or_insert_with(|| vec![atom]);
        }
    }

    // TODO: The frequent lookup of this is inefficient, and needs to be optimized.
    fn parent_leaf_node<'a>(&self, node: Node<'a>) -> Node<'a> {
        let mut n = node;

        while let Some(parent) = n.parent() {
            n = parent;

            if self.specified_leaf_nodes.contains(&n.id()) {
                return n;
            }
        }

        node
    }

    fn expand_softline(&self, atom: Atom, node: Node) -> Option<Atom> {
        if let Atom::Softline {
            spaced,
            considering_ancestor,
        } = atom
        {
            if let Some(parent) = node.parent() {
                let ancestor = if considering_ancestor {
                    earliest_ancestor_of_same_type(parent)
                } else {
                    parent
                };
                let ancestor_id = ancestor.id();

                if self.multi_line_nodes.contains(&ancestor_id) {
                    log::debug!(
                        "Expanding softline to hardline in node {:?} with ancestor {}: {:?}",
                        node,
                        ancestor_id,
                        ancestor
                    );
                    Some(Atom::Hardline)
                } else if spaced {
                    log::debug!(
                        "Expanding softline to space in node {:?} with ancestor {}: {:?}",
                        node,
                        ancestor_id,
                        ancestor
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

    // This function merges the spaces, new lines and blank lines.
    // If there are several tokens of different kind one after the other,
    // the blank line is kept over the new line which itself is kept over the space.
    // Furthermore, this function put the indentation delimiters before any space/line atom.
    pub fn post_process(&mut self) {
        let mut new_vec: Vec<Atom> = Vec::new();
        for next in &(self.atoms) {
            if let Some(prev_var) = new_vec.last() {
                let prev = prev_var.clone();
                post_process_internal(&mut new_vec, prev, next.clone())
            } else {
                // If the new vector is still empty,
                // we skip all the spaces and newlines
                // and add the first significant atom to the new vector.
                match next {
                    Atom::Space | Atom::Hardline | Atom::Blankline => {}
                    _ => new_vec.push(next.clone()),
                };
            }
        }
        ensure_final_hardline(&mut new_vec);
        self.atoms = new_vec;
    }
}

fn post_process_internal(new_vec: &mut Vec<Atom>, prev: Atom, next: Atom) {
    match prev {
        // If the last atom is a space/line
        Atom::Space | Atom::Hardline | Atom::Blankline => {
            match next {
                // And the next one is also a space/line
                Atom::Space | Atom::Hardline | Atom::Blankline => {
                    if is_dominant(next.clone(), prev) {
                        new_vec.pop();
                        new_vec.push(next);
                    }
                }
                // Or an indentation delimiter, then one has to merge/re-order.
                Atom::IndentStart | Atom::IndentEnd => {
                    new_vec.pop();
                    new_vec.push(next);
                    new_vec.push(prev);
                }
                _ => new_vec.push(next),
            }
        }
        // Otherwise, we simply copy the atom to the new vector.
        _ => new_vec.push(next),
    }
}

fn ensure_final_hardline(v: &mut Vec<Atom>) {
    if let Some(Atom::Hardline) = v.last() {
    } else {
        v.push(Atom::Hardline);
    }
}

// This function is only expected to take spaces and newlines as argument.
// It defines the order Blankline > Hardline > Space.
fn is_dominant(next: Atom, prev: Atom) -> bool {
    match next {
        Atom::Space => false,
        Atom::Hardline => prev == Atom::Space,
        Atom::Blankline => prev != Atom::Blankline,
        _ => panic!("Unexpected character in is_dominant"),
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

// TODO: This is taking a bit too much time, and would benefit from an
// optimization.
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

/// So that we can easily extract the atoms using &atom_collection[..]
impl<Idx> std::ops::Index<Idx> for AtomCollection
where
    Idx: std::slice::SliceIndex<[Atom]>,
{
    type Output = Idx::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.atoms[index]
    }
}
