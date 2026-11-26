# 3. How a system is applied: add, then remove

A system is applied to a graph in two phases (C70 section 7, pp. 21-22).

**First phase: adding arrows.**

- For every path whose trees match the left member of a rule, a new path
  carrying the right member is added. It runs from the first vertex of the
  matched path to its last, through new vertices.
- Nothing is taken away.
- The paths tried include the new ones, and paths that mix new arrows with
  old.
- No rule is applied twice to the same path. The phase ends when no rule
  can be applied to a path it has not been applied to.

**Second phase: removing arrows.**

- Every arrow on a path that some rule was applied to is removed.
- Then every arrow no longer on a path from the entry to the exit is
  removed.

`--det` prints each application of the first phase:

- the rule;
- the path it matched;
- the substitution, if the rule has parameters (chapter 4);
- the arrows it added.

```q --det
A + A == B.
B + A == C.
```

```data
-0- A + A + A -1-
```

```output
DONNEES

-01- A + A + A -02-

PHASE 1

    1. rule 1 (line 1) A + A == B
       on   A + A
       adds B
    2. rule 1 (line 1) A + A == B
       on   A + A
       adds B
    3. rule 2 (line 2) B + A == C
       on   B + A
       adds C
  3 applications; 3 arrows added; 4 removed as rewritten, 1 as off every path from entry to exit

-01- C -02-
```

The third application is the important one. `B + A` matched a path made of
a new arrow, the first `B`, and an old one, the third `A`.

The second `B`, made from the last two `A`, was used by no rule, but it was
removed anyway:

- the second phase took away the `A` arrows on either side of it;
- that left it on no path from the entry to the exit;
- so it was *stranded*.

## A system of C70 section 8

The example of C70 section 8 (p. 27) accepts `A…A B…B C…C` with as many of each.
It uses parameters, which chapter 4 explains:

- `A* + A*(U*) == A*(1,U*)` counts runs of one letter, so `A + A` becomes
  `A(1)` and `A + A(1)` becomes `A(1,1)`;
- the second rule joins three counts that are equal.

```q
** EXEMPLE DE SYSTEME-Q.
A* + A*(U*) == A*(1,U*) / A* -DANS- A,B,C.
A(U*) + B(U*) + C(U*) == S(1,U*).
```

```data
-0- A + A + A + B + B + B + C + C + C -1-
/
-0- A + A + B + B + B + C + C -1-
```

```output
DONNEES

-01- A + A + A + B + B + B + C + C + C -02-

PHASE 1

-01- S(1,1,1) -02-

/

DONNEES

-01- A + A + B + B + B + C + C -02-

PHASE 1

-01- A(1) + B(1,1) + C(1) -02-
```

On p. 29 Colmerauer draws the graph between the two phases:

- two arrows `A(1)` and one `A(1,1)`;
- the same for `B` and `C`;
- `S(1,1,1)` under them all.

The playground draws the same picture. Open `abc` and move the slider.

The second graph has two A, three B and two C, and gives no `S`. Notice that
it does not come out empty either. A rejected string is not removed: it is
left, half-analysed, as whatever the rules could make of it.

## When the first phase never ends

If the rules keep making paths they can apply to, the first phase never
ends. C70 p. 22 says the result is then undefined. `qsys` stops a phase that
has added more arrows than its limit, which is 200,000 unless `--max-arrows`
says otherwise, and says so (decision 16):

```q --max-arrows 50
A == B(A).
B(I*) == A.
```

```data
-0- A -1-
```
```output
DONNEES

-01- A -02-

PHASE 1

(the phase did not converge within 50 new arrows: C70 p. 22, "le résultat est indéfini")
```
