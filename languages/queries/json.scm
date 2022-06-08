; We want every object and array to have the { start a newline. So we match on
; the named object/array followed by the first anonymous node { or [.
(object
  . ("{") @append_hardline
)

(array
  . ("[") @append_hardline
)

; Pairs should always end with a newline. Pairs come in two kinds, ones with a
; trailing comma, and those without. Those without are the last
; pair of an object.
((pair) . ",") @append_hardline

; Pairs without a trailing comma are last pair of an object.
(object
  ((pair) @append_hardline . "}")
)

; Items in an array must have a newline after. See also the pairs above.
(array
  ((_) . ",") @append_hardline
)

(array
  ((_) @append_hardline . "]")
)

; Children of arrays/objects should be indented
(array
  _* @indented
)

(object
  _* @indented
)

; By default our tool produces no spaces. We wish to add a space between the ":" and the following value.
(pair
  (":" @append_space . value: (_))
)
