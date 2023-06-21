use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    mem,
    ops::Deref,
};

use tree_sitter_facade::Node;

use crate::{Atom, FormatterError, FormatterResult, ScopeCondition};

/// A struct that holds sets of node IDs that have line breaks before or after them.
///
/// This struct is used by the `detect_line_breaks` function to return the node IDs that
/// have at least `minimum_line_breaks` number of line breaks before or after them in the
/// flattened vector of nodes.
struct NodesWithLinebreaks {
    /// A set of node IDs that have line breaks before them.
    before: HashSet<usize>,
    /// A set of node IDs that have line breaks after them.
    after: HashSet<usize>,
}

/// Contains Topiary's internal representation parsed document.
#[derive(Debug)]
pub struct AtomCollection {
    /// A flat list of all Atoms. This is is updated by some formatting
    /// directives, but most require some more complexity.
    atoms: Vec<Atom>,
    /// Whenever a formatting directive instructs tree-sitter to prepend
    /// something to a node, a new Atom is added to this HashMap.
    /// The key of the hashmap is the identifier of the node.
    prepend: HashMap<usize, Vec<Atom>>,
    /// Whenever a formatting directive instructs tree-sitter to append
    /// something to a node, a new Atom is added to this HashMap.
    /// The key of the hashmap is the identifier of the node.
    append: HashMap<usize, Vec<Atom>>,
    /// A query file can define custom leaf nodes (nodes that Topiary should not
    /// touch during formatting). When such a node is encountered, its id is stored in
    /// this HashSet.
    specified_leaf_nodes: HashSet<usize>,
    /// If a node is a leaf, or if it is explicitly marked as such by the
    /// formatting directives, it is added to this HashMap as the key. The value
    /// of the Map contains all parent nodes.
    parent_leaf_nodes: HashMap<usize, usize>,
    /// Topiary has some formatting directives that only apply if a node spans
    /// multiple lines. During initial collection all such nodes are added to this
    /// HashSet for easy checking if a node spans multiple lines.
    multi_line_nodes: HashSet<usize>,
    /// During initial Atom collection, any node that has a blank lines above
    /// the node is added to this HashSet.
    blank_lines_before: HashSet<usize>,
    /// During initial Atom collection, any node that has a linebreak directly
    /// before it is added to this HashSet.
    line_break_before: HashSet<usize>,
    /// During initial Atom collection, any node that has a linebreak directly
    /// after it is added to this HashSet.
    line_break_after: HashSet<usize>,
    /// The semantics of the types of scope_begin and scope_end is
    /// HashMap<leaf_node_id, (line_number, Vec<scope_id>)>
    /// The line number is passed here because otherwise the information
    /// is lost at post-processing time.
    scope_begin: HashMap<usize, (u32, Vec<String>)>,
    scope_end: HashMap<usize, (u32, Vec<String>)>,
    /// Used to generate unique IDs
    counter: usize,
}

