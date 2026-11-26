# 5. Every alternative at once

The graph is what makes the Q-systems more than string rewriting. Nothing a
rule does stops any other rule from applying, so every analysis of a string
is made, and the graph keeps them all. Parts the analyses share are written
once.

## Two readings of one sentence

This grammar is our own, not Colmerauer's (`research/e05-two-readings.q`).
In it, *avec le télescope* can go with the seeing or with the man:

```q
JEAN == SN(JEAN).
L + HOMME == SN(HOMME).
LE + TELESCOPE == SN(TELESCOPE).
AVEC + SN(I*) == SP(AVEC,I*).
SN(I*) + SP(J*,K*) == SN(I*,SP(J*,K*)).
VOIT + SN(U*) == SV(VOIT,SN(U*)).
SV(I*,J*) + SP(K*,L*) == SV(I*,J*,SP(K*,L*)).
SN(U*) + SV(W*) + . == PH(SN(U*),SV(W*)).
```

```data
-0- JEAN + VOIT + L + HOMME + AVEC + LE + TELESCOPE + . -1-
```

```output
DONNEES

-01- JEAN + VOIT + L + HOMME + AVEC + LE + TELESCOPE + . -02-

PHASE 1

-01- PH(SN(JEAN),SV(VOIT,SN(HOMME),SP(AVEC,TELESCOPE))) -02-
-01- PH(SN(JEAN),SV(VOIT,SN(HOMME,SP(AVEC,TELESCOPE)))) -02-
```

Both analyses come out, as two arrows between the same two vertices:

- nothing chose between them;
- a later phase can work on both at once;
- or it can choose, with a rule whose condition only one of them meets.

## Shared parts are shared

The analyses of the parts of a sentence share everything to their left and
right. That sharing is what lets synthesis be cheap too. In C70's own
example (chapter 7), the 18 paraphrases of one sentence are written in one
graph of 139 arrows, where written out one by one they take 282
(`docs/research.md`, QSYS-E04).

## The price: a rule rewrites arrows, not strings

The second phase removes every arrow of every path a rule was applied to.
An arrow belongs to every string that passes along it, so a rule that
rewrites one string takes the arrows out of the others that share them.

C70 pp. 30-31 gives three rules "si l'on veut supprimer d'un graphe de
chaînes tout chemin portant une chaîne donnée", that is, to remove every
path carrying a given string, here `LE + FAIT + QUE + LE + FAIT + QUE`:

```q
LE + FAIT + QUE + LE + FAIT + QUE == /CUL + /DE + /SAC.
/CUL + /DE == /CULDE. /DE + /SAC == /DESAC.
```

```data
-0- LE + FAIT + QUE + LE + FAIT + QUE -1-
```

```output
DONNEES

-01- LE + FAIT + QUE + LE + FAIT + QUE -02-

PHASE 1

(no arrows: the graph is empty)
```

How the three rules get nothing:

- The first rule rewrites the string, so its six arrows go.
- The path it added, `/CUL + /DE + /SAC`, is rewritten in its turn by the
  other two, so it goes too.
- `/CULDE` and `/DESAC` cannot both stand: they overlap, so neither reaches
  from the entry to the exit. They are stranded.

Now give the graph a second string that shares the first three arrows:

```q
LE + FAIT + QUE + LE + FAIT + QUE == /CUL + /DE + /SAC.
/CUL + /DE == /CULDE. /DE + /SAC == /DESAC.
```

```data
-0- LE + FAIT + QUE -1-
-1- LE + FAIT + QUE -2-
-1- BENOIT -2-
```

```output
DONNEES

-01- LE + FAIT + QUE -02-
-02- BENOIT -03-
-02- LE + FAIT + QUE -03-

PHASE 1

(no arrows: the graph is empty)
```

`LE + FAIT + QUE + BENOIT` is gone too (QSYS-E07). Its first three arrows were
on the rewritten path.

In analysis this is what is wanted. Every arrow of the sentence is used by
some rule, and only what reaches across the whole sentence survives. It is a
thing to remember when a phase is meant to filter.
