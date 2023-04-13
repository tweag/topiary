; Configuration
(#language! nickel)

;; General Spacing

; The following nodes in our source text should not be formatted
[
  (static_string)
  (str_chunks_single)
  (str_chunks_multi)
  (builtin)
] @leaf

; Allow a blank line before the following nodes
[
  (comment)
  (record_field)
  (record_last_field)
] @allow_blank_line_before

; Surround with spaces: keywords, operators, annotation markers
(
  [
    "if"
    "then"
    "else"
    "forall"
    "in"
    "let"
    "match"
    "null"
    "true"
    "false"
    "fun"
    "import"
    "default"
    "doc"
    "force"
    "optional"
    "priority"
    "|"
    ":"
    "?"
    "="
    "rec"
    "Array"
    "Dyn"
    "Num"
    "Bool"
    "Str"
    "->"
    "=>"
    ; Infix operators
    "++"
    "@"
    "*"
    "/"
    "%"
    "+"
    "-"
    "!"
    "&"
    "|>"
    "<"
    "<="
    ">"
    ">="
    "=="
    "!="
    "&&"
    "||"
  ] @prepend_space @append_space
)

; Don't insert spaces before the following delimiters
; NOTE This will destroy the space in a polymorphic record tail. For
; example: forall. { x: Number; a } -> {; a}
; WARNING We don't include "." as it is very common for it to appear in
; string interpolation, for record field access, which will manifest the
; bug documented in Issue #395. The remaining delimiters in this
; alternation can also appear in such contexts, but they're much less
; likely; i.e., this is a trade-off, to avoid over-complicating the
; formatting rules.
[
  ","
  ";"
] @prepend_antispace

; Don't insert spaces immediately inside parentheses. In a multi-line
; context, start an indentation block
; WARNING Using parentheses in string interpolation will manifest the
; bug documented in Issue #395
(atom
  .
  "(" @append_empty_softline @append_indent_start @append_antispace
  ")" @prepend_antispace @prepend_indent_end @prepend_empty_softline
  .
)

; Don't insert spaces between infix operators and their operand
(infix_expr
  .
  [
    "-"
    (infix_u_op_5 "!")
  ] @append_antispace
  .
  (infix_expr)
  .
)

; Flow a chain of infix expressions over new lines, in a multi-line
; context. Note that we _don't_ want this to happen for comparison
; operators, which fall under nodes (infix_b_op_7) and (infix_b_op_8).
(uni_term
  (#scope_id! "infix_chain")
  (infix_expr) @begin_scope
) @end_scope

(infix_expr
  (#scope_id! "infix_chain")
  (infix_expr)
  .
  [
    (infix_b_op_2) ; ++ and @
    (infix_b_op_3) ; *, / and %
    (infix_b_op_4) ; + and -
    (infix_b_op_6) ; & and |>
    (infix_lazy_b_op_9) ; &&
    (infix_lazy_b_op_10) ; ||
  ] @prepend_spaced_scoped_softline
  .
  (infix_expr)
)

; Surround all polymorphic type variables with spaces
(forall
  (ident) @prepend_space
)

;; Comments

(comment) @prepend_input_softline @append_hardline

;; Bound Expressions
; i.e., Let expressions and record fields

; The default, multi-line behaviour for the RHS of a bound expression is
; for it to start an indentation block on a new line. However, the
; following idiomatic exceptions should not behave in this way:
;
; * Record literals                   { ... }
; * Array literals                    [ ... ]
; * Enum literals                     [| ... |]
; * Parentheticals                    ( ... )
; * Function declarations             fun ... => ...
; * Match statements                  match { ... }
; * Multi-line and symbolic strings   m%"..."% / xxx-s%"..."%
;
; These should remain in-line.

(_
  "=" @append_spaced_softline @append_indent_start
  .
  (term
    (uni_term
      [
        ; There is scope for factoring these patterns with
        ; embedded alternations. Keeping them separate is probably more
        ; efficient to process and certainly easier to read.

        ; Record literals
        (infix_expr (applicative (record_operand (atom (uni_record)))))

        ; Array literals
        (infix_expr (applicative (record_operand (atom "["))))

        ; Enum literals
        (infix_expr (applicative (record_operand (atom (type_atom "[|")))))

        ; Parentheticals
        (infix_expr (applicative (record_operand (atom "("))))

        ; Function declarations
        (fun_expr)

        ; Match statements
        (infix_expr (applicative (match_expr)))

        ; Multi-line and symbolic strings
        (infix_expr (applicative (record_operand (atom (str_chunks)))))
      ]? @do_nothing
    )
  ) @append_indent_end
)

; A let expression looks like:
;
;   let [rec] IDENT = EXPR in EXPR
;
; The formatting for the bound expression is handled by the above rules,
; which also apply to record field values. The result expression (i.e.,
; after the "in") should appear on an new line, if that is multi-line.
; We don't start an indentation block for the result expression, to
; avoid long diagonals in a series of let expressions (which is
; idiomatic).

; TODO Tidy this up

(let_expr
  (#scope_id! "let_result")
  (let_in_block
    "in" @begin_scope
  )
  (term) @end_scope
)

(let_expr
  (#scope_id! "let_result")
  (term) @prepend_spaced_scoped_softline
)

;; Annotations

; Create a scope that covers at least all annotation atoms; that is,
; children of the (annot) node. When an assignment is also involved, we
; have another scope that extents to also cover the equals sign.
(
  (#scope_id! "signature")
  _ @begin_scope
  .
  (annot) @end_scope
)

(
  (#scope_id! "assignee")
  (_) @begin_scope
  .
  (annot)*
  .
  "=" @end_scope
)

; Start an indentation block from the start of the annotations to the
; end of the enclosing node
(_
  (annot) @prepend_indent_start
) @append_indent_end

; Put each annotation and the equals sign on a new line, in a multi-line
; context.
(
  (#scope_id! "signature")
  (annot_atom) @prepend_spaced_scoped_softline
)

; FIXME This breaks idempotency for multi-line let expressions with two
; or more annotations on the same line as the equals sign! For example:
;
;  let x
;    : TYPE | ANNOT = 1 in x
;
; It also doesn't do the right thing (push the equals sign to a new
; line) if it lives on the same line as the last annotation in the
; input. For example:
;
;   let x
;     : TYPE = 1 in x
;
; These forms are not attested in the Nickel standard library, as of
; writing.
(_
  (#scope_id! "assignee")
  "=" @prepend_spaced_scoped_softline
)

; Break a multi-line polymorphic type annotation after the type
; variables, starting an indentation block
(forall
  "." @append_spaced_softline @append_indent_start
) @append_indent_end

;; Functions

; Start a function's definition on a new line, in a multi-line context.
; This also defines an indentation block.
(fun_expr
  (#scope_id! "function_definition")
  "=>" @begin_scope @append_indent_start
) @append_indent_end @end_scope

(fun_expr
  (#scope_id! "function_definition")
  (term) @prepend_spaced_scoped_softline
)

(fun_expr
  (pattern) @append_space
)

; The applicative operator is a space, but in a multi-line context, we'd
; like the operands to start on their own line, each indented.
(infix_expr
  (#scope_id! "applicative_chain")
  (applicative) @begin_scope
) @end_scope

; NOTE Unlike infix chains, applicatives bind to the left. So rather
; than creating a single indent block for all operands, we have to
; create one for each operand independently.
(
  (#scope_id! "applicative_chain")
  (applicative
    (applicative) @append_spaced_scoped_softline @append_indent_start
  ) @append_indent_end
)

;; Conditionals

; Flow multi-line match cases into an indented block after the =>
(match_case
  "=>" @append_spaced_softline @append_indent_start
) @append_indent_end

; if...then...else expressions can either be single or multi-line. In a
; multi-line context, they will be formatted like so:
;
;   if CONDITION then
;     TRUE_TERM
;   else
;     FALSE_TERM
;
; NOTE If the FALSE_TERM is another if...then...else expression, in a
; multi-line context, then the indentation block is "cancelled" to
; create the illusion of an "else if" term:
;
;   if CONDITION1 then
;     TRUE_TERM1
;   else if CONDITION2 then
;     TRUE_TERM2
;   else
;     FALSE_TERM2
;
; This style has precedent from the manually formatted stdlib. (An
; alternative style is to give the "then" token its own line.)
(ite_expr
  "then" @append_spaced_softline @append_indent_start
  t1: (term) @append_indent_end @append_spaced_softline
)

(ite_expr
  "else" @append_spaced_softline @append_indent_start
  t2: (term
    ; Don't apply formatting if an "else" is followed by an "if"
    (uni_term (ite_expr))? @do_nothing
  ) @append_indent_end
)

;; Container Types
; i.e., Arrays, records (and dictionaries, vicariously) and enums

; We don't want to add spaces/new lines in empty records, so the
; following query only matches if a named node exists within the record
; NOTE This rule also applies to (match) and (destruct) patterns
(_
  (#scope_id! "container")
  .
  "{" @append_spaced_softline @append_indent_start @begin_scope
  (_)
  "}" @prepend_indent_end @prepend_spaced_softline @end_scope
  .
)

; As with records, we don't want internal spacing to apply if the array
; is empty; this restriction also applies to arrays of exactly one
; element, which is usually short (e.g., [x]).
(_
  (#scope_id! "container")
  .
  "[" @append_spaced_softline @append_indent_start @begin_scope
  (_)
  (_)
  "]" @prepend_indent_end @prepend_spaced_softline @end_scope
  .
)

; It doesn't really make sense for an enum to
; have no members, so we ignore that case
(_
  (#scope_id! "container")
  .
  "[|" @append_spaced_softline @append_indent_start @begin_scope
  "|]" @prepend_indent_end @prepend_spaced_softline @end_scope
  .
)

(
  (#scope_id! "container")
  [
    ","
    ";"
  ] @append_spaced_scoped_softline
)
