use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    mem,
    ops::Deref,
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
            log::debug!("Skipping because context is multi-line and #single_line_only! is set");
            return Ok(());
        }
        if !is_multi_line && predicates.multi_line_only {
            log::debug!("Skipping because context is single-line and #multi_line_only! is set");
            return Ok(());
        }
        if let Some(parent_id) = self.parent_leaf_nodes.get(&node.id()) {
            if *parent_id != node.id() {
                log::warn!("Skipping because the match occurred below a leaf node: {node:?}");
                return Ok(());
            }
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
                for a in self.atoms.iter_mut() {
                    if let Atom::Leaf {
                        id,
                        single_line_no_indent,
                        ..
                    } = a
                    {
                        if *id == node.id() {
                            *single_line_no_indent = true;
                        }
                    }
                }

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

        for atom in self.atoms.iter_mut() {
            match atom {
                Atom::Leaf { id, .. } => {
                    let prepends = self.prepend.entry(*id).or_default();
                    let appends = self.append.entry(*id).or_default();

                    // Rather than cloning the atom from the old vector, we
                    // simply take it. This will leave a default (empty) atom
                    // in its place.
                    let swapped_atom = mem::take(atom);

                    log::debug!("Applying prepend of {prepends:?} to {atom:?}.");
                    expanded.append(prepends);
                    expanded.push(swapped_atom);

                    log::debug!("Applying append of {appends:?} to {atom:?}.");
                    expanded.append(appends);
                }
                _ => {
                    log::debug!("Not a leaf: {atom:?}");
                    expanded.push(mem::take(atom));
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

        if node.end_byte() == node.start_byte() {
            log::info!("Skipping zero-byte node: {node:?}");
        } else if node.child_count() == 0 || self.specified_leaf_nodes.contains(&node.id()) {
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
        let atom = self.expand_multiline(atom, node);
        let atom = self.wrap(atom, predicates);
        // TODO: Pre-populate these
        let target_node = self.first_leaf(node);

        log::debug!("Prepending {atom:?} to node {:?}", target_node,);

        self.prepend
            .entry(target_node.id())
            .or_insert(vec![])
            .push(atom);
    }

    fn append(&mut self, atom: Atom, node: &Node, predicates: &QueryPredicates) {
        let atom = self.expand_multiline(atom, node);
        let atom = self.wrap(atom, predicates);
        let target_node = self.last_leaf(node);

        log::debug!("Appending {atom:?} to node {:?}", target_node,);

        self.append
            .entry(target_node.id())
            .or_insert(vec![])
            .push(atom);
    }

    fn begin_scope_before(&mut self, node: &Node, scope_id: &str) {
        let target_node = self.first_leaf(node);

        log::debug!("Begin scope {scope_id:?} before node {:?}", target_node,);

        self.scope_begin
            .entry(target_node.id())
            .and_modify(|(_, scope_ids)| scope_ids.push(String::from(scope_id)))
            .or_insert_with(|| {
                (
                    target_node.start_position().row(),
                    vec![String::from(scope_id)],
                )
            });
    }

    fn end_scope_after(&mut self, node: &Node, scope_id: &str) {
        let target_node = self.last_leaf(node);

        log::debug!("End scope {scope_id:?} after node {:?}", target_node,);

        self.scope_end
            .entry(target_node.id())
            .and_modify(|(_, scope_ids)| scope_ids.push(String::from(scope_id)))
            .or_insert_with(|| {
                (
                    target_node.end_position().row(),
                    vec![String::from(scope_id)],
                )
            });
    }

    fn expand_multiline(&self, atom: Atom, node: &Node) -> Atom {
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
                    Atom::Hardline
                } else if spaced {
                    log::debug!(
                        "Expanding softline to space in node {:?} with parent {}: {:?}",
                        node,
                        parent_id,
                        parent
                    );
                    Atom::Space
                } else {
                    Atom::Empty
                }
            } else {
                Atom::Empty
            }
        } else {
            atom
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
        let mut modifications: HashMap<ScopedNodeId, Atom> = HashMap::new();
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
                                        Atom::Hardline
                                    } else if *spaced {
                                        Atom::Space
                                    } else {
                                        Atom::Empty
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
                                        atom.deref().clone()
                                    } else {
                                        Atom::Empty
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
            for atom in &mut self.atoms {
                if let Atom::ScopedSoftline { id, .. } = atom {
                    if let Some(replacement) = modifications.remove(id) {
                        *atom = replacement;
                    } else {
                        log::warn!(
                            "Found scoped softline {:?}, but was unable to replace it.",
                            atom
                        );
                        *atom = Atom::Empty
                    }
                } else if let Atom::ScopedConditional { id, .. } = atom {
                    if let Some(replacement) = modifications.remove(id) {
                        *atom = replacement;
                    } else {
                        log::warn!(
                            "Found scoped conditional {:?}, but was unable to replace it.",
                            atom
                        );
                        *atom = Atom::Empty
                    }
                }
            }
        }
    }

    // This function merges the spaces, new lines and blank lines.
    // If there are several tokens of different kind one after the other,
    // the blank line is kept over the new line which itself is kept over the space.
    // Furthermore, this function put the indentation delimiters before any space/line atom.
    pub fn post_process(&mut self) {
        self.post_process_scopes();
        let mut prev: Option<&mut Atom> = None;
        for next in &mut self.atoms {
            if let Some(prev) = prev.as_mut() {
                match prev {
                    // Discard all spaces following an antispace. We'll fix the
                    // preceding ones in the next pass.
                    Atom::Antispace => {
                        match next {
                            // Remove any space or antispace that follows an
                            // antispace by setting it empty.
                            Atom::Space | Atom::Antispace => {
                                *next = Atom::Empty;
                            }
                            _ => {}
                        }
                    }

                    // If the last atom is a space/line
                    Atom::Empty | Atom::Space | Atom::Hardline | Atom::Blankline => {
                        match next {
                            // And the next one is also a space/line
                            Atom::Empty | Atom::Space | Atom::Hardline | Atom::Blankline => {
                                // Set the non-dominant one to empty.
                                if is_dominant(next, prev) {
                                    **prev = Atom::Empty;
                                } else {
                                    *next = Atom::Empty;
                                }
                            }

                            // Or an indentation delimiter, then one has to merge/re-order.
                            Atom::IndentStart | Atom::IndentEnd => {
                                let old_prev = prev.clone();
                                **prev = next.clone();
                                *next = old_prev;
                            }

                            _ => {}
                        }
                    }

                    // If the last one is a DeleteBegin,
                    // we ignore all the atoms until a DeleteEnd is met.
                    Atom::DeleteBegin => {
                        if *next == Atom::DeleteEnd {
                            // Break this pattern. We no longer want prev to
                            // match DeleteBegin.
                            **prev = Atom::Empty;
                        }

                        // We're inside a delete section, so set atom to empty.
                        *next = Atom::Empty;
                    }

                    _ => {}
                }
            } else {
                // If we're at the beginning of the file and still haven't
                // reached a non-empty atom, we remove all the spaces and
                // newlines by setting them empty.
                match next {
                    Atom::Empty
                    | Atom::Space
                    | Atom::Antispace
                    | Atom::Hardline
                    | Atom::Blankline => {
                        *next = Atom::Empty;
                    }
                    _ => {}
                };
            }

            if *next != Atom::Empty {
                // Let prev point to the previous non-empty atom.
                prev = Some(next);
            }
        }

        // We have taken care of spaces following an antispace. Now fix the
        // preceding spaces.
        collapse_spaces_before_antispace(&mut self.atoms);

        log::debug!("List of atoms after post-processing: {:?}", self.atoms);
    }

    fn next_id(&mut self) -> usize {
        self.counter += 1;
        self.counter
    }

    // TODO: first_leaf and last_leaf can probably be simplified.

    /// Given a node, returns the id of the first leaf in the subtree.
    fn first_leaf<'tree, 'node: 'tree>(&self, node: &'node Node<'tree>) -> Cow<'node, Node<'tree>> {
        self.first_leaf_inner(Cow::Borrowed(node))
    }

    fn first_leaf_inner<'tree, 'node: 'tree>(
        &self,
        node: Cow<'node, Node<'tree>>,
    ) -> Cow<'node, Node<'tree>> {
        if node.child_count() == 0 || self.specified_leaf_nodes.contains(&node.id()) {
            node
        } else {
            let node = Cow::Owned(node.child(0).unwrap());
            self.first_leaf_inner(node)
        }
    }

    /// Given a node, returns the id of the last leaf in the subtree.
    fn last_leaf<'tree, 'node: 'tree>(&self, node: &'node Node<'tree>) -> Cow<'node, Node<'tree>> {
        self.last_leaf_inner(Cow::Borrowed(node))
    }

    fn last_leaf_inner<'tree, 'node: 'tree>(
        &self,
        node: Cow<'node, Node<'tree>>,
    ) -> Cow<'node, Node<'tree>> {
        let nr_children = node.child_count();
        if nr_children == 0 || self.specified_leaf_nodes.contains(&node.id()) {
            node
        } else {
            let node = Cow::Owned(node.child(nr_children - 1).unwrap());
            self.last_leaf_inner(node)
        }
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

fn collapse_spaces_before_antispace(v: &mut [Atom]) {
    let mut antispace_mode = false;

    for a in v.iter_mut().rev() {
        if *a == Atom::Antispace {
            *a = Atom::Empty;
            antispace_mode = true;
        } else if *a == Atom::Space && antispace_mode {
            *a = Atom::Empty;
        } else {
            antispace_mode = false;
        }
    }
}

// This function is only expected to take spaces and newlines as argument.
// It defines the order Blankline > Hardline > Space > Empty.
fn is_dominant(next: &Atom, prev: &Atom) -> bool {
    match next {
        Atom::Empty => false,
        Atom::Space => *prev == Atom::Empty,
        Atom::Hardline => *prev == Atom::Space || *prev == Atom::Empty,
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