impl AtomCollection {
    /// Use this to create an initial `AtomCollection`
    pub fn collect_leafs(
        root: &Node,
        source: &[u8],
        specified_leaf_nodes: HashSet<usize>,
    ) -> FormatterResult<Self> {
        // Flatten the tree, from the root node, in a depth-first traversal
        let dfs_nodes = dfs_flatten(root);

        // Detect user specified line breaks
        let multi_line_nodes = detect_multi_line_nodes(&dfs_nodes);
        let blank_line_nodes = detect_line_breaks(&dfs_nodes, 2);
        let line_break_nodes = detect_line_breaks(&dfs_nodes, 1);

        let mut atoms = Self {
            atoms: Vec::new(),
            prepend: HashMap::new(),
            append: HashMap::new(),
            specified_leaf_nodes,
            parent_leaf_nodes: HashMap::new(),
            multi_line_nodes,
            blank_lines_before: blank_line_nodes.before,
            line_break_before: line_break_nodes.before,
            line_break_after: line_break_nodes.after,
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

    /// Resolves a capture name by modifying the AtomCollection based on the
    /// instructions provided by the capture name on the Node.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the capture, starting with `@`.
    /// * `node` - The node that matches the capture in the syntax tree.
    /// * `predicates` - The query predicates that modify the formatting behavior for the capture.
    ///
    /// # Errors
    ///
    /// This function returns an error if the capture name requires a predicate that is not present.
    /// It also returns an error if the capture name is not recognized by Topiary.
    // NOTE: During processing Topiary applies the function below on every match
    // of every query. As such, this function may get called multiple times
    // per node. This means that any performance loss in this function has a
    // order of magnitute higher impact than any other of such losses. We must
    // therefore ensure that the function below is as performant as we can make
    // it (within reason).
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
                is_multi_line = true;
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
                self.append(Atom::Softline { spaced: false }, node, predicates);
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
                self.append(Atom::Softline { spaced: true }, node, predicates);
            }
            "prepend_delimiter" => self.prepend(
                Atom::Literal(requires_delimiter()?.to_string()),
                node,
                predicates,
            ),
            "prepend_empty_softline" => {
                self.prepend(Atom::Softline { spaced: false }, node, predicates);
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
                self.prepend(Atom::Softline { spaced: true }, node, predicates);
            }
            // Skip over leafs
            "leaf" => {}
            // Deletion
            "delete" => {
                self.prepend(Atom::DeleteBegin, node, predicates);
                self.append(Atom::DeleteEnd, node, predicates);
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
                );
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
                );
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
                );
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
                );
            }
            // Mark a leaf to be printed on an single line, with no indentation
            "single_line_no_indent" => {
                for a in &mut self.atoms {
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

                self.append(Atom::Hardline, node, predicates);
            }
            // Mark a leaf to have all its lines be indented
            "multi_line_indent_all" => {
                for a in &mut self.atoms {
                    if let Atom::Leaf {
                        id,
                        multi_line_indent_all,
                        ..
                    } = a
                    {
                        if *id == node.id() {
                            *multi_line_indent_all = true;
                        }
                    }
                }
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

        for atom in &mut self.atoms {
            if let Atom::Leaf { id, .. } = atom {
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
            } else {
                log::debug!("Not a leaf: {atom:?}");
                expanded.push(mem::take(atom));
            }
        }

        self.atoms = expanded;
    }

    /// Marks the provided node as the parent of all its child nodes by adding
    /// it to the `parent_leaf_nodes` HashMap.
    fn mark_leaf_parent(&mut self, node: &Node, parent_id: usize) {
        self.parent_leaf_nodes.insert(node.id(), parent_id);
        for child in node.children(&mut node.walk()) {
            self.mark_leaf_parent(&child, parent_id);
        }
    }

    /// This function collects the leaf nodes of a tree-sitter CST tree and stores them in the `atoms` field of the internal formatter state.
    /// It also marks the leaf parent of each node in the `leaf_parent` field of the formatter.
    /// A leaf node is either a node with no children or a node that is specified as a leaf node by the formatter.
    /// A leaf parent is the closest ancestor of a leaf node.
    ///
    /// # Arguments
    ///
    /// * `node` - The current node to process.
    /// * `source` - The full source code as a byte slice.
    /// * `parent_ids` - A vector of node ids that are the ancestors of the current node.
    /// * `level` - The depth of the current node in the CST tree.
    ///
    /// # Errors
    ///
    /// This function returns an error if it fails to convert the source code belonging to the node to UTF-8.
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
            log::debug!("Skipping zero-byte node: {node:?}");
        } else if node.child_count() == 0
            || self.specified_leaf_nodes.contains(&node.id())
            // We treat error nodes as leafs when `tolerate_parsing_errors` is set to true.
            // This ensures Topiary does not try to further apply transformations on them.
            // If `tolerate_parsing_errors` is set to false, this part of the code is only reached if the tree contains no ERROR nodes,
            // and as such the check below would be redundant.
            || node.kind() == "ERROR"
        {
            self.atoms.push(Atom::Leaf {
                content: String::from(node.utf8_text(source)?),
                id,
                original_position: node.start_position().into(),
                single_line_no_indent: false,
                multi_line_indent_all: false,
            });
            // Mark all sub-nodes as having this node as a "leaf parent"
            self.mark_leaf_parent(node, node.id());
        } else {
            for child in node.children(&mut node.walk()) {
                self.collect_leafs_inner(&child, source, &parent_ids, level + 1)?;
            }
        }

        Ok(())
    }

    /// Prepend an atom to the first leaf node in the subtree of a given node.
    ///
    /// # Arguments
    ///
    /// * `atom` - The atom to prepend.
    /// * `node` - The node to which the atom is prepended.
    /// * `predicates` - The query predicates to wrap the atom with.
    fn prepend(&mut self, atom: Atom, node: &Node, predicates: &QueryPredicates) {
        let atom = self.expand_multiline(atom, node);
        let atom = self.wrap(atom, predicates);
        // TODO: Pre-populate these
        let target_node = self.first_leaf(node);

        log::debug!("Prepending {atom:?} to node {:?}", target_node,);

        self.prepend.entry(target_node.id()).or_default().push(atom);
    }

    /// Append an atom to the last leaf node in the subtree of a given node.
    ///
    /// # Arguments
    ///
    /// * `atom` - The atom to append.
    /// * `node` - The node to which the atom is appended.
    /// * `predicates` - The query predicates to wrap the atom with.
    fn append(&mut self, atom: Atom, node: &Node, predicates: &QueryPredicates) {
        let atom = self.expand_multiline(atom, node);
        let atom = self.wrap(atom, predicates);
        let target_node = self.last_leaf(node);

        log::debug!("Appending {atom:?} to node {:?}", target_node,);

        self.append.entry(target_node.id()).or_default().push(atom);
    }

    /// Begins a scope with the given `scope_id` before the first leaf node of the given `Node`'s subtree.
    ///
    /// # Arguments
    ///
    /// * `node` - A reference to a `Node` object that represents a subtree in the syntax tree.
    /// * `scope_id` - A reference to a string that identifies the scope to begin.
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

    /// Ends a scope with the given `scope_id` after the last leaf node of the given `Node`'s subtree.
    ///
    /// # Arguments
    ///
    /// * `node` - A reference to a `Node` object that represents a subtree in the syntax tree.
    /// * `scope_id` - A reference to a string that identifies the scope to end.
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

    /// Expands a softline atom to a hardline, space or empty atom depending on
    /// if we are in a multiline context or not.
    ///
    /// If the node's parent is labelled as a multi-line node, it expands the
    /// softline to a hardline.
    /// If the node's parent is not labelled as such, but the Atom is labelled
    /// as `spaced`, the Atom is turned into a space. The Atom is discarded if it is not
    /// labelled as `spaced`.
    /// If the node has no parent, the softline atom is discarded by returning an empty atom.
    ///
    /// The function ignores all atoms that are not softlines.
    ///
    /// # Arguments
    ///
    /// * `atom` - An atom to be expanded.
    /// * `node` - The node to which the atom applies.
    ///
    /// # Returns
    ///
    /// A new atom after expanding the softline if applicable.
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

    /// This function expands `ScopedSoftline` atoms depending on whether the context
    /// containing them is multiline.
    /// It does two passes over the atom collection: the first one associates each `ScopedSoftline`
    /// to its scope, and decides what to replace them with when the scope ends.
    /// The second pass applies the modifications to the atoms.
    fn post_process_scopes(&mut self) {
        type ScopeId = String;
        type LineIndex = u32;
        type ScopedNodeId = usize;
        // `opened_scopes` maintains stacks of opened scopes,
        // the line at which they started,
        // and the list of `ScopedSoftline` they contain.
        let mut opened_scopes: HashMap<&ScopeId, Vec<(LineIndex, Vec<&Atom>)>> = HashMap::new();
        // We can't process `ScopedSoftline` in-place as we encounter them in the list of
        // atoms: we need to know when their encompassing scope ends to decide what to
        // replace them with. Instead of in-place modifications, we associate a replacement
        // atom to each `ScopedSoftline` atom (identified by their `id` field), then apply
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
                        if let Some((line_start, atoms)) =
                            opened_scopes.get_mut(scope_id).and_then(Vec::pop)
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
                if let Some((_, vec)) = opened_scopes.get_mut(&scope_id).and_then(|v| v.last_mut())
                {
                    vec.push(atom);
                } else {
                    log::warn!("Found scoped softline {:?} outside of its scope", atom);
                    force_apply_modifications = true;
                }
            // Register the ScopedConditional in the correct scope
            } else if let Atom::ScopedConditional { scope_id, .. } = atom {
                if let Some((_, vec)) = opened_scopes.get_mut(&scope_id).and_then(|v| v.last_mut())
                {
                    vec.push(atom);
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
                        *atom = Atom::Empty;
                    }
                } else if let Atom::ScopedConditional { id, .. } = atom {
                    if let Some(replacement) = modifications.remove(id) {
                        *atom = replacement;
                    } else {
                        log::warn!(
                            "Found scoped conditional {:?}, but was unable to replace it.",
                            atom
                        );
                        *atom = Atom::Empty;
                    }
                }
            }
        }
    }

    /// Separate post_processing of Delete sections, to avoid interference with whitespace logic
    fn post_process_deletes(&mut self) {
        let mut delete_level = 0;
        for atom in &mut self.atoms {
            match atom {
                Atom::DeleteBegin => {
                    delete_level += 1;
                    *atom = Atom::Empty;
                }
                Atom::DeleteEnd => {
                    delete_level -= 1;
                    *atom = Atom::Empty;
                }
                _ => {
                    if delete_level > 0 {
                        *atom = Atom::Empty;
                    }
                }
            }
        }
        if delete_level != 0 {
            log::warn!("The number of DeleteBegin is different from the number of DeleteEnd.");
        }
    }

    /// This function merges the spaces, new lines and blank lines.
    /// If there are several tokens of different kind one after the other,
    /// the blank line is kept over the new line which itself is kept over the space.
    /// Furthermore, this function put the indentation delimiters before any space/line atom.
    pub fn post_process(&mut self) {
        self.post_process_scopes();
        self.post_process_deletes();
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

    /// Returns the first leaf node of a given node's subtree.
    ///
    /// This function recursively traverses the tree from the given node
    /// and returns the first leaf.
    /// A leaf node is a node that either has no children, or is specified in
    /// the `specified_leaf_nodes` HashSet.
    ///
    /// # Arguments
    ///
    /// * `node` - A reference to a node in the tree.
    ///
    /// # Returns
    ///
    /// A `Cow` enum that wraps a borrowed node.
    fn first_leaf<'tree, 'node: 'tree>(&self, node: &'node Node<'tree>) -> Cow<'node, Node<'tree>> {
        self.first_leaf_inner(Cow::Borrowed(node))
    }

    /// Helper function to the `first_leaf` function.
    /// Recursively calls itself on the first child of the given node
    /// until it reaches a leaf node.
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

    /// Returns the last leaf node of a given node's subtree.
    ///
    /// This function recursively traverses the tree from the given node
    /// and returns the last leaf.
    /// A leaf node is a node that either has no children, or is specified in
    /// the `specified_leaf_nodes` HashSet.
    ///
    /// # Arguments
    ///
    /// * `node` - A reference to a node in the tree.
    ///
    /// # Returns
    ///
    /// A `Cow` enum that wraps a borrowed node.
    fn last_leaf<'tree, 'node: 'tree>(&self, node: &'node Node<'tree>) -> Cow<'node, Node<'tree>> {
        self.last_leaf_inner(Cow::Borrowed(node))
    }

    /// Helper function to the `last_leaf` function.
    /// Recursively calls itself on the last child of the given node
    /// until it reaches a leaf node.
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
/// A struct that represents a set of predicates for a query that are relevant for Topiary.
pub struct QueryPredicates {
    /// The predicate used to associate a user-defined delimiter to the
    /// `@append_delimiter` and `@prepend_delimiter` directives.
    pub delimiter: Option<String>,
    /// The predicate used to name a scope indicated by `@begin_scope` and `@end_scope`.
    pub scope_id: Option<String>,
    /// The flag that indicates whether the query only matches single-line nodes.
    pub single_line_only: bool,
    /// The flag that indicates whether the query only matches multi-line nodes.
    pub multi_line_only: bool,
    /// The flag that indicates that the query only triggers if the associated
    /// custom scope containing the matched nodes is single-line.
    pub single_line_scope_only: Option<String>,
    /// The flag that indicates that the query only triggers if the associated
    /// custom scope containing the matched nodes is multi-line.
    pub multi_line_scope_only: Option<String>,
}

