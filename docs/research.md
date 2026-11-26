# What reconstructing the Q-systems found

Each experiment has a test that asserts its numbers:

- `tests/historical.rs` for E01 and E02;
- `tests/research.rs` for the rest.

Run `cargo test --test research -- --nocapture` to see them printed.

## QSYS-E01. The tree-to-string rules of p. 30 do not converge

C70 p. 30 says the two rules

```
A*(U*) == [(A*) + ](U*) / U* ≠ -NUL-.
](I*,U*) == I* + ](U*).
```

"produiraient" `[(A) + B + [(C) + A + [(E) + F + ] + ] + ]` from
`A(B,C(A,E(F)))`. Applied as C70 section 7 defines, they never stop:

- the arrow `[(A)` they make is itself a tree `A*(U*)`, with `A* = [` and
  `U* = A`;
- so it becomes `[([) + ](A)`, then `[([) + ]([)`, and so on.

`qsys --det --max-arrows 12 examples/structure.q examples/structure.dat`
shows the first seven applications.

With one condition the paper does not have, `-ET- A* -HORS- [,]`
(`research/e01-structure-guarded.q`), the rules converge to exactly the
string printed on p. 30. The inverse pair converges as written, because its
condition `[,] -HORS- I*` already keeps `[` and `]` out.

The rules work in only one direction as printed. That bears on the paper's
claim that a system and its inverse do the same job both ways.

## QSYS-E02. Reversibility holds for the paper's own example

All eighteen sentences the synthesis produces (pp. 40-41) were put back
through the analysis phase, the same system under `-INV-`. Every one gives
the single normalised tree of p. 38:

```
-01- PI(V(CONSTATE),SN(RICHARD),PH(V(ETONNE),PH(V(DETRUIT),SN(BENOIT),SN(BEAUCEVILLE)),SN(GAETANE))) + . -02-
```

`research/e02-round-trip.q` and `.dat` run the eighteen as eighteen graphs
of one data file.

## QSYS-E03. The grammar of section 11 is not ambiguous, as far as it was tried

The grammar of p. 35 is recursive, so it allows infinitely many sentences
("un ensemble restreint mais cependant infini"). Every normalised sentence
with at most one embedded clause was synthesised:

- there are 3 verbs and 4 names;
- each argument of the main clause is a name or a clause whose arguments are
  names;
- that makes 8,112 trees;
- they give 131,424 distinct sentences.

No sentence comes from two trees. Every one of the 131,424, analysed by the
inverted system, gives back its one tree. This takes about a minute in
release:

```sh
cargo test --release --test research -- --ignored e03
```

The report's own example has two embedded clauses, so it is not among them.
Its 18 paraphrases analyse back too (E02). The many groupings C70's
introduction speaks of exist only during the first phase, as partial
analyses that the second phase throws away (E04).

## QSYS-E04. How much each phase grows and prunes the graph

The treatment of section 11, on the sentence of p. 38:

| phase | arrows in | applications | arrows after phase one | removed as rewritten | removed as stranded | arrows left | strings |
|---|---:|---:|---:|---:|---:|---:|---:|
| 1 analysis (`-INV-`) | 12 | 25 | 50 | 35 | 13 | 2 | 1 |
| 2 synthesis | 2 | 156 | 330 | 191 | 0 | 139 | 18 |
| 3 pack each sentence | 139 | 224 | 363 | 345 | 0 | 18 | 18 |
| 4 unpack | 18 | 264 | 546 | 264 | 0 | 282 | 18 |

- **Analysis** makes 38 arrows and keeps 1 of them.
  - 13 are dead ends: partial groupings that no rule took further, cut off
    when the arrows around them were rewritten.
  - The `.` arrow comes through untouched.
- **Synthesis** writes the 18 paraphrases into 139 arrows. Written out
  separately they need 282 (phase 4). The sharing halves the size here,
  because each choice is made once for all the sentences that contain it.
- **Packing and unpacking** each sentence (phases 3 and 4) is how C70 gets
  one sentence to a line. It costs more arrows than the synthesis itself.

## QSYS-E05. Two readings at once (an experiment, not historical)

The grammar is of our own, in `research/e05-two-readings.q`. It lets `AVEC LE
TELESCOPE` attach to the verb or to the noun. Analysed, `JEAN VOIT L HOMME
AVEC LE TELESCOPE .` gives one graph with both readings, as two arrows
between the same two vertices:

```
-01- PH(SN(JEAN),SV(VOIT,SN(HOMME),SP(AVEC,TELESCOPE))) -02-
-01- PH(SN(JEAN),SV(VOIT,SN(HOMME,SP(AVEC,TELESCOPE)))) -02-
```

- Nothing chooses between the readings. The result of a phase is every
  analysis, and a later phase can work on all of them at once.
- The same holds for analyses of parts of a sentence, as long as they span
  the same stretch of the string: the graph holds them side by side, sharing
  everything to their left and right.

## QSYS-E06. The order of the rules does not change the result

C70 p. 22 says that if the first phase converges, its result "est
indépendant de l'ordre dans lequel on a appliqué les règles".

- The rules of all four phases of section 11 were put in every rotation,
  forwards and backwards, which is 32 orders. Phases 3 and 4 have one rule
  each, so the orders differ only in phases 1 and 2.
- Every order gave the same four printed graphs.
- This holds because the machine applies every rule to every path,
  whatever the order. It is a property of the definition, not luck.
- The printout is the same because it is sorted (decision 20). The arrows
  are made in different orders.

## QSYS-E07. Removing a string removes its neighbours

