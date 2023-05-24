#!/usr/bin/env bash
nix run . -- --help > usage.txt

cat usage.txt |
{
    while IFS= read -r line
    do
        if ! grep -Fxq "$line" README.md
        then
            echo "Usage is not correctly documented in README.md. Update the file with the following:"
            cat usage.txt
            exit 1
        fi
    done

    echo "Usage is correctly documented in README.md."
}
