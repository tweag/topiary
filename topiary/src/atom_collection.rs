use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

use tree_sitter_facade::Node;

use crate::{Atom, FormatterError, FormatterResult, ScopeCondition};

#[derive(Debug)]
pub struct AtomCollection {
    atoms: Vec<Atom>,
    prepend: HashMap<usize, Vec<Atom>>,
    append: HashMap<usize, Vec<Atom>>,
    specified_leaf_nodes: HashSet<usize>,
    parent_leaf_nodes: HashMap<usize, usize>,
    multi_line_nodes: HashSet<usize>,
    blank_lines_before: HashSet<usize>,
    line_break_before: HashSet<usize>,
    line_break_after: HashSet<usize>,
    /// The semantics of the types of scope_begin and scope_end is
    // HashMap<leaf_node_id, (line_number, Vec<scope_id>)>
    // The line number is passed here because otherwise the information
    // is lost at post-processing time.
    scope_begin: HashMap<usize, (u32, Vec<String>)>,
    scope_end: HashMap<usize, (u32, Vec<String>)>,
    /// Used to generate unique IDs
    counter: usize,
}

impl AtomCollection {
    /// Use this to create an initial AtomCollection
    pub fn collect_leafs(
        root: &Node,
        source: &[u8],
        specified_leaf_nodes: HashSet<usize>,
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
            parent_leaf_nodes: HashMap::new(),
            multi_line_nodes,
            blank_lines_before,
            line_break_before,
            line_break_after,
            scope_begin: HashMap::new(),
            scope_end: HashMap::new(),
            counter: 0,
        };

        atoms.collect_leafs_inner(root, source, &Vec::new(), 0)?;

