;; -----------------------------------------------------------------------------
;; Comments
;; -----------------------------------------------------------------------------
(line_comment) @prepend_input_softline @append_hardline @allow_blank_line_before

;; -----------------------------------------------------------------------------
;; Modules
;; -----------------------------------------------------------------------------
(module
  "module" @append_space
  name: (identifier) @append_space
  "version"? @append_space
  version_info: (quoted_string)? @append_space
  version_uri: (iri)? @append_space
) @allow_blank_line_before

(module_body) @prepend_space

;; -----------------------------------------------------------------------------
;; Indentation
;; -----------------------------------------------------------------------------
["is" "of"] @append_indent_start @append_hardline
["end"] @allow_blank_line_before @prepend_indent_end

;; -----------------------------------------------------------------------------
;; Imports
;; -----------------------------------------------------------------------------
(import_statement
  "import" @append_space
) @allow_blank_line_before @append_hardline

(import_statement
  "[" @append_indent_start @append_space
  "]" @prepend_space @prepend_indent_end
)

[(member_import) (module_import)] @append_space

(member_import "as" @append_space)
(module_import "as" @append_space)

;; -----------------------------------------------------------------------------
;; Annotations
;; -----------------------------------------------------------------------------
(annotation) @allow_blank_line_before @append_hardline

;; -----------------------------------------------------------------------------
;; Annotations >> Properties
;; -----------------------------------------------------------------------------
(annotation_property "=" @prepend_space @append_space) @allow_blank_line_before

;; -----------------------------------------------------------------------------
;; Annotations >> Constraints
;; -----------------------------------------------------------------------------
(constraint "assert" @append_space name: (identifier) @append_space)

;; -----------------------------------------------------------------------------
;; Annotations >> Constraints >> Informal
;; -----------------------------------------------------------------------------
(informal_constraint "=" @prepend_space @append_space)

;; -----------------------------------------------------------------------------
;; Annotations >> Constraints >> Formal
;; -----------------------------------------------------------------------------
(formal_constraint "end" @prepend_hardline)

(equation "=" @prepend_space @append_space)

[
  (less_than)
  (greater_than)
  (less_than_or_equal)
  (greater_than_or_equal)
] @prepend_space @append_space

[
  (conjunction)
  (disjunction)
  (exclusive_disjunction)
  (implication)
  (biconditional)
] @prepend_space @append_space

(negation) @append_space

(quantified_sentence "," @prepend_antispace @append_space)

[(universal) (existential)] @append_space

(quantified_variable ["in" "∈"] @prepend_space @append_space)

(actual_arguments
  [
    ((term) . ")" @do_nothing)
    ((term) . (term) @prepend_space)
  ]
)

(environment_def
  "def" @append_space
  name: (identifier) @append_space
) @allow_blank_line_before @append_hardline

(constraint_environment_end
) @prepend_indent_end @append_indent_start @append_hardline

(function_body [":=" "≔"] @prepend_space @append_space)

(constant_def [":=" "≔"] @prepend_space @append_space)

(function_parameter
  name: (identifier) @append_space
  ["->" "→"] @append_space
)

(function_cardinality_expression
  ordering: (_)? @append_space
  uniqueness: (_)? @append_space
) @append_space

(sequence_builder "|" @prepend_space @append_space)

(mapping_variable ["->" "→"] @prepend_space @append_space)

;; -----------------------------------------------------------------------------
;; Definitions
;; -----------------------------------------------------------------------------
(definition) @allow_blank_line_before

;; -----------------------------------------------------------------------------
;; Definitions >> Class
;; -----------------------------------------------------------------------------
(type_class_def
  "class" @append_space
  name: (identifier) @append_space
) @allow_blank_line_before

(method_def
  "def" @append_space
  name: (identifier) @append_space
  signature: (function_signature) @append_space
) @append_hardline @allow_blank_line_before

(function_signature
  [
    ((function_parameter) . ")" @do_nothing)
    ((function_parameter) . (function_parameter) @prepend_space)
  ]
)

(function_parameter ["->" "→"] @prepend_space @append_space)

(function_signature
  ["->" "→"] @prepend_space @append_space
  (function_cardinality_expression)? @append_space
  (function_type_reference) @append_space
)

(function_body
  [":=" "≔"] @append_hardline
) @prepend_indent_start @append_indent_end @append_space

;; -----------------------------------------------------------------------------
;; Definitions >> Datatype
;; -----------------------------------------------------------------------------
(data_type_def
  "datatype" @append_space
  name: (identifier) @append_space
  ["<-" "←"] @append_space
  opaque: (opaque)? @append_space
  base: (_) @append_space
) @allow_blank_line_before

