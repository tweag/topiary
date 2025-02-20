#!/usr/bin/env bash

set -eu

cd web-playground/public/scripts/

# We don't use mktemp because Tree-sitter doesn't work across disparate filesystems
WORKDIR=./tmp/
mkdir -p $WORKDIR
readonly WORKDIR
trap 'echo -e "${BLUE}Cleanup...${NC}"; rm -rf "${WORKDIR}" a.out.{js,wasm}' EXIT

BLUE="$(tput setaf 4)"
readonly BLUE

ORANGE="$(tput setaf 3)"
readonly ORANGE

GREEN="$(tput setaf 2)"
readonly GREEN

NC="$(tput sgr0)"
readonly NC

CONFIG=$(nickel export --format json "$(git rev-parse --show-toplevel)/topiary-config/languages.ncl")
readonly CONFIG

echo -e "${BLUE}Updating all Topiary grammars. This process can take a few minutes."

ref_for_language() {
  echo "$CONFIG" | jq -r ".languages.$1.grammar.rev"
}

repo_for_language() {
  echo "$CONFIG" | jq -r ".languages.$1.grammar.git"
}

json() {
  echo -e "${BLUE}JSON: Fetching${NC}"
  REPO=$(repo_for_language "json")
  git clone "${REPO}" "${WORKDIR}/tree-sitter-json" &> /dev/null
  REV=$(ref_for_language "json")
  pushd "${WORKDIR}/tree-sitter-json" &> /dev/null
    git checkout "$REV" &> /dev/null
  popd &> /dev/null
  echo -e "${ORANGE}JSON: Building${NC}"
  tree-sitter build --wasm "${WORKDIR}/tree-sitter-json"
  echo -e "${GREEN}JSON: Done${NC}"
}

nickel() {
  echo -e "${BLUE}Nickel: Fetching${NC}"
  REPO=$(repo_for_language "nickel")
  git clone "${REPO}" "${WORKDIR}/tree-sitter-nickel" &> /dev/null
  REV=$(ref_for_language "nickel")
  pushd "${WORKDIR}/tree-sitter-nickel" &> /dev/null
    git checkout "$REV" &> /dev/null
  popd &> /dev/null
  echo -e "${ORANGE}Nickel: Building${NC}"
  tree-sitter build --wasm "${WORKDIR}/tree-sitter-nickel"
  echo -e "${GREEN}Nickel: Done${NC}"
}

ocaml() {
  echo -e "${BLUE}OCaml: Fetching${NC}"
  REPO=$(repo_for_language "ocaml")
  git clone "${REPO}" "${WORKDIR}/tree-sitter-ocaml" &> /dev/null
  REV=$(ref_for_language "ocaml")
  pushd "${WORKDIR}/tree-sitter-ocaml" &> /dev/null
    git checkout "$REV" &> /dev/null
  popd &> /dev/null
  echo -e "${ORANGE}OCaml: Building${NC}"
  tree-sitter build --wasm "${WORKDIR}/tree-sitter-ocaml/ocaml"
  tree-sitter build --wasm "${WORKDIR}/tree-sitter-ocaml/interface/"
  echo -e "${GREEN}OCaml: Done${NC}"
}

ocamllex() {
  echo -e "${BLUE}OCamllex: Fetching${NC}"
  REPO=$(repo_for_language "ocamllex")
  git clone "${REPO}" "${WORKDIR}/tree-sitter-ocamllex" &> /dev/null
  REV=$(ref_for_language "ocamllex")
  pushd "${WORKDIR}/tree-sitter-ocamllex" &> /dev/null
    git checkout "$REV" &> /dev/null
  popd &> /dev/null
  echo -e "${ORANGE}OCamllex: Building${NC}"
  tree-sitter build --wasm "${WORKDIR}/tree-sitter-ocamllex"
  echo -e "${GREEN}Ocamllex: Done${NC}"
}

