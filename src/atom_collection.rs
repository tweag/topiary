use std::collections::{BTreeSet, HashMap, HashSet};

use itertools::Itertools;
use tree_sitter::Node;

use crate::{Atom, FormatterError, FormatterResult};

#[derive(Debug)]
pub struct AtomCollection {
    atoms: Vec<Atom>,
    // node_start_indices: HashMap<usize, usize>,
    // node_end_indices: HashMap<usize, usize>,
    prepend: HashMap<usize, Vec<Atom>>,
    append: HashMap<usize, Vec<Atom>>,
    specified_leaf_nodes: BTreeSet<usize>,
    multi_line_nodes: HashSet<usize>,
    blank_lines_before: HashSet<usize>,
    line_break_before: HashSet<usize>,
    line_break_after: HashSet<usize>,
}

impl AtomCollection {
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

            // for node_id in parent_ids {
            //     self.update_start_index(node_id, self.atoms.len() - 1);
            //     self.update_end_index(node_id, self.atoms.len() - 1);
            // }
        } else {
            for child in node.children(&mut node.walk()) {
                self.collect_leafs_inner(child, source, &parent_ids, level + 1)?;
            }
        }

        Ok(())
    }

    pub fn resolve_capture(
        &mut self,
        name: String,
        node: Node,
        delimiter: Option<&str>,
    ) -> FormatterResult<()> {
        log::debug!("Resolving {name}");

        match name.as_ref() {
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
            "append_empty_softline" => self.append(Atom::Softline { spaced: false }, node),
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
            "append_spaced_softline" => self.append(Atom::Softline { spaced: true }, node),
            "prepend_empty_softline" => self.prepend(Atom::Softline { spaced: false }, node),
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
            "prepend_spaced_softline" => self.prepend(Atom::Softline { spaced: true }, node),
            // Skip over leafs
            _ => {}
        }

        Ok(())
    }

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

    pub fn post_process(&mut self) {
        // TODO: Make sure these aren't unnecessarily inefficient, in terms of
        // recreating a vector of atoms over and over.
        log::debug!("Before post-processing: {:?}", self.atoms);
        log::info!("Do post-processing");
        self.put_before(Atom::IndentEnd, Atom::Space, &[]);
        self.atoms = self.trim_following(Atom::Blankline, Atom::Space);
        self.put_before(Atom::Hardline, Atom::Blankline, &[Atom::Space]);
        self.put_before(Atom::IndentStart, Atom::Space, &[]);
        self.put_before(Atom::IndentStart, Atom::Hardline, &[Atom::Space]);
        self.put_before(Atom::IndentEnd, Atom::Hardline, &[Atom::Space]);
        self.atoms = self.trim_following(Atom::Hardline, Atom::Space);
        self.atoms = self.clean_up_consecutive(Atom::Space);
        self.atoms = self.clean_up_consecutive(Atom::Hardline);
        self.ensure_final_hardline();
        log::debug!("Final list of atoms: {:?}", self.atoms);
    }

    // fn update_start_index(&mut self, node_id: usize, index: usize) {
    //     self.node_start_indices
    //         .entry(node_id)
    //         .and_modify(|i| {
    //             if index < *i {
    //                 *i = index;
    //             }
    //         })
    //         .or_insert(index);
    // }

    // fn update_end_index(&mut self, node_id: usize, index: usize) {
    //     self.node_end_indices
    //         .entry(node_id)
    //         .and_modify(|i| {
    //             if index > *i {
    //                 *i = index;
    //             }
    //         })
    //         .or_insert(index);
    // }

    fn prepend(&mut self, atom: Atom, node: Node) {
        if let Some(atom) = self.expand_softline(atom, node) {
            // TODO: Pre-populate these
            let target_node = first_leaf(node);
            self.prepend
                .entry(target_node.id())
                .and_modify(|atoms| atoms.push(atom.clone()))
                .or_insert_with(|| vec![atom]);
            // let index = find_node(target_node, atoms);
            // let index = self.node_start_indices[&node.id()];
            // self.atoms.insert(index, atom);
            // shift_indices(&mut self.node_start_indices, index);
            // shift_indices(&mut self.node_end_indices, index);
        }
    }

    fn append(&mut self, atom: Atom, node: Node) {
        if let Some(atom) = self.expand_softline(atom, node) {
            let target_node = last_leaf(node);

            // If this is a child of a node that we have deemed as a leaf node
            // (e.g. a character in a string), we need to use that node id
            // instead.
            let target_node = self.parent_leaf_node(target_node);

            log::debug!(
                "Appending {atom:?} to node {:?}, id {}.",
                target_node,
                target_node.id()
            );
            self.append
                .entry(target_node.id())
                .and_modify(|atoms| atoms.push(atom.clone()))
                .or_insert_with(|| vec![atom]);
            // let index = find_node(target_node, atoms);
            // let index = self.node_end_indices[&node.id()];
            // if index > self.atoms.len() {
            //     self.atoms.push(atom);
            // } else {
            //     self.atoms.insert(index + 1, atom);
            //     shift_indices(&mut self.node_start_indices, index + 1);
            //     shift_indices(&mut self.node_end_indices, index + 1);
            // }
        }
    }

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

    fn clean_up_consecutive(&self, atom: Atom) -> Vec<Atom> {
        let filtered = self
            .atoms
            .split(|a| *a == atom)
            .filter(|chain| !chain.is_empty());

        Itertools::intersperse(filtered, &[atom.clone()])
            .flatten()
            .cloned()
            .collect_vec()
    }

    fn trim_following(&self, delimiter: Atom, skip: Atom) -> Vec<Atom> {
        let trimmed = self
            .atoms
            .split(|a| *a == delimiter)
            .map(|slice| slice.iter().skip_while(|a| **a == skip).collect::<Vec<_>>());

        Itertools::intersperse(trimmed, vec![&delimiter])
            .flatten()
            .cloned()
            .collect_vec()
    }

    fn put_before(&mut self, before: Atom, after: Atom, ignoring: &[Atom]) {
        for i in 0..self.atoms.len() - 1 {
            if self.atoms[i] == after {
                for j in i + 1..self.atoms.len() {
                    if self.atoms[j] != before
                        && self.atoms[j] != after
                        && !ignoring.contains(&self.atoms[j])
                    {
                        // stop looking
                        break;
                    }
                    if self.atoms[j] == before {
                        // switch
                        self.atoms[i] = before.clone();
                        self.atoms[j] = after.clone();
                        break;
                    }
                }
            }
        }
    }

    fn ensure_final_hardline(&mut self) {
        if let Some(Atom::Hardline) = self.atoms.last() {
        } else {
            self.atoms.push(Atom::Hardline);
        }
    }
}

impl<Idx> std::ops::Index<Idx> for AtomCollection
where
    Idx: std::slice::SliceIndex<[Atom]>,
{
    type Output = Idx::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.atoms[index]
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

// fn shift_indices(map: &mut HashMap<usize, usize>, shift_point: usize) {
//     for (_, index) in map.iter_mut() {
//         if *index >= shift_point {
//             *index += 1;
//         }
//     }
// }

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