;;(data_type_boolean_base
;; (boolean_connective) @append_space
;; "[" @append_space
;; first: (_) @append_space
;; rest: (_) @append_space
;; "]" @append_space)

;;(data_type_restriction
;; "{" @append_indent_start @append_hardline
;; "}" @prepend_indent_end)

;;(length_restriction_facet "=" @prepend_space @append_space)
;;(digit_restriction_facet "=" @prepend_space @append_space)
;;(value_restriction_facet "=" @prepend_space @append_space)
;;(tz_restriction_facet "=" @prepend_space @append_space)
;;(pattern_restriction_facet "=" @prepend_space @append_space)
;;(is_fixed) @append_space

;; -----------------------------------------------------------------------------
;; Definitions >> Dimension
;; -----------------------------------------------------------------------------
(dimension_def
  "dimension" @append_space
  name: (identifier) @append_space
) @allow_blank_line_before

;; -----------------------------------------------------------------------------
;; Definitions >> Entity
;; -----------------------------------------------------------------------------
(entity_def
  "entity" @append_space
  name: (identifier) @append_space
) @allow_blank_line_before

;; -----------------------------------------------------------------------------
;; Definitions >> Enum
;; -----------------------------------------------------------------------------
(enum_def
  "enum" @append_space
  name: (identifier) @append_space
) @allow_blank_line_before

;; -----------------------------------------------------------------------------
;; Definitions >> Event
;; -----------------------------------------------------------------------------
(event_def
  "event" @append_space
  name: (identifier) @append_space
) @allow_blank_line_before

;; -----------------------------------------------------------------------------
;; Definitions >> Property [[ nothing required, uses member_def rules ]]
;; -----------------------------------------------------------------------------

;; -----------------------------------------------------------------------------
;; Definitions >> RDF
;; -----------------------------------------------------------------------------
(rdf_def
  "rdf" @append_space
  name: (identifier) @append_space
) @allow_blank_line_before

(rdf_types
  "[" @append_indent_start @append_space
  type: (identifier_reference) @append_space
  "]" @prepend_indent_end
)

;; -----------------------------------------------------------------------------
;; Definitions >> Structure
;; -----------------------------------------------------------------------------
(structure_def
  "structure" @append_space
  name: (identifier) @append_space
) @allow_blank_line_before

;; -----------------------------------------------------------------------------
;; Definitions >> Union
;; -----------------------------------------------------------------------------
(union_def
  "union" @append_space
  name: (identifier) @append_space
) @allow_blank_line_before

;; -----------------------------------------------------------------------------
;; Members
;; -----------------------------------------------------------------------------
(entity_identity "identity" @append_space)

(member_def
  name: (identifier) @append_space
  ["->" "→"] @append_space
  target: (type_reference) @append_space
) @allow_blank_line_before

(member_def
  [
    (type_reference)
    (annotation_only_body)
  ] @append_hardline
  .
)

(cardinality_expression
  ordering: (_)? @append_space
  uniqueness: (_)? @append_space
) @append_space

(property_ref
  "ref" @append_space
  property: (identifier_reference) @append_space
) @allow_blank_line_before

;; -----------------------------------------------------------------------------
;; Variants
;; -----------------------------------------------------------------------------
(value_variant body: (annotation_only_body)? @prepend_space)

(value_variant
  [
    (identifier)
    (annotation_only_body)
  ] @append_hardline
  .
)

(type_variant
  "as"? @prepend_space
  rename: (identifier)? @prepend_space
  body: (annotation_only_body)? @prepend_space
)

(type_variant
  [
    (identifier_reference)
    (identifier)
    (annotation_only_body)
  ] @append_hardline
  .
)

;; -----------------------------------------------------------------------------
;; Values
;; -----------------------------------------------------------------------------
[
  (boolean)
  (integer)
  (unsigned)
  ;;(rational)
  (decimal)
  (double)
  (string)
  (iri)
] @leaf

(binary "#[" @append_space (hex_byte)* @append_space)

(sequence_of_values
  (
    (sequence_ordering)? @append_space
    .
    (sequence_uniqueness)
    "}"? @append_space
  )
  (
    "[" @append_space
    element: (_)* @append_space
  )
)

(sequence_of_predicate_values
  (
    (sequence_ordering)? @append_space
    .
    (sequence_uniqueness)
    "}"? @append_space
  )
  (
    "[" @append_space
    element: (_)* @append_space
  )
)

;; no additional spacing: (value_constructor)

(mapping_value
  domain: (simple_value) @append_space
  range: (value) @prepend_space
)