        Ok(atoms)
    }

    // wrap inside a conditional atom if #single/multi_line_scope_only! is set
    fn wrap(&mut self, atom: Atom, predicates: &QueryPredicates) -> Atom {
        if let Some(scope_id) = &predicates.single_line_scope_only {
            let id = self.next_id();
            Atom::ScopedConditional {
                id,
                scope_id: scope_id.to_string(),
                condition: ScopeCondition::SingleLineOnly,
                atom: Box::new(atom),
            }
        } else if let Some(scope_id) = &predicates.multi_line_scope_only {
            let id = self.next_id();
            Atom::ScopedConditional {
                id,
                scope_id: scope_id.to_string(),
                condition: ScopeCondition::MultiLineOnly,
                atom: Box::new(atom),
            }
        } else {
            atom
        }
    }

    /// This gets called a lot during query processing, and needs to be efficient.
    pub fn resolve_capture(
        &mut self,
        name: &str,
        node: &Node,
        predicates: &QueryPredicates,
    ) -> FormatterResult<()> {
        log::debug!("Resolving {name}");

        let requires_delimiter = || {
            predicates.delimiter.as_deref().ok_or_else(|| {
                FormatterError::Query(format!("@{name} requires a #delimiter! predicate"), None)
            })
        };
        let requires_scope_id = || {
            predicates.scope_id.as_deref().ok_or_else(|| {
                FormatterError::Query(format!("@{name} requires a #scope_id! predicate"), None)
            })
        };

        let mut is_multi_line = false;
        if let Some(parent) = node.parent() {
            let parent_id = parent.id();
            if self.multi_line_nodes.contains(&parent_id) {
                is_multi_line = true
            }
        }
        if is_multi_line && predicates.single_line_only {
            log::debug!("Aborting because context is multi-line and #single_line_only! is set");
            return Ok(());
        }
        if !is_multi_line && predicates.multi_line_only {
            log::debug!("Aborting because context is single-line and #multi_line_only! is set");
            return Ok(());
        }

        match name {
            "allow_blank_line_before" => {
                if self.blank_lines_before.contains(&node.id()) {
                    self.prepend(Atom::Blankline, node, predicates);
                }
            }
            "append_delimiter" => self.append(
                Atom::Literal(requires_delimiter()?.to_string()),
                node,
                predicates,
            ),
            "append_empty_softline" => {
                self.append(Atom::Softline { spaced: false }, node, predicates)
            }
            "append_hardline" => self.append(Atom::Hardline, node, predicates),
            "append_indent_start" => self.append(Atom::IndentStart, node, predicates),
            "append_indent_end" => self.append(Atom::IndentEnd, node, predicates),
            "append_input_softline" => {
                let space = if self.line_break_after.contains(&node.id()) {
                    Atom::Hardline
                } else {
                    Atom::Space
                };

                self.append(space, node, predicates);
            }
            "append_space" => self.append(Atom::Space, node, predicates),
            "append_antispace" => self.append(Atom::Antispace, node, predicates),
            "append_spaced_softline" => {
                self.append(Atom::Softline { spaced: true }, node, predicates)
            }
            "prepend_delimiter" => self.prepend(
                Atom::Literal(requires_delimiter()?.to_string()),
                node,
                predicates,
            ),
            "prepend_empty_softline" => {
                self.prepend(Atom::Softline { spaced: false }, node, predicates)
            }
            "prepend_hardline" => self.prepend(Atom::Hardline, node, predicates),
            "prepend_indent_start" => self.prepend(Atom::IndentStart, node, predicates),
            "prepend_indent_end" => self.prepend(Atom::IndentEnd, node, predicates),
            "prepend_input_softline" => {
                let space = if self.line_break_before.contains(&node.id()) {
                    Atom::Hardline
                } else {
                    Atom::Space
                };

                self.prepend(space, node, predicates);
            }
            "prepend_space" => self.prepend(Atom::Space, node, predicates),
            "prepend_antispace" => self.prepend(Atom::Antispace, node, predicates),
            "prepend_spaced_softline" => {
                self.prepend(Atom::Softline { spaced: true }, node, predicates)
            }
            // Skip over leafs
            "leaf" => {}
            // Deletion
            "delete" => {
                self.prepend(Atom::DeleteBegin, node, predicates);
                self.append(Atom::DeleteEnd, node, predicates)
            }
            // Scope manipulation
            "begin_scope" => self.begin_scope_before(node, requires_scope_id()?),
            "end_scope" => self.end_scope_after(node, requires_scope_id()?),
            // Scoped softlines
            "append_empty_scoped_softline" => {
                let id = self.next_id();
                self.append(
                    Atom::ScopedSoftline {
                        id,
                        scope_id: requires_scope_id()?.to_string(),
                        spaced: false,
                    },
                    node,
                    predicates,
                )
            }
            "append_spaced_scoped_softline" => {
                let id = self.next_id();
                self.append(
                    Atom::ScopedSoftline {
                        id,
                        scope_id: requires_scope_id()?.to_string(),
                        spaced: true,
                    },
                    node,
                    predicates,
                )
            }
            "prepend_empty_scoped_softline" => {
                let id = self.next_id();
                self.prepend(
                    Atom::ScopedSoftline {
                        id,
                        scope_id: requires_scope_id()?.to_string(),
                        spaced: false,
                    },
                    node,
                    predicates,
                )
            }
            "prepend_spaced_scoped_softline" => {
                let id = self.next_id();
                self.prepend(
                    Atom::ScopedSoftline {
                        id,
                        scope_id: requires_scope_id()?.to_string(),
                        spaced: true,
                    },
                    node,
                    predicates,
                )
            }
            // Mark a leaf to be printed on an single line, with no indentation
            "single_line_no_indent" => {
                self.atoms = self
                    .atoms
                    .iter()
                    .map(|atom| match atom {
                        Atom::Leaf { content, id, .. } if *id == node.id() => Atom::Leaf {
                            content: content.to_string(),
                            id: *id,
                            single_line_no_indent: true,
                        },
                        _ => atom.clone(),
                    })
                    .collect();
                self.append(Atom::Hardline, node, predicates)
            }
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

    fn mark_leaf_parent(&mut self, node: &Node, parent_id: usize) {
        self.parent_leaf_nodes.insert(node.id(), parent_id);
        for child in node.children(&mut node.walk()) {
            self.mark_leaf_parent(&child, parent_id)
        }
    }

    fn collect_leafs_inner(
        &mut self,
        node: &Node,
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
                single_line_no_indent: false,
            });
            // Mark all sub-nodes as having this node as a "leaf parent"
            self.mark_leaf_parent(node, node.id())
        } else {
            for child in node.children(&mut node.walk()) {
                self.collect_leafs_inner(&child, source, &parent_ids, level + 1)?;
            }
        }

        Ok(())
    }

    fn prepend(&mut self, atom: Atom, node: &Node, predicates: &QueryPredicates) {
        if let Some(atom) = self.expand_multiline(atom, node) {
            let atom = self.wrap(atom, predicates);
            // TODO: Pre-populate these
            let target_node = first_leaf(node);

            // If this is a child of a node that we have deemed as a leaf node
            // (e.g. a character in a string), we need to use that node id
            // instead.
            let target_node_id = self.parent_leaf_node(&target_node);

            log::debug!("Prepending {atom:?} to node {:?}", target_node,);

            self.prepend
                .entry(target_node_id)
                .and_modify(|atoms| atoms.push(atom.clone()))
                .or_insert_with(|| vec![atom]);
        }
    }

    fn append(&mut self, atom: Atom, node: &Node, predicates: &QueryPredicates) {
        if let Some(atom) = self.expand_multiline(atom, node) {
            let atom = self.wrap(atom, predicates);
            let target_node = last_leaf(node);

            // If this is a child of a node that we have deemed as a leaf node
            // (e.g. a character in a string), we need to use that node id
            // instead.
            let target_node_id = self.parent_leaf_node(&target_node);

            log::debug!("Appending {atom:?} to node {:?}", target_node,);

            self.append
                .entry(target_node_id)
                .and_modify(|atoms| atoms.push(atom.clone()))
                .or_insert_with(|| vec![atom]);
        }
    }

    fn begin_scope_before(&mut self, node: &Node, scope_id: &str) {
        let target_node = first_leaf(node);

        // If this is a child of a node that we have deemed as a leaf node
        // (e.g. a character in a string), we need to use that node id
        // instead.
        let target_node_id = self.parent_leaf_node(&target_node);

        log::debug!("Begin scope {scope_id:?} before node {:?}", target_node,);

        self.scope_begin
            .entry(target_node_id)
            .and_modify(|(_, scope_ids)| scope_ids.push(String::from(scope_id)))
            .or_insert_with(|| {
                (
                    target_node.start_position().row(),
                    vec![String::from(scope_id)],
                )
            });
    }

    fn end_scope_after(&mut self, node: &Node, scope_id: &str) {
        let target_node = last_leaf(node);

        // If this is a child of a node that we have deemed as a leaf node
        // (e.g. a character in a string), we need to use that node id
        // instead.
        let target_node_id = self.parent_leaf_node(&target_node);

        log::debug!("End scope {scope_id:?} after node {:?}", target_node,);

        self.scope_end
            .entry(target_node_id)
            .and_modify(|(_, scope_ids)| scope_ids.push(String::from(scope_id)))
            .or_insert_with(|| {
                (
                    target_node.end_position().row(),
                    vec![String::from(scope_id)],
                )
            });
    }

    fn parent_leaf_node(&mut self, node: &Node) -> usize {
        if let Some(id) = self.parent_leaf_nodes.get(&node.id()) {
            *id
        } else {
            node.id()
        }
    }

    fn expand_multiline(&self, atom: Atom, node: &Node) -> Option<Atom> {
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

    /// This function expands ScopedSoftline atoms depending on whether the context
    // containing them is multiline.
    // It does two passes over the atom collection: the first one associates each ScopedSoftline
    // to its scope, and decides what to replace them with when the scope ends.
    // The second pass applies the modifications to the atoms.
    fn post_process_scopes(&mut self) {
        type ScopeId = String;
        type LineIndex = u32;
        type ScopedNodeId = usize;
        // `opened_scopes` maintains stacks of opened scopes,
        // the line at which they started,
        // and the list of ScopedSoftline they contain.
        let mut opened_scopes: HashMap<&ScopeId, Vec<(LineIndex, Vec<&Atom>)>> = HashMap::new();
        // We can't process ScopedSoftlines in-place as we encounter them in the list of
        // atoms: we need to know when their encompassing scope ends to decide what to
        // replace them with. Instead of in-place modifications, we associate a replacement
        // atom to each ScopedSoftline atom (identified by their `id` field), then apply
        // the modifications in a second pass over the atoms.
        let mut modifications: HashMap<ScopedNodeId, Option<Atom>> = HashMap::new();
        // `force_apply_modifications` keeps track of whether something has gone wrong in the
        // post-processing (e.g. closing an unopened scope, finding a scoped atom outside
        // of its scope). If we detect any error, we don't skip the "Apply modifications" part
        // of the processing, even if the `modifications` map is empty. This is to ensure we will
        // get rid of misplaced scoped atoms.
        let mut force_apply_modifications = false;

        for atom in &self.atoms {
            if let Atom::Leaf { id, .. } = atom {
                // Begin a new scope
                if let Some((line_start, scope_ids)) = self.scope_begin.get(id) {
                    for scope_id in scope_ids {
                        opened_scopes
                            .entry(scope_id)
                            .or_insert_with(Vec::new)
                            .push((*line_start, Vec::new()));
                    }
                }
                // End a scope, and register the ScopedSoftline transformations
                // in `modifications`
                if let Some((line_end, scope_ids)) = self.scope_end.get(id) {
                    for scope_id in scope_ids {
                        if let Some((line_start, atoms)) = opened_scopes
                            .get_mut(scope_id)
                            .map(Vec::pop)
                            .unwrap_or(None)
                        {
                            let multiline = line_start != *line_end;
                            for atom in atoms {
                                if let Atom::ScopedSoftline { id, spaced, .. } = atom {
                                    let new_atom = if multiline {
                                        Some(Atom::Hardline)
                                    } else if *spaced {
                                        Some(Atom::Space)
                                    } else {
                                        None
                                    };
                                    modifications.insert(*id, new_atom);
                                } else if let Atom::ScopedConditional {
                                    id,
                                    atom,
                                    condition,
                                    ..
                                } = atom
                                {
                                    let multiline_only =
                                        *condition == ScopeCondition::MultiLineOnly;
                                    let new_atom = if multiline == multiline_only {
                                        Some((**atom).clone())
                                    } else {
                                        None
                                    };
                                    modifications.insert(*id, new_atom);
                                }
                            }
                        } else {
                            log::warn!("Closing unopened scope {scope_id:?}");
                            force_apply_modifications = true;
                        }
                    }
                }
            // Register the ScopedSoftline in the correct scope
            } else if let Atom::ScopedSoftline { scope_id, .. } = atom {
                if let Some((_, vec)) = opened_scopes
                    .get_mut(&scope_id)
                    .map(|v| v.last_mut())
                    .unwrap_or(None)
                {
                    vec.push(atom)
                } else {
                    log::warn!("Found scoped softline {:?} outside of its scope", atom);
                    force_apply_modifications = true;
                }
            // Register the ScopedConditional in the correct scope
            } else if let Atom::ScopedConditional { scope_id, .. } = atom {
                if let Some((_, vec)) = opened_scopes
                    .get_mut(&scope_id)
                    .map(|v| v.last_mut())
                    .unwrap_or(None)
                {
                    vec.push(atom)
                } else {
                    log::warn!("Found scoped conditional {:?} outside of its scope", atom);
                    force_apply_modifications = true;
                }
            }
        }
        let still_opened: Vec<&String> = opened_scopes
            .into_iter()
            .filter_map(|(scope_id, vec)| if vec.is_empty() { None } else { Some(scope_id) })
            .collect();
        if !still_opened.is_empty() {
            log::warn!("Some scopes have been left opened: {:?}", still_opened);
            force_apply_modifications = true;
        }

        // Apply modifications.
        // For performance reasons, skip this step if there are no modifications to make
        if !modifications.is_empty() || force_apply_modifications {
            let new_atoms = self
                .atoms
                .iter()
                .filter_map(|atom| {
                    if let Atom::ScopedSoftline { id, .. } = atom {
                        if let Some(atom_option) = modifications.remove(id) {
                            atom_option
                        } else {
                            log::warn!(
                                "Found scoped softline {:?}, but was unable to replace it.",
                                atom
                            );
                            None
                        }
                    } else if let Atom::ScopedConditional { id, .. } = atom {
                        if let Some(atom_option) = modifications.remove(id) {
                            atom_option
                        } else {
                            log::warn!(
                                "Found scoped conditional {:?}, but was unable to replace it.",
                                atom
                            );
                            None
                        }
                    } else {
                        Some(atom.clone())
                    }
                })
                .collect();
            self.atoms = new_atoms
        }
    }

    // This function merges the spaces, new lines and blank lines.
    // If there are several tokens of different kind one after the other,
    // the blank line is kept over the new line which itself is kept over the space.
    // Furthermore, this function put the indentation delimiters before any space/line atom.
    pub fn post_process(&mut self) {
        self.post_process_scopes();
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
                    Atom::Space | Atom::Antispace | Atom::Hardline | Atom::Blankline => {}
                    _ => new_vec.push(next.clone()),
                };
            }
        }
        collapse_antispace(&mut new_vec);
        self.atoms = new_vec;
    }

    fn next_id(&mut self) -> usize {
        self.counter += 1;
        self.counter
    }
}

