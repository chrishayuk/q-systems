# 8. The tools

## qsys

```sh
qsys PROGRAM DATA              # the data, then the graph after each phase
qsys --det PROGRAM DATA        # and every application of every rule
qsys --width 72 PROGRAM DATA   # fold lines as the line printer did, anywhere
qsys --max-arrows N ...        # a phase that adds more is taken not to converge
qsys check PROGRAM             # the rules as applied: -- filled in, -INV- done
qsys json PROGRAM DATA         # every arrow of every phase, for the playground
qsys lsp                       # the language server
```

`PROGRAM` may be `-`, for standard input. `--data TEXT` gives the data in
place of a file.

`qsys check` is the quickest way to see what `--` and `-INV-` do to a
system:

```sh
qsys check examples/paraphrase.q
```

## The playground

`web/index.html` runs `qsys`, compiled to WebAssembly, in the page. Serve the
folder and open it:

```sh
node web/build.mjs
python3 -m http.server -d web 8000
```

The page has:

- an editor for the treatment and one for the data;
- a tab for each phase and each graph;
- a slider through the phase.

The graph is drawn as C70 p. 29 draws it: the data on a line, and each added
path as a bracket above or below. A step of the slider is one application of
one rule:

- the path it matched shows in blue;
- the path it added shows in gold;
- the last two steps are the second phase.

For a dense graph, use the zoom buttons and *fade all but this step*.
Hovering over the text gives the language server's help.

## The language server, and VS Code

`qsys lsp` speaks the Language Server Protocol on standard input and output.
It gives:

- the problems in a treatment (`.q`) or its data (`.dat`) as you type;
- on hover over a rule, the rule as the machine applies it, with `--` filled
  in and the members exchanged under `-INV-`;
- on hover over a parameter or a word of the language, what it does, with its
  page of C70;
- colours, an outline of phases and rules, and completion of the words.

`editors/vscode` is an extension that bundles it with a grammar. *Run This
Treatment on Its Data* (⌘↵) runs a `.q` file on the `.dat` of the same name.
See `editors/README.md`.
