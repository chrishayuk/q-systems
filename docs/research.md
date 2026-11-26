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

## QSYS-E10. The grammar of section 11 beyond one embedded clause

E03 tried every sentence with at most one embedded clause. With two there
are about 198 million trees, too many to try, so E10 follows three shapes of
embedding as deep as they go cheaply:

- clauses in the object, `X constate que Y détruit … que …`;
- clauses in the subject;
- clauses in both, a full binary tree of clauses.

Each tree was synthesised, and its sentences were analysed back. Where a
tree has more than 2,000 sentences, 500 random walks through the graph were
analysed instead.

| shape | depth | synthesis applications | arrows kept | sentences | analysed back to the one tree |
|---|---:|---:|---:|---:|---|
| object | 0 | 12 | 9 | 2 | all |
| object | 2 | 156 | 131 | 18 | all |
| object | 5 | 4,368 | 3,719 | 486 | all |
| subject | 5 | 4,368 | 4,042 | 486 | all |
| both | 2 | 516 | 456 | 1,458 | all |
| both | 3 | 3,108 | 2,760 | 9,565,938 | 500 sampled |

- **Synthesis grows with the tree, not with the sentences.** Each clause
  multiplies the sentences by three in a chain (2·3ᵈ). With clauses on both
  sides it multiplies them far faster. At depth 3, seven clauses give
  9,565,938 sentences in 2,760 arrows, made in 3,108 applications. This is
  E08's fanning out, at work in C70's own grammar.
- **Analysis grows with the sentence.** The longest analysis took 196
  applications.
- **No sentence tried was ambiguous.** Every one analysed to exactly its
  tree. This is still a sample beyond depth 1, not a proof.

## QSYS-E11 and E12. Merging equal arrows (an experimental extension, not historical)

E08 found that the machine never merges two arrows with the same ends and
the same tree. E11 asks what happens if it does.

- `src/experimental.rs` is a second machine. With C70's settings it gives
  the historical machine's graphs and application counts on every example,
  and a test checks this.
- `Merge::Intern` reuses an equal arrow, or an equal path the phase has
  already made, instead of adding a new one.
- `Merge::Keep` (E12) does the same, but records every application that
  would have made each arrow. The number of derivations, which is what C70
  would have made copies of, can then still be counted.

| system | machine | applications | arrows kept | distinct strings | derivations of the kept arrows |
|---|---|---:|---:|---:|---:|
| parity, 12 choices | C70 | 8,190 | 4,096 | 2 | 4,096 |
| | merged | 46 | 2 | 2 | 2 |
| | merged, history kept | 46 | 2 | 2 | 4,096 |
| packing, 12 choices | all three | 8,190 | 4,096 | 4,096 | 4,096 |
| `R(A*,B*) + R(B*,C*) == R(A*,C*).` on 7 facts in a row | C70 | 294 | 132 | 1 | 132 |
| | merged | 56 | 1 | 1 | 1 |
| | merged, history kept | 56 | 1 | 1 | 132 |

- **Merging removes the cost of fanning in.**
  - Parity becomes linear: 46 applications instead of 8,190.
  - Packing does not change, because its results really are all different.
  - Composing a relation along seven facts gives one fact `R(A,H)`. C70's
    machine makes it 132 times, once for each way of bracketing the chain
    (the Catalan number C₆).
- **The history can be kept without the copies.** `Merge::Keep` counts the
  4,096 and the 132 from one record per application, 46 and 56 of them.
  This is a shared derivation forest, like the packed parse forests of
  later chart parsers.
- **On these systems merging changes no set of strings.** On every C70
  example it prints exactly what C70 prints, because those examples never
  make two equal arrows (E08).
- **Merging is not harmless in general.**
  - Under C70, `A == A.` never converges (E09).
  - Merged, its one application makes nothing new: it finds the data's own
    arrow, and the second phase removes that arrow as rewritten. The graph
    ends empty.
  - So merging changes the meaning of a system whose rule gives back an
    arrow equal to one it read.
  - Apart from rules like this one, the differences seen were all in
    counts: how many copies of a string the graph holds, and how many
    applications `-DET-` would list.

