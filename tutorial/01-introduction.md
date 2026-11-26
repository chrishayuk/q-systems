# 1. A graph of every alternative

Alain Colmerauer designed the Q-systems at the Université de Montréal for
TAUM, the project that translated English into French by machine. The
first program, in ALGOL on the university's CDC 6400, was working by
October 1969. In TAUM an English sentence went through fifteen Q-systems one
after another before it came out in French.

His report of September 1970 opens with the problem they solve:

> Le problème principal est de nature combinatoire: pris isolément chaque
> groupe d'éléments de la phrase peut se combiner de différentes façons avec
> d'autres groupes et en former de nouveaux qui à leur tour peuvent se
> recombiner, et ainsi de suite. (C70 p. 4)

In English: the main problem is combinatorial. Each group of words, taken
alone, can combine in several ways with other groups, and form new groups,
which can combine again, and so on.

His answer is not to choose. A Q-system works on a **graph of strings**:

- arrows carry trees between an entry and an exit;
- every path from the entry to the exit is one string;
- the graph holds every way of grouping the sentence at once, and shares
  what the groupings have in common.

A rule never replaces anything. It **adds** a path beside each path it
matches. Only when no rule can do anything more are the rewritten arrows
taken away.

This tutorial teaches the language as it is reconstructed here, from
Colmerauer's report (cited as C70; see `sources/README.md`). Where the report
is silent and a decision had to be made, the tutorial says so and points to
`docs/decisions.md`.

## Install

You need [Rust](https://rustup.rs). There are no other dependencies.

```sh
cargo install --path .
qsys examples/abc.q examples/abc.dat
```

Or open the [playground](../web/index.html). It runs the same program in the
browser and draws the graph one rule application at a time.

## A first treatment

`qsys` takes two files:

- the **treatment**, which is the rules;
- the **data**, which is the graph they are applied to.

Here they are side by side. The first block is the treatment, the second the
data, the third what `qsys` prints:

```q
A + B == C.
```

```data
-0- A + B -1-
```

```output
DONNEES

-01- A + B -02-

PHASE 1

-01- C -02-
```

The rule reads "A + B *se réécrit en* C". The data is one path, from vertex
`-0-` to vertex `-1-`, carrying the string `A + B`.

`qsys` prints:

- the data;
- the graph after each phase, here one;
- vertices numbered afresh from `-01-`, as the TAUM line printer did.

The path `A + B` matched the rule, so the path `C` was added beside it. Then
`A` and `B` were removed, because a rule had been applied to a path they were
on.

## The order of a text does not matter

There is no program counter. Every rule is tried on every path, including
the paths other rules add, until nothing new can be done:

```q
B + C == D.
A + B == E.
E + C == F.
A + D == F.
```

```data
-0- A + B + C -1-
```

```output
DONNEES

-01- A + B + C -02-

PHASE 1

-01- F -02-
-01- F -02-
```

`A + B + C` can be grouped as `A + (B + C)` or `(A + B) + C`:

- the first two rules each found their path;
- the last two each made `F` from what the first two added.

So there are two arrows `F`, one for each grouping. Every arrow the rules
used was removed, and what is left is the two analyses side by side. Chapter
5 comes back to this.

The next chapter is about what can stand on an arrow.
