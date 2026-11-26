# q-systems

The Q-systems, the rule language Alain Colmerauer designed at the
Université de Montréal for the TAUM machine-translation project, working
from October 1969. This repository reconstructs it from his own report of
September 1970.

A Q-system does not transform a string. It transforms a **graph of
strings**: a set of arrows, each carrying a tree, between an entry and an
exit. Every path from the entry to the exit is one string, so one graph holds
many alternatives, with their common parts shared.

A rule `a1 + … + ap == b1 + … + bq.` applies in two phases:

- **Adding.** For every path whose trees match `a1 + … + ap`, a new path
  carrying `b1 + … + bq` is added beside it. Nothing is replaced. This goes
  on until no rule applies to a path it has not been applied to. The paths
  include the new ones.
- **Removing.** Every arrow some rule was applied to is removed, and then
  every arrow left off the way from entry to exit.

Under `-INV-` the same rules run backwards.

```
$ qsys examples/paraphrase.q examples/paraphrase.dat
DONNEES

-01- RICHARD + CONSTATE + QUE + LA + DESTRUCTION + DE + BEAUCEVILLE + PAR + BENOIT + ETONNE + GAETANE + . -02-

PHASE 1

-01- PI(V(CONSTATE),SN(RICHARD),PH(V(ETONNE),PH(V(DETRUIT),SN(BENOIT),SN(BEAUCEVILLE)),SN(GAETANE))) + . -02-

PHASE 2

-01- L + ETONNEMENT + DE + GAETANE -02-
-02- PAR + LA + DESTRUCTION + DE + BEAUCEVILLE + PAR + BENOIT -03-
-02- PAR + LE + FAIT + QUE -04-
-04- BEAUCEVILLE + EST + DETRUIT + PAR + BENOIT -03-
-04- BENOIT + DETRUIT + BEAUCEVILLE -03-
-01- LE + FAIT + QUE -05-
…
-10- . -19-
```

The two phases use one system of 16 rules:

- **Phase 1**, the system inverted, analyses the sentence into its
  normalised tree.
- **Phase 2**, the same system as written, synthesises every paraphrase of
  it: active or passive, each embedded clause as a clause or as a noun.
  There are 18 sentences, in one graph of 19 vertices.

Phase 2 is Colmerauer's printout, C70 pp. 38-40, **line for line, with his
vertex numbers**.

## Status

- **The source.** No program text survives. The reconstruction rests on
  Colmerauer's report, *Les systèmes-Q ou un formalisme pour analyser et
  synthétiser des phrases sur ordinateur* (C70). It was read in full from
  the page scans. It gives:
  - the whole grammar;
  - the meaning of every construct;
  - worked examples with their printed results.

  One page of the scan (printed p. 23) is missing. A 1972 TAUM printout
  shows the output format. See [`sources/README.md`](sources/README.md).
- **Every rule of the machine is listed with its evidence** in
  [`docs/decisions.md`](docs/decisions.md). Most are sources. The decisions
  are about reading text and the order of printing.
- **Every computation C70 gives a result for comes out as it says**, with
  one exception (below). There are 21 acceptance tests, each naming its page
  ([`tests/historical.rs`](tests/historical.rs)). They cover:
  - labels;
  - the ways of writing a tree;
  - the seven examples of `-DANS-` and `-HORS-`;
  - the AAABBBCCC system, arrow by arrow through its first phase;
  - its inverse;
  - `$$`;
  - structured strings;
  - the cul-de-sac;
  - the four phases of the paraphrase treatment.
- **The exception is the order of the last printout.** The 18 sentences of
  pp. 40-41 come out in exactly the reverse of the paper's order. No one
  rule for ordering fits both pp. 38-40 and pp. 40-41 (decision 20).
- **Not seen.** The TAUM grammars of *TAUM 71*, the only large Q-system
  programs, and any later version of the formalism.
