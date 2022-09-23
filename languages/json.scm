; Sometimes we want to indicate that certain parts of our source text should
; not be formatted, but taken as is. We use the leaf capture name to inform the
; tool of this.
(string) @leaf

; We want every object and array to have the { start a newline. So we match on
; the named object/array followed by the first anonymous node { or [.
(object
  . 
  "{" @append_hardline
)

(array
  . 
  "[" @append_hardline
)

; Pairs should always end with a newline. Pairs come in two kinds, ones with a
; trailing comma, and those without. Those without are the last
; pair of an object.
(object
  "," @append_hardline
)

; Pairs without a trailing comma are last pair of an object.
(object
  (pair) @append_hardline
  .
)

; Items in an array must have a newline after. See also the pairs above.
(array
  "," @append_hardline
)

(array
  (_) @append_hardline
  .
)

; Append space after colons
(
  ":" @append_space
)

; Children of arrays/objects should be indented
(object
  "{" @append_indent_start
)

(object
  (_) @append_indent_end
  .
)

(array
  "[" @append_indent_start
)

(array
  (_) @append_indent_end
  .
)
