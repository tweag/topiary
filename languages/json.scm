; Configuration
(#language! json)

; Sometimes we want to indicate that certain parts of our source text should
; not be formatted, but taken as is. We use the leaf capture name to inform the
; tool of this.
(string) @leaf

; We want every object and array to have the { start a softline. So we match on
; the named object/array followed by the first anonymous node { or [.
(object
  . 
  "{" @append_empty_softline
)

(array
  . 
  "[" @append_empty_softline
)

; Pairs should always end with a softline. Pairs come in two kinds, ones with a
; trailing comma, and those without. Those without are the last
; pair of an object.
(object
  "," @append_spaced_softline
)

; Pairs without a trailing comma are last pair of an object.
(object
  (pair) @append_spaced_softline
  .
)

; Items in an array must have a softline after. See also the pairs above.
(array
  "," @append_spaced_softline
)

(array
  (_) @append_spaced_softline
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
  "}" @prepend_indent_end
)

(array
  "[" @append_indent_start
)

(array
  "]" @prepend_indent_end
)