C70 pp. 30-31 gives three rules "si l'on veut supprimer d'un graphe de
chaînes tout chemin portant une chaîne donnée", here `LE + FAIT + QUE + LE
+ FAIT + QUE`.

- The second phase removes every arrow of a rewritten path.
- So any other string that shares one of those arrows goes too.
- In `research/e07-shared-arrows.dat` the graph carries both
  `LE + FAIT + QUE + LE + FAIT + QUE` and `LE + FAIT + QUE + BENOIT`. Nothing
  is left of either.

A rule rewrites arrows, not strings. A shared prefix belongs to every string
through it, so it cannot be taken from one and left to the others. The same
holds for every rule, not only these three:

- an arrow that some rule used is gone from every string that ran through
  it;
- that is harmless only when every such string was also rewritten, which is
  the usual case in analysis.

## QSYS-E08. Working on many strings at once costs what the rules look at, not what they find

*An experiment, not historical.* The data are `n` independent choices
between `A` and `B`. That is n + 1 vertices and 2n arrows carrying 2ⁿ
strings, so forty choices hold 1,099,511,627,776 strings in 80 arrows. The
question is whether a phase can work on all of them without writing them
out.

| system | n | applications | arrows kept | strings after |
|---|---:|---:|---:|---:|
| `A == X. B == Y.` (each arrow on its own) | 40 | 80 | 80 | 2⁴⁰ |
| `A == X. A == Y.` on one string of n `A` (fanning out) | 40 | 80 | 80 | 2⁴⁰ |
| `A + B == C.` (pairs) | 40 | 39 | 41 | 2 |
| parity of the number of `A`, carried left to right from `S` | 12 | 8,190 | 4,096 | 4,096, only 2 different |
| the whole string packed into one tree `P(…)`, left to right | 12 | 8,190 | 4,096 | 4,096, all different |

- **Local rules work on the shared graph as it stands.** A rule whose left
  side spans a bounded stretch applies once per path of that length. On
  these graphs that is a number in proportion to n, however many strings the
  graph carries. Forty choices take 80 applications.
- **So does fanning out.** One string with two rewritings of every word
  gives 2ⁿ strings in 2n arrows. This is the sharing C70 speaks of, and the
  sharing of the synthesis in E04 (139 arrows for 282).
- **Fanning in does not.** A result carried along the string, like the
  parity of the `A`s, is made once for every prefix: 2ⁿ⁺¹ − 2 applications.
  This is exactly what packing the whole string costs, though parity has
  only two possible results and packing has 2ⁿ. At n = 12, 8,166 of the
  8,190 arrows that parity makes repeat an arrow already made, with the same
  ends and the same tree.
- **This follows from the definition.**
  - C70 p. 22 says each application adds "le tout nouveau chemin", so equal
    arrows are never merged.
  - No rule can merge them either. A left member matches one path, and two
    parallel arrows lie on different paths, so no rule ever sees both.
  - The machine's cost is therefore the number of distinct derivations, not
    the number of distinct results. The graph shares what is the same
    *before* a rule. It cannot share what becomes the same *after* one.
- **C70's own examples never meet the question.** None of them ever makes
  two equal arrows (the test checks every phase of every example). So the
  printed results cannot say whether the lost program merged them. The text
  does not suggest it did.
- `A + B == C.` keeps only 2 of the 2⁴⁰ strings. That is a loss, not a
  saving, and E09 looks at it.

## QSYS-E09. What the second phase loses

*An experiment, not historical.* E07 showed that removing one string takes
its neighbours with it. This experiment measures the effect, finds what
avoids it, and asks which phases can be undone.

**A rule removes strings it never matched.** `A + B == C.` was applied to
`n` choices of `A` or `B`:

| n | strings | strings with no `A + B` | strings after | of those with no `A + B`, kept |
|---:|---:|---:|---:|---:|
| 2 | 4 | 3 | 2 | 1 |
| 3 | 8 | 4 | 2 | 0 |
| 4 | 16 | 5 | 2 | 0 |
| 10 | 1,024 | 11 | 2 | 0 |

From n = 3 on, nothing the rule did not apply to is left. `A + A + A` has no
`A + B`, but its first arrow is also the first arrow of `A + B + A`, so it
goes. Every arrow but the first `B` and the last `A` is on some path the rule
was applied to. The two strings left are made of `C`s, with that `B` and that
`A` at the ends.

**Copying every arrow keeps them.** With `A == X.` and `B == Y.` added, every
original string comes back, relabelled, beside every string with `C` in it:
5, 12, 29, 169, 985 and 5,741 strings for the n above. These are the Pell
numbers, since each step is `X`, `Y`, or a `C` covering two. The copy must go
to a new label. `A == A.` never converges, because the arrow it makes is a
new path it applies to.

So a phase keeps every string, in some form, when every label a rule reads
also has a rule of its own that copies it to a new label.

**Which phases undo.** Each system was run on all 64 strings of six choices,
then run again under `-INV-` on the result:

| system | strings that come back exactly | what the inverse gives |
|---|---:|---|
| `A == X. B == Y.` | 64 of 64 | the one string |
| `A == X. B == X.` | 0 | all 64 strings, for every input |
| parity (E08) | 0 | nothing: it never converges |

- **A one-to-one system undoes exactly**, as the grammar of C70 section 11
  does on the strings tried (E02, E03).
- **A two-to-one system undoes to every string with the same image.** The
  inverse is not wrong, since every one of the 64 does give `X + … + X`. But
  it keeps nothing of which string was there.
- **A system that forgets as it goes has an inverse that does not stop.**
  Inverted, `E + B == E.` is `E == E + B.`, which applies to its own result
  without end.

Reversal under `-INV-` is exact only for systems that keep everything they
read. C70's claim for its own grammar holds (E02, E03), but it is a property
of that grammar, not of `-INV-`.
