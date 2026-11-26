# 7. Treatments: phases one after another

A treatment is a series of phases separated by `-REQ-`. Each phase is a
system, perhaps marked `-INV-`. The first phase is applied to the data, and
each phase after it to what the phase before left (C70 section 10, p. 34).
`qsys` prints the graph after every phase, as the CDC did:

> Lors de l'exécution effective d'un traitement-q l'ordinateur imprime le ou
> les graphes qui lui sont fournis en données et les graphes obtenus après
> l'application de chaque phase. (C70 p. 34)

In English: when a Q-treatment is run, the computer prints the graph or
graphs it is given as data, and the graphs obtained after each phase.

A phase marked `-DET-` also prints every application of every rule. `--det`
does the same for every phase. The words come in the order `-DET-`, then
`-INV-`, then the system, and a comment may not come before them.

## Analysis and synthesis with one system

C70's last section (pp. 35-42) is one system of 16 rules used twice:

- inverted, it analyses a French sentence into a normalised tree;
- as written, it synthesises from that tree every paraphrase of the
  sentence:
  - active or passive;
  - each embedded clause either as a clause (*le fait que…*) or as a noun
    (*la destruction de…*).

The whole treatment is `examples/paraphrase.q`. Its first two phases, on a
shorter sentence than C70's, look like this:

```q
-INV- ** PHASE D ANALYSE.
PI(V(A*),I*,J*) == SUJ + I* + V(A*) + OBJ + J*.
             -- == SUJ + J* + EST + V(A*) + PPA + I*.
PH(V(A*),I*,J*) == SN(V(A*)) + DDE + J* + PPA + I*.
SUJ + PH(U*) == LE + FAIT + QUE + PI(U*).
OBJ + PH(U*) == QUE + PI(U*).
PPA + PH(U*) == PAR + LE + FAIT + QUE + PI(U*).
DDE + PH(U*) == DE + CE + QUE + PI(U*).
SUJ + SN(I*) == N(I*).
OBJ + SN(I*) == N(I*).
PPA + SN(I*) == PAR + N(I*).
DDE + SN(I*) == DE + N(I*).
N(A*) == A* / A* -DANS- BEAUCEVILLE,BENOIT,GAETANE,RICHARD.
N(V(DETRUIT)) == LA + DESTRUCTION.
N(V(ETONNE)) == L + ETONNEMENT.
N(V(CONSTATE)) == LA + CONSTATATION.
V(A*) == A* / A* -DANS- DETRUIT,ETONNE,CONSTATE.
-REQ- ** PHASE DE SYNTHESE.
PI(V(A*),I*,J*) == SUJ + I* + V(A*) + OBJ + J*.
             -- == SUJ + J* + EST + V(A*) + PPA + I*.
PH(V(A*),I*,J*) == SN(V(A*)) + DDE + J* + PPA + I*.
SUJ + PH(U*) == LE + FAIT + QUE + PI(U*).
OBJ + PH(U*) == QUE + PI(U*).
PPA + PH(U*) == PAR + LE + FAIT + QUE + PI(U*).
DDE + PH(U*) == DE + CE + QUE + PI(U*).
SUJ + SN(I*) == N(I*).
OBJ + SN(I*) == N(I*).
PPA + SN(I*) == PAR + N(I*).
DDE + SN(I*) == DE + N(I*).
N(A*) == A* / A* -DANS- BEAUCEVILLE,BENOIT,GAETANE,RICHARD.
N(V(DETRUIT)) == LA + DESTRUCTION.
N(V(ETONNE)) == L + ETONNEMENT.
N(V(CONSTATE)) == LA + CONSTATATION.
V(A*) == A* / A* -DANS- DETRUIT,ETONNE,CONSTATE.
```

```data
-0- LA + DESTRUCTION + DE + BEAUCEVILLE + PAR + BENOIT + ETONNE + GAETANE + . -1-
```

```output
DONNEES

-01- LA + DESTRUCTION + DE + BEAUCEVILLE + PAR + BENOIT + ETONNE + GAETANE + . -02-

PHASE 1

-01- PI(V(ETONNE),PH(V(DETRUIT),SN(BENOIT),SN(BEAUCEVILLE)),SN(GAETANE)) + . -02-

PHASE 2

-01- GAETANE + EST + ETONNE -02-
-02- PAR + LA + DESTRUCTION + DE + BEAUCEVILLE + PAR + BENOIT -03-
-02- PAR + LE + FAIT + QUE -04-
-04- BEAUCEVILLE + EST + DETRUIT + PAR + BENOIT -03-
-04- BENOIT + DETRUIT + BEAUCEVILLE -03-
-01- LA + DESTRUCTION + DE + BEAUCEVILLE + PAR + BENOIT -05-
-01- LE + FAIT + QUE -06-
-06- BEAUCEVILLE + EST + DETRUIT + PAR + BENOIT -05-
-06- BENOIT + DETRUIT + BEAUCEVILLE -05-
-05- ETONNE + GAETANE -03-
-03- . -07-
```

Phase 1 found the one tree. Phase 2 wrote out all six paraphrases:

- the destruction as a noun, or as a clause in the active or the passive;
- each of those in the active or the passive of *étonner*.

Each choice is made once, at its own vertex:

- from `-01-`, the two voices of *étonner* part;
- inside each voice, the three ways of saying *Benoît détruit Beauceville*
  part and meet again (`-02-` to `-03-`, and `-01-` to `-05-`).

The three are written twice, not once, because the passive puts them after
*par* and the active before *étonne*, and those are different words.

On C70's own sentence the same two phases give:

- the tree of p. 38;
- a graph of 19 vertices carrying 18 paraphrases, which is the graph printed
  on pp. 38-40, line for line.

Every one of the 18, put back through the first phase, analyses to the one
tree (QSYS-E02).

## One sentence to a line

To print each sentence on its own, C70 adds two phases of one rule each:

- the first packs every path into one tree, `.(RICHARD,CONSTATE,…)`;
- the second unpacks each tree into a path of its own.

```q
A* + .(U*) == .(A*,U*).
-REQ-
.(A*,U*) == A* + .(U*).
```

```data
-0- PAUL + DORT + . -1-
-0- MARIE + DORT + . -1-
```

```output
DONNEES

-01- MARIE + DORT + . -02-
-01- PAUL + DORT + . -02-

PHASE 1

-01- .(MARIE,DORT) -02-
-01- .(PAUL,DORT) -02-

PHASE 2

-01- MARIE + DORT + . -02-
-01- PAUL + DORT + . -02-
```

The `.` that ends the sentence is a label like any other. After `.(A*,U*)`
comes a second `.`, the full stop that ends the rule.