/// Collapses spaces before antispace atoms in a vector of atoms.
///
/// This function modifies the given vector of atoms in place, replacing any
/// space atoms that precede an antispace atom with empty atoms.
///
/// # Arguments
///
/// * `v` - A mutable reference to a vector of atoms.
///
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

/// Flatten the tree, depth-first, into a vector of nodes.
///
/// This function takes a reference to a node and returns a vector of references
/// to all the nodes in the tree rooted at that node, in depth-first order.
///
/// # Arguments
///
/// * `node` - A reference to a node in the tree.
///
/// # Returns
///
/// A vector of references to nodes in the provides node's subtree.
///
/// # Notes
///
/// This function uses an iterative approach instead of a recursive one for performance reasons.
/// See https://github.com/tweag/topiary/pull/417#issuecomment-1499085230 for more details.
fn dfs_flatten<'tree>(node: &Node<'tree>) -> Vec<Node<'tree>> {
    // Flatten the tree, depth-first, into a vector of nodes
    let mut walker = node.walk();
    let mut dfs_nodes = Vec::new();

    'walk: loop {
        dfs_nodes.push(walker.node());

        if !walker.goto_first_child() {
            while !walker.goto_next_sibling() {
                if !walker.goto_parent() {
                    break 'walk;
                }
            }
        }
    }

    dfs_nodes
}

