# Suggested query development workflow

In order to work productively on query files, the following is one
suggested way to work:

1. If you're working on a new language, follow the steps in [the previous chapter](adding-a-new-language.md).

2. Add a snippet of code you want to format to `topiary-cli/tests/samples/input/mylanguage.mlg`.

3. Add the properly formatted version of the code to `topiary-cli/tests/samples/expected/mylanguage.mlg`.

4. Run:

   ```sh
   cargo test \
     --no-default-features \
     -F mylanguage \
     -p topiary-cli \
     input_output_tester \
     -- --nocapture
   ```

   Provided it works, it should output a _lot_ of log messages. Copy
   that output to a text editor. You are particularly interested in the
   CST output that starts with a line like this: `CST node: {Node
   compilation_unit (0, 0) - (5942, 0)} - Named: true`.

> **Note**\
> As an alternative to using the debugging output, the [`visualise`
> subcommand](../cli/usage/visualise.md) line option exists to output
> the Tree-sitter syntax tree in a variety of formats.

5. The test run will output all the differences between the actual
   output and the expected output, e.g. missing spaces between tokens.
   Pick a difference you would like to fix, and find the line number and
   column in the input file.

<!-- FIXME: Is this still true? I seem to remember this being changed to 1-based output -->
> **Note**\
> Keep in mind that the CST output uses 0-based line and column numbers,
> so if your editor reports line 40, column 37, you probably want line
> 39, column 36.

6. In the CST debug or visualisation output, find the nodes in this
   region, such as the following:

   ```
   [DEBUG atom_collection] CST node:   {Node constructed_type (39, 15) - (39, 42)} - Named: true
   [DEBUG atom_collection] CST node:     {Node type_constructor_path (39, 15) - (39, 35)} - Named: true
   [DEBUG atom_collection] CST node:       {Node type_constructor (39, 15) - (39, 35)} - Named: true
   [DEBUG atom_collection] CST node:     {Node type_constructor_path (39, 36) - (39, 42)} - Named: true
   [DEBUG atom_collection] CST node:       {Node type_constructor (39, 36) - (39, 42)} - Named: true
   ```

7. This may indicate that you would like spaces after all
   `type_constructor_path` nodes:

   ```scheme
   (type_constructor_path) @append_space
   ```

   Or, more likely, you just want spaces between pairs of them:

   ```scheme
   (
     (type_constructor_path) @append_space
     .
     (type_constructor_path)
   )
   ```

   Or maybe you want spaces between all children of `constructed_type`:

   ```scheme
   (constructed_type
     (_) @append_space
     .
     (_)
   )
   ```

8. Run `cargo test` again, to see if the output is better now, and then
   return to step 5.
