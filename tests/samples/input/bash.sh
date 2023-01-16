#!/usr/bin/env bash

# Here is a comment
do_a_thing
produce | consume

if some_command
then
do_something
another_thing --foo    --bar
fi


if [[ -e "/some/file" ]]|| true; then
  foo
elif !((1==0))
then
  bar
  baz
else
    baz \
  && quux || xyzzy&
fi

multi \
| line |& pipeline
