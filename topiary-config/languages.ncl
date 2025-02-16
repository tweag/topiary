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
        rev = "94f5c527b2965465956c2000ed6134dd24daf2a7",
      },
    },

    nickel = {
      extensions = ["ncl"],
      grammar.source.git = {
        git = "https://github.com/nickel-lang/tree-sitter-nickel",
        rev = "88d836a24b3b11c8720874a1a9286b8ae838d30a",
      },
    },

    cfml = {
      extensions = ["cfm"],
      indent = "    ", # 4 spaces
      grammar.source.git = {
        git = "https://github.com/cfmleditor/tree-sitter-cfml.git",
        rev = "979825d2e27ac9e65a217e5132c5c7eef4004b0e",
        subdir = "cfml",
      },
    },

    ocaml = {
      extensions = ["ml"],
      grammar.source.git = {
        git = "https://github.com/tree-sitter/tree-sitter-ocaml.git",
        rev = "9965d208337d88bbf1a38ad0b0fe49e5f5ec9677",
        subdir = "ocaml",
      },
    },

    ocaml_interface = {
      extensions = ["mli"],
      grammar.source.git = {
        git = "https://github.com/tree-sitter/tree-sitter-ocaml.git",
        rev = "9965d208337d88bbf1a38ad0b0fe49e5f5ec9677",
        subdir = "interface",
      },
    },

    ocamllex = {
      extensions = ["mll"],
      grammar.source.git = {
        git = "https://github.com/314eter/tree-sitter-ocamllex.git",
        rev = "4b9898ccbf198602bb0dec9cd67cc1d2c0a4fad2",
      },
    },

    openscad = {
      extensions = ["scad"],
      grammar.source.git = {
        git = "https://github.com/mkatychev/tree-sitter-openscad.git",
        rev = "270e5ff749edfacc84a6e4a434abd4e8b0f70bbe",
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
