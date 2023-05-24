#!/usr/bin/env bash
nix run . -- --help |
{
    while IFS= read -r line
    do
        grep -Fxq "$line" README.md
        if [ $? -ne 0 ]
        then
            echo "Usage is not correctly documented in README.md."
            exit 1
        fi
    done

    echo "Usage is correctly documented in README.md."
}
