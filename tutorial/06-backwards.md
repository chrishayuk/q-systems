# 6. Running a system backwards

The two members of a rule are written the same way: strings of trees with
parameters. So the members can be exchanged:

> Une des originalités des systèmes-q est leur aspect réversible. Ceci veut
> dire que pratiquement le même système-q peut être utilisé pour décrire une
> transformation et la transformation inverse. (C70 p. 4)

In English: one of the original features of Q-systems is that they can be
reversed. Practically the same Q-system can describe a transformation and
its inverse.

You need not rewrite a system to invert it. Put `-INV-` at the head of the
phase, and every rule is applied with its members exchanged (C70 p. 33).
Here is the system of chapter 3, inverted, given its result:

```q
-INV-
A* + A*(U*) == A*(1,U*) / A* -DANS- A,B,C.
A(U*) + B(U*) + C(U*) == S(1,U*).
```

```data
-0- S(1,1,1) -1-
```

```output
DONNEES

-01- S(1,1,1) -02-

PHASE 1

-01- A + A + A + B + B + B + C + C + C -02-
```

## The restriction on parameters

A parameter of the right member or of the condition must be in the left
member (C70 p. 27, "pour des raisons d'efficacité"):

- without it, the right member could not be built;
- under `-INV-` the members are exchanged, so the restriction turns round
  (p. 33).

So `B(A*,I*) == A*.` is a good rule, but under `-INV-` it would have to
build `B(A*,I*)` from `A*` alone, with nothing to put for `I*`. It is
refused:

```q
-INV-
B(A*,I*) == A*.
```

```data
-0- A -1-
```

```output
qsys: line 2: I* appears in the left member or condition of the rule but not in its right member (C70 p. 33)
```

## Not every system runs both ways

On p. 30 C70 gives two pairs of rules:

- one pair takes a tree apart into a "structured string";
- the other puts it back together.

Back together works:

```q
[(A*) + ](U*) == A*(U*).
I* + ](U*) == ](I*,U*) / [,] -HORS- I*.
```

```data
-0- [(A) + B + [(C) + A + [(E) + F + ] + ] + ] -1-
```

```output
DONNEES

-01- [(A) + B + [(C) + A + [(E) + F + ] + ] + ] -02-

PHASE 1

-01- A(B,C(A,E(F))) -02-
```

Taking apart, as printed, does not finish:

```q --max-arrows 2000
A*(U*) == [(A*) + ](U*) / U* ≠ -NUL-.
](I*,U*) == I* + ](U*).
```

```data
-0- A(B,C(A,E(F))) -1-
```

```output
DONNEES

-01- A(B,C(A,E(F))) -02-

PHASE 1

(the phase did not converge within 2000 new arrows: C70 p. 22, "le résultat est indéfini")
```

Why it never stops:

- The `[(A)` it makes is itself a tree `A*(U*)`, with `A*` the label `[` and
  `U*` the list `A`.
- So the first rule applies to its own output, and goes on applying for ever
  (QSYS-E01).
- The rules that put the tree back are guarded by `[,] -HORS- I*`, which keeps
  `[` and `]` out. The rules that take it apart have no such guard.

With one, they give exactly the string C70 prints:

```q
A*(U*) == [(A*) + ](U*) / U* ≠ -NUL- -ET- A* -HORS- [,].
](I*,U*) == I* + ](U*).
```

```data
-0- A(B,C(A,E(F))) -1-
```
```output
DONNEES

-01- A(B,C(A,E(F))) -02-

PHASE 1

-01- [(A) + B + [(C) + A + [(E) + F + ] + ] + ] -02-
```
