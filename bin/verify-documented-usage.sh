#!/usr/bin/env nix-shell
#!nix-shell -i bash --packages diffutils gnused
#shellcheck shell=bash

set -euo pipefail

readonly FENCE='```'

get-cli-usage() {
  # Get the help text from the CLI
  local subcommand="${1-ROOT}"

  case "${subcommand}" in
    "ROOT") nix run . -- --help;;
    *)      nix run . -- "${subcommand}" --help;;
  esac
}

get-readme-usage() {
  # Get the help text from the README
  local subcommand="${1-ROOT}"

  sed --quiet "
    /usage:start:${subcommand}/, /usage:end:${subcommand}/ {
      //d          # Delete the markers (last pattern)
      /${FENCE}/d  # Delete the code fences
      p            # Print anything else
    }
  " README.md
}

diff-usage() {
  # Generate a diff between the README and CLI help text
  local subcommand="${1-ROOT}"

  diff --text \
       --ignore-all-space \
       <(get-readme-usage "${subcommand}") \
       <(get-cli-usage "${subcommand}")
}

main() {
  local -a subcommands=(ROOT format visualise config completion prefetch)

  local _diff
  local _subcommand
  for _subcommand in "${subcommands[@]}"; do
    if ! _diff=$(diff-usage "${_subcommand}"); then
      >&2 echo "Usage is not correctly documented in README.md for the ${_subcommand} subcommand!"
      echo "${_diff}"
      exit 1
    fi
  done

  >&2 echo "Usage is correctly documented in README.md"
}

main