## QSYS-E13. Purifying by strings instead of arrows (an experimental extension, not historical)

E09 found that C70's second phase removes strings no rule touched, because
it removes arrows. `Purify::Paths` instead keeps every string from the entry
to the exit that does not run through a path some rule was applied to.

To keep one string and drop another that shares an arrow with it, the graph
must be unshared. A vertex of the result is a vertex of the old graph
together with the last arrows crossed to reach it.

| data | C70 | by strings |
|---|---|---|
| `A + B == C.` on 10 choices | 2 strings in 11 arrows | 1,024 strings in 69 arrows |
| C70 p. 29, `A A A B B B C C C` | 1 string, `S(1,1,1)` | 8 strings |
| C70 pp. 30-31, the cul-de-sac | nothing | `/CUL + /DESAC` and `/CULDE + /SAC` |
| C70 p. 38, the analysis | 1 string, the tree | 72 strings |

- **On the choices it loses nothing.** Every string comes through, with each
  `A + B` rewritten as `C`, so there are again 2ⁿ. The cost is the unsharing:
  69 arrows where C70 keeps 11.
- **On C70's own systems it breaks them.**
  - The analysis keeps 72 strings: the tree, and every partial analysis
    that no rule took further.
  - The cul-de-sac recipe stops working. It relies on removing every arrow
    of the dead-end paths it builds.

So the loss of E09 is not a defect C70 overlooked. The second phase is what
makes an analysis return only complete analyses, and C70 uses it on purpose
in the cul-de-sac. Losing the bystanders is the price of that.

## QSYS-E14. Phases in sequence

- **Phases do not commute.** On `A + B`, the rules `A == B.` and
  `A + B == D.` give:
  - `D` in one phase;
  - `B + B` as two phases in that order, because the first phase rewrites
    `A` away before the second can use it;
  - `D` as two phases in the other order.

  Within a phase the order of the rules does not matter (E06), but the
  division into phases does.
- **A pair of phases can keep the strings and lose the sharing.** C70's
  phases 3 and 4 give back exactly the 18 strings of phase 2. But phase 2
  holds them in 139 arrows, and phase 4 in 282. Packing each sentence into
  one tree and unpacking it is the identity on strings, and it undoes the
  compression of E04.

## QSYS-E15. Other computations in the language as it is

These are written in C70's language and run by the historical machine.
Their programs are in `research/e15-*.q`.

| computation | how | rules |
|---|---|---:|
| unary addition, equality | lists: `N(U*) + PLUS + N(V*) == N(U*,V*).`; a repeated parameter tests equality | 1 each |
| unary multiplication | an accumulator, adding the first number once for each `I` of the second | 2 |
| adding one in binary | the carry is a word `Z` moving right | 4 |
| reversing a list | an accumulator | 1 |
| one step of the cellular automaton rule 110 | a window of three cells moved left to right, into an accumulator | 9 |
| composing a relation | facts side by side in a string | 1 |

- **Lists and trees make data easy.** Addition, equality and concatenation
  are single rules, because list parameters match any list.
- **Repetition is done by scanning.** A computation whose steps depend on
  each other is written as a left-to-right scan with an accumulator.
  - A rule reads its whole left member and replaces it. It cannot read a
    neighbour as context and leave it.
  - So the overlapping windows of a cellular automaton cannot all be
    rewritten at once into a new row.
- **Facts are joined only when they are next to each other.**
  - `R(B,C) + R(A,B)` is never composed, because the facts are in the wrong
    order.
  - A rule that lets facts change places joins them, but it never
    converges.
  - Two facts held as alternatives, on parallel arrows, are never joined
    either: no path runs through both. This is E08's limit again: a rule
    sees one path, never two alternatives.
- **Derivations are counted, not results.** `R(A,B) + R(B,C) + R(C,D)`
  gives `R(A,D)` twice, once for each bracketing.
