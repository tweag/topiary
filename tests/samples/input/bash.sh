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

for thing in foo bar quux
do
  echo $thing
  rm -rf /
done

select thing in foo bar quux; do
  echo $thing
  break
done

for (( i=0; i<10; i++ )); do
  echo $i
done

while true
do
  echo "Hello world!"
done

until true
do
  echo "Hello world!"
done