- **Not yet checked.** Nothing has been checked by a person against the
  sources. No literature search has been made, so nothing here is claimed as
  new.

## What reconstructing it found

- **The tree-to-string rules of p. 30 do not converge as printed.**
  - The `[(A)` they make is itself a tree that the first rule matches
    again, without end.
  - One extra condition, `A* -HORS- [,]`, makes them produce exactly the
    printed string.
  - The inverse rules work as printed. (QSYS-E01)
- **Reversibility holds for the paper's example.** All 18 synthesised
  sentences analyse back to the one tree. (QSYS-E02)
- **The paper's French grammar is unambiguous.** 8,112 normalised trees give
  131,424 distinct sentences, and no sentence comes from two trees. (E03)
- **The graph shares.**
  - Analysis builds 38 arrows of partial groupings and keeps 1.
  - Synthesis writes 18 sentences in 139 arrows, where written out
    separately they take 282. (E04)
- **Ambiguity costs nothing extra.** With a grammar of our own, both
  readings of `JEAN VOIT L HOMME AVEC LE TELESCOPE` come out of one analysis
  as two parallel arrows. Nothing chooses. (E05)
- **The result does not depend on the order of the rules.** 32 orders gave
  one printout, as C70 p. 22 claims. (E06)
- **Rules rewrite arrows, not strings.**
  - The p. 30 recipe for removing one string from a graph also removes every
    string sharing an arrow with it.
  - So `LE + FAIT + QUE + BENOIT` goes with `LE + FAIT + QUE + LE + FAIT +
    QUE`. (E07)

The experiments, with their numbers, are in
[`docs/research.md`](docs/research.md).

## Install and use

You need [Rust](https://rustup.rs). There are no dependencies.

```bash
cargo install --path .
qsys examples/abc.q examples/abc.dat          # the data graph, then the graph after each phase
qsys --det examples/abc.q examples/abc.dat    # every rule application, as -DET- does for a phase
qsys --width 72 PROGRAM DATA                  # fold lines as the line printer did
qsys check examples/paraphrase.q              # the rules as applied: -- filled in, -INV- done
cargo test                                    # the acceptance tests, the examples, the experiments
```

## A system

C70 p. 27:

```
** EXEMPLE DE SYSTEME-Q.
A* + A*(U*) == A*(1,U*) / A* -DANS- A,B,C.
A(U*) + B(U*) + C(U*) == S(1,U*).
```

Applied to `-0- A + A + A + B + B + B + C + C + C -1-` it leaves
`-01- S(1,1,1) -02-`. It leaves an `S` only when there are as many A as B as
C. Inverted with `-INV-`, it takes `S(1,1,1)` back to the string.

The parameters are:

- `A*` to `F*`: a label;
- `I*` to `N*`: a tree;
- `U*` to `Z*`: a list, perhaps empty.

Each may take a digit, as in `A*1`.

The conditions are built from:

- `=`, `≠` (or `#`), `-DANS-` and `-HORS-`, between lists;
- `-NON-`, `-ET-` and `-OU-`;
- `(.` and `.)` for grouping.

Other notation:

- `--` repeats the matching part of the rule before.
- `**` begins a comment, which ends at the next full stop.
- `-REQ-` separates the phases of a treatment.
- `-INV-` inverts a phase.
- `-DET-` prints its details.

## Layout

| | |
|---|---|
| `src/tree.rs` | trees, characters, `-DANS-` and `-HORS-` |
| `src/syntax.rs` | reading treatments and graphs (C70 appendix) |
| `src/matching.rs` | parameters, substitution, conditions |
| `src/machine.rs` | graphs of strings and the two phases |
| `src/print.rs` | the `-01- … -02-` printout |
| `examples/` | every example of C70, with its `.out` |
| `research/` | the inputs of the experiments |
| `tests/c70/` | C70's printed results, transcribed from the scan |

## Licence

Apache-2.0. The report itself is Colmerauer's and is not included.
