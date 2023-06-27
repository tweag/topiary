#!/usr/bin/env nix-shell
#!nix-shell -i bash --packages inotify-tools
#shellcheck shell=bash

# "Quick-and-Dirty" Topiary Playground

set -euo pipefail

readonly PROGNAME="$(basename "$0")"

fail() {
  local error="$*"

  cat >&2 <<-EOF
	Error: ${error}

	Usage: ${PROGNAME} (LANGUAGE | QUERY_FILE) [INPUT_SOURCE]

	LANGUAGE can be one of the supported languages (e.g., "ocaml", "rust",
	etc.); alternatively, give the path to the query file itself, as
	QUERY_FILE.

	The INPUT_SOURCE is optional. If not specified, it defaults to trying
	to find the bundled integration test input file for the given language.
	EOF

  exit 1
}

get_sample_input() {
  local language="$1"

  # Only return the first result, presuming there is one
  find topiary/tests/samples/input -name "${language}.*" \
  | head -1
}

format() {
  local query="$1"
  local input="$2"
  local skip_idempotence="${3-1}"

  local -a topiary_args=(
    --query "${query}"
    --input-files "${input}"
  )

  if (( skip_idempotence )); then
    topiary_args+=(--skip-idempotence)
  fi

  cargo run --quiet -- "${topiary_args[@]}"
}

idempotency() {
  local query="$1"
  local input="$2"

  if format "${query}" "${input}" 0 >/dev/null 2>&1; then
    printf "Yes"
  else
    if (( $? == 7 )); then
      printf "No"
    else
      printf "n/a"
    fi
  fi
}

main() {
  local query="${1-}"
  if ! [[ -e "${query}" ]]; then
    query="languages/${query}.scm"
    [[ -e "${query}" ]] || fail "Couldn't find language query file '${query}'"
  fi

  local language="$(basename --suffix=.scm "${query}")"
  local input="${2-$(get_sample_input "${language}")}"
  [[ -e "${input}" ]] || fail "Couldn't find input source file '${input}'"

  # Horizontal rule (this is a function because executing it in a TTY-
  # -less subshell, to assign it to a variable, sets COLUMNS to 0)
  hr() { printf "%${COLUMNS}s\n" "" | tr " " "-"; }

  while true; do
    clear

    hr
    cat <<-EOF
		Query File    ${query}
		Input Source  ${input}
		EOF
    hr

    format "${query}" "${input}" || true
    hr
    printf "Idempotent    %s\n" "$(idempotency "${query}" "${input}")"

    # NOTE Different editors have different strategies for modifying
    # files, so we wait on multiple events. This *may* not be an
    # exhaustive list; you are encouraged to experiment if the
    # playground doesn't refresh when you expect it to!
    inotifywait \
      -qq \
      --event modify \
      --event move_self \
      "${query}" "${input}"
  done

}

main "$@"
