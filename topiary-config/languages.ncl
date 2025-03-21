{
  languages = {
    bash = {
      extensions = ["sh", "bash"],
      grammar.source.git = {
        git = "https://github.com/tree-sitter/tree-sitter-bash.git",
        rev = "d1a1a3fe7189fdab5bd29a54d1df4a5873db5cb1",
      }
    },

    css = {
      extensions = ["css"],
      grammar.source.git = {
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
      grammar.source.git = {
        git = "https://github.com/tree-sitter/tree-sitter-json.git",
        rev = "v0.24.8",
      },
    },

    nickel = {
      extensions = ["ncl"],
      grammar.source.git = {
        git = "https://github.com/nickel-lang/tree-sitter-nickel",
        rev = "88d836a24b3b11c8720874a1a9286b8ae838d30a",
      },
    },

    ocaml = {
      extensions = ["ml"],
      grammar.source.git = {
        git = "https://github.com/tree-sitter/tree-sitter-ocaml.git",
        rev = "a45fda5fe73cda92f2593d16340b3f6bd46674b8",
        subdir = "grammars/ocaml",
      },
    },

    ocaml_interface = {
      extensions = ["mli"],
      grammar.source.git = {
        git = "https://github.com/tree-sitter/tree-sitter-ocaml.git",
        rev = "a45fda5fe73cda92f2593d16340b3f6bd46674b8",
        subdir = "grammars/interface",
      },
    },

    ocamllex = {
      extensions = ["mll"],
      grammar.source.git = {
        git = "https://github.com/314eter/tree-sitter-ocamllex.git",
        rev = "5da5bb7508ac9fd3317561670ef18c126a0fe2aa",
      },
    },

    openscad = {
      extensions = ["scad"],
      grammar.source.git = {
        git = "https://github.com/openscad/tree-sitter-openscad.git",
        rev = "acc196e969a169cadd8b7f8d9f81ff2d30e3e253",
      },
    },

    rust = {
      extensions = ["rs"],
      indent = "    ", # 4 spaces
      grammar.source.git = {
        git = "https://github.com/tree-sitter/tree-sitter-rust.git",
        rev = "e0e8b6de6e4aa354749c794f5f36a906dcccda74",
      },
    },

    sdml = {
      extensions = ["sdm", "sdml"],
      grammar.source.git = {
        git = "https://github.com/sdm-lang/tree-sitter-sdml",
        rev = "056fc1d0b8624aa4e58967c67bb129ebdfa6d563"
      },
    },

    toml = {
      extensions = ["toml"],
      grammar.source.git = {
        git = "https://github.com/tree-sitter/tree-sitter-toml.git",
        rev = "342d9be207c2dba869b9967124c679b5e6fd0ebe",
      },
    },

    tree_sitter_query = {
      extensions = ["scm"],
      grammar = {
        source.git = {
          git = "https://github.com/nvim-treesitter/tree-sitter-query",
          rev = "a0ccc351e5e868ec1f8135e97aa3b53c663cf2df",
        },
        symbol = "tree_sitter_query",
      },
    },
  },
}
