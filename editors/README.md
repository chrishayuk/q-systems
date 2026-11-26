# Q-systems in your editor

Two pieces give an editor an understanding of a Q-system:

- **`qsys lsp`**, a language server. It is the `qsys` program itself,
  given the argument `lsp`, and it works with any editor that speaks the
  Language Server Protocol.
- **A Visual Studio Code extension**, in [`vscode`](vscode). It bundles
  `qsys` compiled to WebAssembly, and adds grammars and run commands.

Both read texts with the reconstruction's own reader, so the editor and
`qsys` cannot disagree about what a text means.

There are two kinds of file:

- **`.q`**, a treatment: rules, phases, comments;
- **`.dat`** (or `.qdat`), its data: graphs of strings.

## The language server

`qsys lsp` talks on standard input and output.

- **Diagnostics.**
  - In a treatment, anything not in the grammar of C70's appendix.
  - Also C70's restriction on parameters: a parameter of the right member
    or the condition must be in the left member (p. 27), or the other way
    round under `-INV-` (p. 33).
  - In data, anything not in the grammar, and graphs that break C70's
    conditions: one entry and one exit, and every path into a vertex written
    before any path out of it.
- **Hover** on a rule shows it as the machine applies it, with:
  - `--` filled in;
  - the members exchanged under `-INV-`;
  - its phase and number;
  - its parameters.

  Hover on a parameter says what it stands for. On a word of the language
  (`-INV-`, `-DANS-`, `$$`, `--` …), it says what the word does, with the page
  of C70.
- **Semantic tokens.** The legend is:
  - `comment`;
  - `keyword` (`-INV-`, `-REQ-`, `-DET-`);
  - `operator` (`==`, `--`, `$$`, `/`, `(.`, `.)`);
  - `parameter`;
  - `number` (vertices);
  - `macro` (`-DANS-`, `-HORS-`, `-ET-`, `-OU-`, `-NON-`, `-NUL-`, `=`, `≠`).
- **Document symbols**: the phases of a treatment, and the rules of each.
- **Completion** of the words of the language.

The same server, compiled to WebAssembly, gives the hover text in the
playground (`web/lsp-worker.js`).

## Other editors

Any editor with an LSP client can start `qsys lsp`. For Neovim:

```lua
vim.filetype.add({ extension = { q = "qsystems", dat = "qsystems" } })
vim.lsp.start({ name = "qsys", cmd = { "qsys", "lsp" }, root_dir = vim.fn.getcwd() })
```
