# 4. Parameters and conditions

## Parameters

A parameter is a letter, a `*`, and perhaps a digit (C70 pp. 11-13). The
letter says what it stands for:

| letters | stands for | example |
|---|---|---|
| `A*` to `F*` | a label | `A*(U*)`, `$$A*` |
| `I*` to `N*` | a tree | `SN(I*)` |
| `U*` to `Z*` | a list of trees, perhaps empty | `PH(U*)`, `](I*,U*)` |

Every occurrence of a parameter in a rule stands for the same thing. A label
parameter standing as a tree is the tree `A*(-NUL-)`, so it matches leaves
only:

```q
A* + A* == TWO(A*).
```

```data
-0- X + X + Y + Y(Z) -1-
```

```output
DONNEES

-01- X + X + Y + Y(Z) -02-

PHASE 1

-01- TWO(X) + Y + Y(Z) -02-
```

`X + X` matched. `Y + Y(Z)` did not, because `Y(Z)` is not a leaf.

## Every substitution is a rule of its own

C70 defines a rule with parameters as the infinity of rules got by
substituting for them. Two substitutions are two rules, "même si elles sont
représentées par la même suite de caractères" (p. 26). So a list split two
ways gives two paths:

```q --det
L(U*,V*) == U(U*) + V(V*).
```

```data
-0- L(A,B) -1-
```

```output
DONNEES

-01- L(A,B) -02-

PHASE 1

    1. rule 1 (line 1) L(U*,V*) == U(U*) + V(V*)
       on   L(A,B)
       with U* = -NUL-, V* = A,B
       adds U + V(A,B)
    2. rule 1 (line 1) L(U*,V*) == U(U*) + V(V*)
       on   L(A,B)
       with U* = A, V* = B
       adds U(A) + V(B)
    3. rule 1 (line 1) L(U*,V*) == U(U*) + V(V*)
       on   L(A,B)
       with U* = A,B, V* = -NUL-
       adds U(A,B) + V
  3 applications; 6 arrows added; 1 removed as rewritten, 0 as off every path from entry to exit

-01- U + V(A,B) -02-
-01- U(A) + V(B) -02-
-01- U(A,B) + V -02-
```

Three substitutions, three new paths: the list `A,B` can be cut before `A`,
between, or after `B`.

## Taking labels apart

`$$` with a label parameter takes a label apart into its characters, or puts
it together again (C70 p. 29):

```q
A* == MOT($$A*).
-REQ-
MOT($$A*) == A*.
```

```data
-0- VERCINGETORIX -1-
```

```output
DONNEES

-01- VERCINGETORIX -02-

PHASE 1

-01- MOT(V,E,R,C,I,N,G,E,T,O,R,I,X) -02-

PHASE 2

-01- VERCINGETORIX -02-
```

The `-REQ-` splits the treatment into two phases. Chapter 7 is about
treatments.

## Conditions

A rule may end with `/` and a condition. The rule then applies only under the
substitutions that make the condition true (C70 pp. 25-27):

- **The relations, between two lists:**
  - `=` and `≠` (`#` may be typed for `≠`);
  - `l1 -DANS- l2`, every tree of l1 is found in l2, with its own list
    -DANS- the other's;
  - `l1 -HORS- l2`, every tree of l1 differs from every tree of l2.
- **The connectives:** `-NON-`, `-ET-` and `-OU-`, binding in that order,
  after the relations.
- **Grouping:** `(.` and `.)`.

```q
A* == NOM(A*) / A* -DANS- PAUL,MARIE.
A* == VERBE(A*) / A* -DANS- VOIT,AIME -ET- -NON- A* = DORT.
```

```data
-0- PAUL + VOIT + MARIE -1-
```

```output
DONNEES

-01- PAUL + VOIT + MARIE -02-

PHASE 1

-01- NOM(PAUL) + VERBE(VOIT) + NOM(MARIE) -02-
```

`-HORS-` is not the negation of `-DANS-` (C70 p. 14):

- both are false when either side is `-NUL-`;
- `A -DANS- A(B)` is true, because a leaf's empty list is in any list;
- `A -HORS- A(B)` is false.

The seven examples of p. 15 are among the acceptance tests.

## The rule before: `--`

`--` stands for the same part of the rule before (C70 p. 26):

- in a left member, its left member;
- in a right member, its right member;
- in a condition, its condition, in `(. .)`.

So two rules with one left member are written:

```q --det
PI(V(A*),I*,J*) == SUJ + I* + V(A*) + OBJ + J*.
             -- == SUJ + J* + EST + V(A*) + PPA + I*.
```

```data
-0- PI(V(VOIT),SN(PAUL),SN(MARIE)) -1-
```

```output
DONNEES

-01- PI(V(VOIT),SN(PAUL),SN(MARIE)) -02-

PHASE 1

    1. rule 1 (line 1) PI(V(A*),I*,J*) == SUJ + I* + V(A*) + OBJ + J*
       on   PI(V(VOIT),SN(PAUL),SN(MARIE))
       with A* = VOIT, I* = SN(PAUL), J* = SN(MARIE)
       adds SUJ + SN(PAUL) + V(VOIT) + OBJ + SN(MARIE)
    2. rule 2 (line 2) PI(V(A*),I*,J*) == SUJ + J* + EST + V(A*) + PPA + I*
       on   PI(V(VOIT),SN(PAUL),SN(MARIE))
       with A* = VOIT, I* = SN(PAUL), J* = SN(MARIE)
       adds SUJ + SN(MARIE) + EST + V(VOIT) + PPA + SN(PAUL)
  2 applications; 11 arrows added; 1 removed as rewritten, 0 as off every path from entry to exit

-01- SUJ + SN(MARIE) + EST + V(VOIT) + PPA + SN(PAUL) -02-
-01- SUJ + SN(PAUL) + V(VOIT) + OBJ + SN(MARIE) -02-
```

That is C70's own active and passive (p. 37).

## Comments

A comment is `**`, then any characters but `.` and `=`, then `.`. It ends at
its first full stop, so a comment cannot contain one. `** EXEMPLE DE
SYSTEME-Q.` is a comment; `** P. 27.` is the comment `** P` and then text
that does not read.
