{
  languages = {
    bash = {
      extensions = ["sh", "bash"],
      grammar.source.path = "@bash@",
    },

    css = {
      extensions = ["css"],
      grammar.source.path = "@css@",
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
      grammar.source.path = "@json@",
    },

    nickel = {
      extensions = ["ncl"],
      grammar.source.path = "@nickel@",
    },

    ocaml = {
      extensions = ["ml"],
      grammar.source.path = "@ocaml@",
    },

    ocaml_interface = {
      extensions = ["mli"],
      grammar.source.path = "@ocaml_interface@",
    },

    # ocamllex = {
    #   extensions = ["mll"],
    #   grammar.source.path = "@ocamllex@",
    # },

    rust = {
      extensions = ["rs"],
      indent = "    ", # 4 spaces
      grammar.source.path = "@rust@",
    },

    toml = {
      extensions = ["toml"],
      grammar.source.path = "@toml@",
    },

    tree_sitter_query = {
      extensions = ["scm"],
      grammar = {
        source.path = "@tree_sitter_query@",
        symbol = "tree_sitter_query",
      },
    },
  },
}
