#!/usr/bin/env bash

set -e

cd web-playground/public/scripts/

WORKDIR=$(mktemp -d)
BLUE='\033[0;34m'
GREEN='\033[0;32m'
NC='\033[0m'

echo -e "${BLUE}Updating all Topiary grammars. This process can take a few minutes."

json() {
  echo -e "${BLUE}JSON: Fetching${NC}"
  git clone --depth=1 https://github.com/tree-sitter/tree-sitter-json.git "${WORKDIR}/tree-sitter-json" &> /dev/null
  echo -e "${BLUE}JSON: Building${NC}"
  tree-sitter build-wasm "${WORKDIR}/tree-sitter-json"
  echo -e "${GREEN}JSON: Done${NC}"
}

nickel() {
  echo -e "${BLUE}Nickel: Fetching${NC}"
  git clone --depth=1 https://github.com/nickel-lang/tree-sitter-nickel.git "${WORKDIR}/tree-sitter-nickel" &> /dev/null
  echo -e "${BLUE}Nickel: Building${NC}"
  tree-sitter build-wasm "${WORKDIR}/tree-sitter-nickel"
  echo -e "${GREEN}Nickel: Done${NC}"
}

ocaml() {
  echo -e "${BLUE}OCaml: Fetching${NC}"
  git clone --depth=1 https://github.com/tree-sitter/tree-sitter-ocaml.git "${WORKDIR}/tree-sitter-ocaml" &> /dev/null
  echo -e "${BLUE}OCaml: Building${NC}"
  tree-sitter build-wasm "${WORKDIR}/tree-sitter-ocaml/ocaml"
  tree-sitter build-wasm "${WORKDIR}/tree-sitter-ocaml/interface/"
  echo -e "${GREEN}OCaml: Done${NC}"
}

ocamllex() {
  echo -e "${BLUE}OCamllex: Fetching${NC}"
  git clone --depth=1 https://github.com/314eter/tree-sitter-ocamllex.git "${WORKDIR}/tree-sitter-ocamllex" &> /dev/null
  echo -e "${BLUE}OCamllex: Building${NC}"
  tree-sitter build-wasm "${WORKDIR}/tree-sitter-ocamllex"
  echo -e "${GREEN}Ocamllex: Done${NC}"
}

rust() {
  echo -e "${BLUE}Rust: Fetching${NC}"
  git clone --depth=1 https://github.com/tree-sitter/tree-sitter-rust.git "${WORKDIR}/tree-sitter-rust" &> /dev/null
  echo -e "${BLUE}Rust: Building${NC}"
  tree-sitter build-wasm "${WORKDIR}/tree-sitter-rust"
  echo -e "${GREEN}Rust: Done${NC}"
}


toml() {
  echo -e "${BLUE}TOML: Fetching${NC}"
  git clone --depth=1 https://github.com/tree-sitter/tree-sitter-toml.git "${WORKDIR}/tree-sitter-toml" &> /dev/null
  echo -e "${BLUE}TOML: Building${NC}"
  tree-sitter build-wasm "${WORKDIR}/tree-sitter-toml"
  echo -e "${GREEN}TOML: Done${NC}"
}

tree-sitter-query() {
  echo -e "${BLUE}Query: Fetching${NC}"
  git clone --depth=1 https://github.com/nvim-treesitter/tree-sitter-query.git "${WORKDIR}/tree-sitter-query" &> /dev/null
  echo -e "${BLUE}Query: Building${NC}"
  tree-sitter build-wasm "${WORKDIR}/tree-sitter-query"
  echo -e "${GREEN}Query: Done${NC}"
}

(trap 'kill 0' SIGINT; json & nickel & ocaml & ocamllex & rust & toml & tree-sitter-query & wait)

echo -e "${BLUE}Cleanup...${NC}"
rm -rf "${WORKDIR}"
echo -e "${GREEN}Done! All grammars have been updated${NC}"
