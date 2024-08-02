{
  languages = {
    bash = {
      extensions = ["sh", "bash"],
      grammar = {
        git = "https://github.com/tree-sitter/tree-sitter-bash.git",
        rev = "1b0321ee85701d5036c334a6f04761cdc672e64c",
      }
    },

    css = {
      extensions = ["css"],
      grammar = {
        git = "https://github.com/tree-sitter/tree-sitter-css.git",
        rev = "02b4ee757654b7d54fe35352fd8e53a8a4385d42",
      },
    },

    json = {
      extensions = [
          "json",
          "avsc",
          "geojson",
          "gltf",
          "har",
          "ice",
          "JSON-tmLanguage",
          "jsonl",
          "mcmeta",
          "tfstate",
          "tfstate.backup",
          "topojson",
          "webapp",
          "webmanifest"
        ],
      grammar = {
        git = "https://github.com/tree-sitter/tree-sitter-json.git",
        rev = "94f5c527b2965465956c2000ed6134dd24daf2a7",
      },
    },

    nickel = {
      extensions = ["ncl"],
      grammar = {
        git = "https://github.com/nickel-lang/tree-sitter-nickel",
        rev = "43433d8477b24cd13acaac20a66deda49b7e2547",
      },
    },

    ocaml = {
      extensions = ["ml"],
      grammar = {
        git = "https://github.com/tree-sitter/tree-sitter-ocaml.git",
        rev = "9965d208337d88bbf1a38ad0b0fe49e5f5ec9677",
        subdir = "grammars/ocaml",
      },
    },

    ocaml_interface = {
      extensions = ["mli"],
      grammar = {
        git = "https://github.com/tree-sitter/tree-sitter-ocaml.git",
        rev = "9965d208337d88bbf1a38ad0b0fe49e5f5ec9677",
        subdir = "grammars/interface",
      },
    },

    ocamllex = {
      extensions = ["mll"],
      grammar = {
        git = "https://github.com/314eter/tree-sitter-ocamllex.git",
        rev = "4b9898ccbf198602bb0dec9cd67cc1d2c0a4fad2",
      },
    },

    rust = {
      extensions = ["rs"],
      indent = "    ", # 4 spaces
      grammar = {
        git = "https://github.com/tree-sitter/tree-sitter-rust.git",
        rev = "e0e8b6de6e4aa354749c794f5f36a906dcccda74",
      },
    },

    toml = {
      extensions = ["toml"],
      grammar = {
        git = "https://github.com/tree-sitter/tree-sitter-toml.git",
        rev = "342d9be207c2dba869b9967124c679b5e6fd0ebe",
      },
    },

    tree_sitter_query = {
      extensions = ["scm"],
      grammar = {
        symbol = "tree_sitter_query",
        git = "https://github.com/nvim-treesitter/tree-sitter-query",
        rev = "a0ccc351e5e868ec1f8135e97aa3b53c663cf2df",
      },
    },
  },
}
