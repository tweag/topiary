#!/usr/bin/env bash
usage="$(nix run . -- --help)"

echo "$usage" |
{
    while IFS= read -r line
    do
        if ! grep -Fxq "$line" README.md
        then
            echo "Usage is not correctly documented in README.md. Update the file with the following:"
            echo "$usage"
            exit 1
        fi
    done

    echo "Usage is correctly documented in README.md."
}
