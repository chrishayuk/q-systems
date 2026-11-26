# Q-systems for Visual Studio Code

Colours, problems as you type, hover and a run button for
[Q-systems](https://github.com/chrishayuk/q-systems), the graph-rewriting
language Alain Colmerauer designed at the Université de Montréal for the
TAUM machine-translation project (1969-70), reconstructed from his 1970
report.

- `.q` files are treatments: rules, phases separated by `-REQ-`, comments.
- `.dat` files are their data: graphs of strings, `-0- A + B -1-`.

## Features

- **Problems as you type**, in treatments and data:
  - anything not in C70's grammar;
  - a parameter that breaks the restriction of C70 pp. 27 and 33;
  - a graph with paths in the wrong order.
- **Hover** on a rule: the rule as the machine applies it, with `--` filled in
  and the members exchanged under `-INV-`. On a parameter or a word of the
  language: what it does, with its page of the report.
- **Colours** from TextMate grammars and from the server's semantic tokens.
- **Outline**: the phases of the treatment and their rules.
- **Run This Treatment on Its Data** (⌘↵ / Ctrl+Enter, or the ▶ in the
  editor's title bar). It runs a `.q` file on the `.dat` of the same name,
  or a `.dat` file under the `.q` of the same name; failing that, it asks
  for the other file. It prints the data and the graph after each phase, as
  the TAUM line printer did. *Run This Treatment, Showing Every Application
  of Every Rule* adds `--det`.

Everything runs on `qsys` compiled to WebAssembly, inside the extension, so
nothing else needs installing. To use a native build instead, set
`qsystems.path`. `qsystems.width` folds printed lines at a width, as the
printer did.

## Building and testing

```sh
npm install
npm run build          # server/qsys.wasm and dist/extension.js
npm test               # the grammar, the language server (WebAssembly and native), the runner
npm run test:vscode    # the extension in a real VS Code
npm run package        # a .vsix
```
