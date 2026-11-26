# 2. Labels, trees and graphs

## Labels

A label is a character followed by letters and digits (C70 p. 6). The
characters are those of the CDC 6400:

- the capital letters (there are no small ones);
- the digits;
- `+ - * / ( ) $ = ≠ . ,`;
- `≡ [ ] : → ∨ ∧ ↑ ↓ < > ≤ ≥ ¬ ;`.

So `HIBOU`, `-PL3`, `+`, `(` and `,A` are labels. `DIT-IL` is not, since a
`-` may only come first. Nor is `3.14`.

## Trees

A tree is a label, or a label with a list of trees in parentheses:

    PHRASE(SN(IL),SV(V(MANGE),SN(LA,CHOUCROUTE)))
    +(10,*(-30,45))

The empty list is written `-NUL-`, and `A(-NUL-)` is just `A`. `$$` turns a
label into the list of its characters, so `MOT($$ABC)` is `MOT(A,B,C)`. Both
are only ways of writing (C70 pp. 9-10):

```q
MOT(A,B,C) == SAME.
```

```data
-0- MOT($$ABC) + X(-NUL-) -1-
```

```output
DONNEES

-01- MOT(A,B,C) + X -02-

PHASE 1

-01- SAME + X -02-
```

A graph given as data may not use `-NUL-` or `$$` (C70 p. 20), and `qsys` says
so:

- if `qsys` accepted them in data, a printed graph would no longer read back
  the same;
- rules may use them freely.

## Strings and graphs of strings

A **string** is trees joined by `+`. Here the `+` between trees separates,
and a `+` standing alone as a tree is a label: `A + + + B` is the three trees
`A`, `+` and `B`.

A **graph of strings** is written as paths without crossings, each a vertex,
a string and a vertex (C70 pp. 17-20). This graph carries six sentences:

```q
** NO RULES: THE GRAPH IS PRINTED AS IT IS.
```

```data
-1- OUI -2-
-1- NON -2-
-2- SN(PAUL) -3-
-3- EST + VENU -4-
-3- V(MANGER,3PSII) -4-
-4- HIER -5-
-3- A + JOUE + MARDI -5-
-5- . -6-
```

```output
DONNEES

-01- NON -02-
-01- OUI -02-
-02- SN(PAUL) -03-
-03- A + JOUE + MARDI -04-
-03- EST + VENU -05-
-03- V(MANGER,3PSII) -05-
-05- HIER -04-
-04- . -06-

PHASE 1

-01- NON -02-
-01- OUI -02-
-02- SN(PAUL) -03-
-03- A + JOUE + MARDI -04-
-03- EST + VENU -05-
-03- V(MANGER,3PSII) -05-
-05- HIER -04-
-04- . -06-
```

Two conditions hold for every graph (C70 p. 17):

- there is no circuit;
- there is one entry and one exit, and every arrow is on a path between them.

Every path into a vertex must be written before any path out of it (C70 p.
20). That rule is what rules out circuits.

The vertices are printed renumbered, and in an order of their own (decisions
19 and 20). That order reproduces Colmerauer's printout of p. 38 line for
line.

Several graphs may be given at once, separated by `/`. Each is treated on its
own:

```q
A + B == C.
```

```data
-0- A + B -1-
/
-0- B + A -1-
```

```output
DONNEES

-01- A + B -02-

PHASE 1

-01- C -02-

/

DONNEES

-01- B + A -02-

PHASE 1

-01- B + A -02-
```

The rule matched in the first graph only. In the second nothing applied, so
nothing changed.