#[derive(Clone, Debug, Default)]
// A struct to store query predicates that are relevant to Topiary
pub struct QueryPredicates {
    pub delimiter: Option<String>,
    pub scope_id: Option<String>,
    pub single_line_only: bool,
    pub multi_line_only: bool,
    pub single_line_scope_only: Option<String>,
    pub multi_line_scope_only: Option<String>,
}

fn post_process_internal(new_vec: &mut Vec<Atom>, prev: Atom, next: Atom) {
    match prev {
        // Discard all spaces "connected" to an antispace
        Atom::Antispace => {
            match next {
                // Skip over a space or antispace that follows an antispace...
                Atom::Space | Atom::Antispace => {}

                // ...otherwise, pop the previous antispace (as we're done with
                // processing it) and any spaces that preceded it, and push
                // whatever follows
                _ => {
                    collapse_antispace(new_vec);
                    new_vec.push(next);
                }
            }
        }

        // If the last atom is a space/line
        Atom::Space | Atom::Hardline | Atom::Blankline => {
            match next {
                // And the next one is also a space/line
                Atom::Space | Atom::Hardline | Atom::Blankline => {
                    if is_dominant(&next, &prev) {
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

        // If the last one is a DeleteBegin,
        // we ignore all the atoms until a DeleteEnd is met.
        Atom::DeleteBegin => {
            if next == Atom::DeleteEnd {
                new_vec.pop();
            }
        }

        // Otherwise, we simply copy the atom to the new vector.
        _ => new_vec.push(next),
    }
}

fn collapse_antispace(v: &mut Vec<Atom>) {
    while let Some(last) = v.last() {
        match last {
            Atom::Space | Atom::Antispace => v.pop(),
            _ => break,
        };
    }
}

// This function is only expected to take spaces and newlines as argument.
// It defines the order Blankline > Hardline > Space.
fn is_dominant(next: &Atom, prev: &Atom) -> bool {
    match next {
        Atom::Space => false,
        Atom::Hardline => *prev == Atom::Space,
        Atom::Blankline => *prev != Atom::Blankline,
        _ => panic!("Unexpected character in is_dominant"),
    }
}

fn detect_multi_line_nodes(node: &Node) -> HashSet<usize> {
    let mut ids = HashSet::new();

    for child in node.children(&mut node.walk()) {
        ids.extend(detect_multi_line_nodes(&child));
    }

    let start_line = node.start_position().row();
    let end_line = node.end_position().row();

    if end_line > start_line {
        let id = node.id();
        ids.insert(id);
        log::debug!("Multi-line node {}: {:?}", id, node,);
    }

    ids
}

fn detect_blank_lines_before(node: &Node) -> HashSet<usize> {
    detect_line_breaks_inner(node, 2, None, None).0
}

fn detect_line_break_before_and_after(node: &Node) -> (HashSet<usize>, HashSet<usize>) {
    let result = detect_line_breaks_inner(node, 1, None, None);
    (result.0, result.1)
}

// TODO: This is taking a bit too much time, and would benefit from an
// optimization.
// TODO 2: The whole function is a mess now, and should be rewritten.
fn detect_line_breaks_inner(
    node: &Node,
    minimum_line_breaks: u32,

    // TODO: Replace these with just previous_node: Option<&Node>
    previous_node_id: Option<usize>,
    previous_end: Option<u32>,
) -> (HashSet<usize>, HashSet<usize>, Option<usize>, Option<u32>) {
    let mut nodes_with_breaks_before = HashSet::new();
    let mut nodes_with_breaks_after = HashSet::new();

    if let (Some(previous_node_id), Some(previous_end)) = (previous_node_id, previous_end) {
        let current_start = node.start_position().row();

        if current_start >= previous_end + minimum_line_breaks {
            nodes_with_breaks_before.insert(node.id());
            nodes_with_breaks_after.insert(previous_node_id);

            log::debug!(
                "There are at least {} blank lines between {:?} and {:?}",
                minimum_line_breaks,
                previous_node_id,
                node.id()
            );
        }
    }

    let mut previous_node_id = Some(node.id());
    let mut previous_end = Some(node.end_position().row());

    for child in node.children(&mut node.walk()) {
        let (before, after, node_id, end) =
            detect_line_breaks_inner(&child, minimum_line_breaks, previous_node_id, previous_end);

        previous_node_id = node_id;
        previous_end = end;
        nodes_with_breaks_before.extend(before);
        nodes_with_breaks_after.extend(after);
    }

    (
        nodes_with_breaks_before,
        nodes_with_breaks_after,
        previous_node_id,
        previous_end,
    )
}

// TODO: first_leaf and last_leaf can probably be simplified.

/// Given a node, returns the id of the first leaf in the subtree.
fn first_leaf<'tree, 'node: 'tree>(node: &'node Node<'tree>) -> Cow<'node, Node<'tree>> {
    first_leaf_inner(Cow::Borrowed(node))
}

fn first_leaf_inner<'tree, 'node: 'tree>(node: Cow<'node, Node<'tree>>) -> Cow<'node, Node<'tree>> {
    if node.child_count() == 0 {
        node
    } else {
        let node = Cow::Owned(node.child(0).unwrap());
        first_leaf_inner(node)
    }
}

/// Given a node, returns the id of the last leaf in the subtree.
fn last_leaf<'tree, 'node: 'tree>(node: &'node Node<'tree>) -> Cow<'node, Node<'tree>> {
    last_leaf_inner(Cow::Borrowed(node))
}

fn last_leaf_inner<'tree, 'node: 'tree>(node: Cow<'node, Node<'tree>>) -> Cow<'node, Node<'tree>> {
    let nr_children = node.child_count();
    if nr_children == 0 {
        node
    } else {
        let node = Cow::Owned(node.child(nr_children - 1).unwrap());
        last_leaf_inner(node)
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
