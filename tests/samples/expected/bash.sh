#!/usr/bin/env bash

# Here is a comment

if some_command; then
  do_something
  another_thing --foo --bar
fi

if [[ -e "/some/file" ]]; then
  foo
elif ! (( 1 == 0 )); then
  bar
else
  baz
  quux
fi
