# Okta Expression Language Factory (oelfactory)

We work a lot with Okta and its expression language,
so it would be nice to have a way to make it less error prone.

This is an attempt at that.

## tree-sitter-oel

A tree sitter grammar for Okta Expression Language.

It is probably close to being in a state to be submitted somewhere,
it just needs to be battle tested.

## tower-lsp-oel

Uses the rust bindings built by tree-sitter-oel
to try and provide an LSP server to add hints
etc. when you hover over things.

Currently provides syntax highlighting in VSCode
via semantic highlighting, but more is possible!
