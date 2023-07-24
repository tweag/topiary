#!/usr/bin/env bash

set -euo pipefail

get-cli-usage() {
  # Get the help text from the CLI and strip trailing whitespace
  local subcommand="${1-ROOT}"

  case "${subcommand}" in
    "ROOT") nix run . -- --help;;
    *)      nix run . -- "${subcommand}" --help;;
  esac \
  | sed 's/\s*$//'
}

get-readme-usage() {
  # Get the help text from the README and strip trailing whitespace
  local subcommand="${1-ROOT}"

  sed --quiet "/usage:start:${subcommand}/,/usage:end:${subcommand}/ { //!p; }" README.md \
  | sed --quiet '/```/ !p; s/\s*$//;'
}

main() {
  local -a subcommands=(ROOT fmt vis cfg)

  local _diff
  local _subcommand
  for _subcommand in "${subcommands[@]}"; do
    if ! _diff=$(diff <(get-readme-usage "${_subcommand}") <(get-cli-usage "${_subcommand}")); then
      >&2 echo "Usage is not correctly documented in README.md for the ${_subcommand} subcommand!"
      echo "${_diff}"
      exit 1
    fi
  done

  >&2 echo "Usage is correctly documented in README.md"
}

main
