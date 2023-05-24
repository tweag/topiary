#!/usr/bin/env bash
nix run . -- --help > usage.txt

cat usage.txt |
{
    while IFS= read -r line
    do
        grep -Fxq "$line" README.md
        if [ $? -ne 0 ]
        then
            echo "Usage is not correctly documented in README.md. Update the file with the following:"
            cat usage.txt
            exit 1
        fi
    done

    echo "Usage is correctly documented in README.md."
}