bash() {
  echo -e "${BLUE}Bash: Fetching${NC}"
  REPO=$(repo_for_language "bash")
  git clone "${REPO}" "${WORKDIR}/tree-sitter-bash" &> /dev/null
  REV=$(ref_for_language "bash")
  pushd "${WORKDIR}/tree-sitter-bash" &> /dev/null
    git checkout "$REV" &> /dev/null
  popd &> /dev/null
  echo -e "${ORANGE}Bash: Building${NC}"
  tree-sitter build --wasm "${WORKDIR}/tree-sitter-bash"
  echo -e "${GREEN}Bash: Done${NC}"
}

rust() {
  echo -e "${BLUE}Rust: Fetching${NC}"
  REPO=$(repo_for_language "rust")
  git clone "${REPO}" "${WORKDIR}/tree-sitter-rust" &> /dev/null
  REV=$(ref_for_language "rust")
  pushd "${WORKDIR}/tree-sitter-rust" &> /dev/null
    git checkout "$REV" &> /dev/null
  popd &> /dev/null
  echo -e "${ORANGE}Rust: Building${NC}"
  tree-sitter build --wasm "${WORKDIR}/tree-sitter-rust"
  echo -e "${GREEN}Rust: Done${NC}"
}

toml() {
  echo -e "${BLUE}TOML: Fetching${NC}"
  REPO=$(repo_for_language "toml")
  git clone "${REPO}" "${WORKDIR}/tree-sitter-toml" &> /dev/null
  REV=$(ref_for_language "toml")
  pushd "${WORKDIR}/tree-sitter-toml" &> /dev/null
    git checkout "$REV" &> /dev/null
  popd &> /dev/null
  echo -e "${ORANGE}TOML: Building${NC}"
  tree-sitter build --wasm "${WORKDIR}/tree-sitter-toml"
  echo -e "${GREEN}TOML: Done${NC}"
}

tree-sitter-query() {
  echo -e "${BLUE}Query: Fetching${NC}"
  REPO=$(repo_for_language "tree_sitter_query")
  git clone "${REPO}" "${WORKDIR}/tree-sitter-query" &> /dev/null
  REV=$(ref_for_language "tree_sitter_query")
  pushd "${WORKDIR}/tree-sitter-query" &> /dev/null
    git checkout "$REV" &> /dev/null
  popd &> /dev/null
  echo -e "${ORANGE}Query: Building${NC}"
  tree-sitter build --wasm "${WORKDIR}/tree-sitter-query"
  echo -e "${GREEN}Query: Done${NC}"
}

css() {
  echo -e "${BLUE}CSS: Fetching${NC}"
  REPO=$(repo_for_language "css")
  git clone "${REPO}" "${WORKDIR}/tree-sitter-css" &> /dev/null
  REV=$(ref_for_language "css")
  pushd "${WORKDIR}/tree-sitter-css" &> /dev/null
    git checkout "$REV" &> /dev/null
  popd &> /dev/null
  echo -e "${ORANGE}CSS: Building${NC}"
  tree-sitter build --wasm "${WORKDIR}/tree-sitter-css"
  echo -e "${GREEN}CSS: Done${NC}"
}

openscad() {
  echo -e "${BLUE}OpenSCAD: Fetching${NC}"
  REPO=$(repo_for_language "openscad")
  git clone "${REPO}" "${WORKDIR}/tree-sitter-openscad" &> /dev/null
  REV=$(ref_for_language "opescad")
  pushd "${WORKDIR}/tree-sitter-openscad" &> /dev/null
    git checkout "$REV" &> /dev/null
  popd &> /dev/null
  echo -e "${ORANGE}OpenSCAD: Building${NC}"
  tree-sitter build --wasm "${WORKDIR}/tree-sitter-openscad"
  echo -e "${GREEN}OpenSCAD: Done${NC}"
}


(trap 'kill 0' SIGINT; json & nickel & ocaml & ocamllex & bash & rust & toml & tree-sitter-query & css & openscad & wait)

echo -e "${GREEN}Done! All grammars have been updated${NC}"
