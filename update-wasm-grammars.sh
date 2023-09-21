#!/usr/bin/env bash

set -eu

cd web-playground/public/scripts/

WORKDIR="$(mktemp -d)"
readonly WORKDIR
trap 'echo -e "${BLUE}Cleanup...${NC}"; rm -rf "${WORKDIR}"' EXIT

BLUE="$(tput setaf 4)"
readonly BLUE

ORANGE="$(tput setaf 3)"
readonly ORANGE

GREEN="$(tput setaf 2)"
readonly GREEN

NC="$(tput sgr0)"
readonly NC

echo -e "${BLUE}Updating all Topiary grammars. This process can take a few minutes."

ref_for_language() {
  toml2json "$(git rev-parse --show-toplevel)/Cargo.lock" | jq ".package[] | select(.name==\"tree-sitter-$1\") | .source" | sed -e 's/.*#\(.*\)"/\1/'
}

json() {
  echo -e "${BLUE}JSON: Fetching${NC}"
  git clone --depth=1 https://github.com/tree-sitter/tree-sitter-json.git "${WORKDIR}/tree-sitter-json" &> /dev/null
  REV=$(ref_for_language "json")
  pushd "${WORKDIR}/tree-sitter-json" &> /dev/null
    git checkout "$REV" &> /dev/null
  popd &> /dev/null
  echo -e "${ORANGE}JSON: Building${NC}"
  tree-sitter build-wasm "${WORKDIR}/tree-sitter-json"
  echo -e "${GREEN}JSON: Done${NC}"
}

nickel() {
  echo -e "${BLUE}Nickel: Fetching${NC}"
  git clone --depth=1 https://github.com/nickel-lang/tree-sitter-nickel.git "${WORKDIR}/tree-sitter-nickel" &> /dev/null
  REV=$(ref_for_language "nickel")
  pushd "${WORKDIR}/tree-sitter-nickel" &> /dev/null
    git checkout "$REV" &> /dev/null
  popd &> /dev/null
  echo -e "${ORANGE}Nickel: Building${NC}"
  tree-sitter build-wasm "${WORKDIR}/tree-sitter-nickel"
  echo -e "${GREEN}Nickel: Done${NC}"
}

ocaml() {
  echo -e "${BLUE}OCaml: Fetching${NC}"
  git clone --depth=1 https://github.com/tree-sitter/tree-sitter-ocaml.git "${WORKDIR}/tree-sitter-ocaml" &> /dev/null
  REV=$(ref_for_language "ocaml")
  pushd "${WORKDIR}/tree-sitter-ocaml" &> /dev/null
    git checkout "$REV" &> /dev/null
  popd &> /dev/null
  echo -e "${ORANGE}OCaml: Building${NC}"
  tree-sitter build-wasm "${WORKDIR}/tree-sitter-ocaml/ocaml"
  tree-sitter build-wasm "${WORKDIR}/tree-sitter-ocaml/interface/"
  echo -e "${GREEN}OCaml: Done${NC}"
}

ocamllex() {
  echo -e "${BLUE}OCamllex: Fetching${NC}"
  git clone --depth=1 https://github.com/314eter/tree-sitter-ocamllex.git "${WORKDIR}/tree-sitter-ocamllex" &> /dev/null
  REV=$(ref_for_language "ocamllex")
  pushd "${WORKDIR}/tree-sitter-ocamllex" &> /dev/null
    git checkout "$REV" &> /dev/null
  popd &> /dev/null
  echo -e "${ORANGE}OCamllex: Building${NC}"
  tree-sitter build-wasm "${WORKDIR}/tree-sitter-ocamllex"
  echo -e "${GREEN}Ocamllex: Done${NC}"
}

rust() {
  echo -e "${BLUE}Rust: Fetching${NC}"
  git clone --depth=1 https://github.com/tree-sitter/tree-sitter-rust.git "${WORKDIR}/tree-sitter-rust" &> /dev/null
  REV=$(ref_for_language "rust")
  pushd "${WORKDIR}/tree-sitter-rust" &> /dev/null
    git checkout "$REV" &> /dev/null
  popd &> /dev/null
  echo -e "${ORANGE}Rust: Building${NC}"
  tree-sitter build-wasm "${WORKDIR}/tree-sitter-rust"
  echo -e "${GREEN}Rust: Done${NC}"
}


toml() {
  echo -e "${BLUE}TOML: Fetching${NC}"
  git clone --depth=1 https://github.com/tree-sitter/tree-sitter-toml.git "${WORKDIR}/tree-sitter-toml" &> /dev/null
  REV=$(ref_for_language "toml")
  pushd "${WORKDIR}/tree-sitter-toml" &> /dev/null
    git checkout "$REV" &> /dev/null
  popd &> /dev/null
  echo -e "${ORANGE}TOML: Building${NC}"
  tree-sitter build-wasm "${WORKDIR}/tree-sitter-toml"
  echo -e "${GREEN}TOML: Done${NC}"
}

tree-sitter-query() {
  echo -e "${BLUE}Query: Fetching${NC}"
  git clone --depth=1 https://github.com/nvim-treesitter/tree-sitter-query.git "${WORKDIR}/tree-sitter-query" &> /dev/null
  REV=$(ref_for_language "query")
  pushd "${WORKDIR}/tree-sitter-query" &> /dev/null
    git checkout "$REV" &> /dev/null
  popd &> /dev/null
  echo -e "${ORANGE}Query: Building${NC}"
  tree-sitter build-wasm "${WORKDIR}/tree-sitter-query"
  echo -e "${GREEN}Query: Done${NC}"
}

(trap 'kill 0' SIGINT; json & nickel & ocaml & ocamllex & rust & toml & tree-sitter-query & wait)

echo -e "${GREEN}Done! All grammars have been updated${NC}"
