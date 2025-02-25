# Vertical spacing

<!----------------------------------------------------------------------
TODO: To decant from README

* @allow_blank_line_before
* @{append,prepend}_{empty,spaced,input}_softline
* @{append,prepend}_hardline
* Understanding the different newline captures
* #{single,multi}_line_only!

Plus motivate reason single/multi-line-ness
----------------------------------------------------------------------->

<!-- FIXME Moved from worked example; incorporate here, somewhere -->
The formatter goes through the CST nodes and detects all that are
spanning more than one line. This is interpreted to be an indication
from the programmer who wrote the input that the node in question should
be formatted as multi-line. Any other nodes will be formatted as
single-line. Whenever a query match has inserted a _softline_, it will
be expanded to a newline if the node is multi-line, or to a space or
nothing if the node is single-line, depending on whether
`@append_spaced_softline` or `@append_empty_softline` was used.