/// Detects multi-line nodes in a vector of nodes and returns a set of their ids.
///
/// This function takes a slice of `Node`s that represents the nodes in a depth-first search
/// order of a syntax tree and iterates over them. For each node, it compares the start and end
/// line numbers and checks if they are different. If they are, it means the node spans multiple
/// lines and its id is added to the returned set.
///
/// # Arguments
///
/// * `dfs_nodes` - A slice of nodes in depth-first search order.
///
/// # Returns
///
/// A `HashSet` containing the ids of the multi-line nodes.
fn detect_multi_line_nodes(dfs_nodes: &[Node]) -> HashSet<usize> {
    dfs_nodes
        .iter()
        .filter_map(|node| {
            let start_line = node.start_position().row();
            let end_line = node.end_position().row();

            if end_line > start_line {
                log::debug!("Multi-line node {}: {:?}", node.id(), node,);
                return Some(node.id());
            }

            None
        })
        .collect()
}

/// Detects line breaks between nodes in a flattened vector of nodes.
///
/// This function takes a slice of nodes that have been flattened by a depth-first search (DFS)
/// and returns a struct that contains two sets of node IDs: one for the nodes that have a line break
/// before them, and one for the nodes that have a line break after them.
///
/// # Arguments
///
/// * `dfs_nodes` - A slice of nodes that have been flattened by a DFS.
/// * `minimum_line_breaks` - The minimum number of line breaks that must exist between two adjacent nodes
///   in order for them to be considered as having a line break before or after them.
///
/// # Returns
///
/// A `NodesWithLinebreaks` struct that contains two sets of node IDs: one for the nodes that have a line break
/// before them, and one for the nodes that have a line break after them.
fn detect_line_breaks(dfs_nodes: &[Node], minimum_line_breaks: u32) -> NodesWithLinebreaks {
    // Zip the flattened vector with its own tail => Iterator of pairs of adjacent nodes
    // Filter this by the threshold distance between pair components
    // Unzip into "nodes with spaces before" and "after" sets, respectively
    let (before, after) = dfs_nodes
        .iter()
        .zip(dfs_nodes[1..].iter())
        .filter_map(|(left, right)| {
            let last = left.end_position().row();
            let next = right.start_position().row();

            if next >= last + minimum_line_breaks {
                log::debug!(
                    "There are at least {} line breaks between {:?} and {:?}",
                    minimum_line_breaks,
                    left.id(),
                    right.id()
                );

                // Our node pairs look like this, hence the flip:
                //
                //          /------ before -----\
                //   <Left> <Line Breaks> <Right>
                //   \----- after ------/
                //
                // NOTE We only need the node IDs
                return Some((right.id(), left.id()));
            }

            None
        })
        .unzip();

    NodesWithLinebreaks { before, after }
}

/// So that we can easily extract the atoms using `&atom_collection[..]`
impl<Idx> std::ops::Index<Idx> for AtomCollection
where
    Idx: std::slice::SliceIndex<[Atom]>,
{
    type Output = Idx::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.atoms[index]
    }
}
