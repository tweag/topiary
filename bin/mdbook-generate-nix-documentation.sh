#!/usr/bin/env sh
set -euC

if [ $# -gt 0 ]; then
    if [ "$1" = supports ]; then
        exit 0
    else
        printf >&2 'Unexpected arguments: %s\n' "$*"
        exit 2
    fi
fi

## This script is called from `docs/book`, but works on `nix/`, so we get there.
##
cd ../../nix

## Crawl Nix files with `nixdoc` and generate documentation in Markdown format.
##
content=$(
    printf '# Nix API\n\n'
    for file in utils/*.nix lib/*.nix; do
        nixdoc \
            --file "$file" \
            --category "" \
            --description "" \
            --prefix "lib" \
            --anchor-prefix "" \
            ;
    done
)

## Behave as a mdbook preprocessor: get the JSON input from stdin (an array
## [context, book]), find the book's chapter named "Nix API", replace its
## content with the generated documentation, and return the modified book.
##
jq --arg content "$content" '
    .[1] |
    .sections |= map(
        if .Chapter.name == "Nix API" then (
            .Chapter.content = $content
        ) end
    )
'
