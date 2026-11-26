# What the reconstruction does, and what each rule rests on

Every rule of the machine is listed here with its evidence. Three kinds of
statement are kept apart:

- **Source**: what C70 (or the TAUM printout, T72) says. The page is given.
  The sources are listed in `sources/README.md`.
- **Decision**: what the reconstruction does where the sources are silent,
  ambiguous or in conflict. The reason is given.
- **Experiment**: something that is not historical. It exists only so the
  rule beside it can be studied.

C70 is much fuller than the sources of the other reconstructions in this
series. It gives the whole grammar (pp. 43-45), the meaning of every
construct, and the printed results of its examples. So most rules here are
sources, and the decisions are mostly about reading text and printing
graphs.

## Characters and reading

1. *Source.* **The characters are those of the CDC 6400** (C70 p. 6):
   - the capital letters;
   - the digits;
   - the significant signs `+ - * / ( ) $ = ≠ . ,`;
   - the insignificant signs `≡ [ ] : → ∨ ∧ ↑ ↓ < > ≤ ≥ ¬ ;`.

   There are no small letters, "car elles n'existent pas sur notre
   ordinateur", and reading one is an error. The set and its order are CDC
   display code; `→` is display code 65, drawn in C70 as a hooked arrow.

2. *Decision.* **`#` may be typed for `≠`.** `≠` is not on an ordinary
   keyboard. `#` is not a CDC character, so it cannot be confused with one.

3. *Decision.* **Blanks and line ends separate, and are otherwise
   ignored.**
   - A label ends at a blank.
   - C70 sets its examples with blanks round `+`, `==` and `/`, and
     sometimes before `(`.
   - Nothing says how cards were read. T72 shows only that output lines were
     folded anywhere, even inside a label.

4. *Source.* **A label is any character followed by letters and digits**
   (C70 p. 6). So `+`, `(`, `,`, `.`, `/`, `[` and `-30` are labels (p. 7
   gives `+`, `(` and `,A` as correct). The grammar is written on characters,
   and the reader decides what a character is by where it stands:
   - Where a tree is expected, any character begins a label, except:
     - a letter A-F, I-N or U-Z followed by `*`, which is a parameter;
     - `-NUL-` and `$$`, which are lists, not trees.
   - After a tree:
     - `+` joins a string;
     - `,` joins a list;
     - `(` opens the tree's list;
     - `==`, `/` and `.` end a member, the left side of a condition, or the
       rule.

   So `[,]` is the list of the two labels `[` and `]` (C70 p. 30), and
   `A* + .(U*) == .(A*,U*).` has the label `.` twice before its final full
   stop (p. 38).

5. *Source.* **A comment is `**`, then any characters but `.` and `=`, then
   `.`** (C70 p. 25). So a comment ends at its first full stop, and the
   comments in `examples/` have none inside. A comment may stand only where
   a rule may: after `-DET-` and `-INV-`, never before them (the grammar's
   `<phase> ::= <détails> <inversion> <système-q>`).

6. *Decision.* **A program and its data are two files.** On the CDC they
   were presumably two parts of one deck. Nothing says how the parts were
   separated, so `qsys PROGRAM DATA` keeps them apart.

## Trees, lists and relations

7. *Source.* **`-NUL-` and `$$` are only ways of writing** (C70 pp. 9-10).
   `e(-NUL-)` is `e`; `l,-NUL-` is `l`; `$$c1…cn` is `c1,…,cn`. They are
   removed on reading, so a tree has one written form.

8. *Source.* **`-DANS-` and `-HORS-` are the five equivalences of pp. 14-15
   each**, taken in the order written.
   - When both sides have more than one element, the left side is taken
     apart first.
   - For `-DANS-` this is the only order that agrees with the examples.
   - For `-HORS-` the two orders give the same result.

   All seven examples of p. 15 hold (`tests/historical.rs`).

## Rules

9. *Source.* **A parameter matches whatever may be substituted for it**
   (C70 pp. 13, 26):
   - a label parameter, a label;
   - a tree parameter, a tree;
   - a list parameter, any list, `-NUL-` included;
   - `$$A*`, a list of one-character leaves that spell a label.

   A label parameter standing as a tree is the tree `A*(-NUL-)`, so it
   matches leaves only. This is why `A* + A*(U*)` applies to `A + A(1)` and
   not to `A(1) + A` (p. 29's figure).

10. *Source.* **Every substitution is a different rule** (C70 p. 26: two
    rules got by different substitutions "sont distinctes même si elles
    sont représentées par la même suite de caractères"). So matching finds
    every substitution, and each one that satisfies the condition adds its
    own path. Two substitutions that build the same trees add two paths.

11. *Source.* **`--` is replaced before anything else** (C70 p. 26, step B):
    - in a left member, by the left member of the rule before;
    - in a right member, by the right member of the rule before;
    - in a condition, by `(.` and the condition before and `.)`.

    Comments do not count as rules. `--` in the first rule, or in the
    condition of a rule after one with no condition, is an error.

12. *Source.* **`-INV-` exchanges the members of every rule**, after `--`
    has been replaced (C70 p. 33). The condition stays.

13. *Source.* **A parameter of the right member or the condition must be in
    the left member** (C70 p. 27). Under `-INV-` it is the other way round,
    since the members are exchanged (p. 33). A system that breaks this is
    refused.

14. *Source.* **Conditions**: `-OU-`, `-ET-`, `-NON-`, `(. .)` and the four
    relations, binding in the order relations, `-NON-`, `-ET-`, `-OU-` (C70
    p. 27).

## Applying a system

15. *Source.* **The first phase adds arrows until no rule applies to a path
    it has not yet been applied to** (C70 pp. 21-22).
    - The paths include those through arrows added in the phase. Without
      them `A + A(1)` could never give `A(1,1)` (p. 29).
    - The right member of each application is added as a new path between
      the first and last vertices of the path it was applied to, through new
      vertices.
    - The machine considers each path once, when the newest of its arrows is
      reached, against every rule and every substitution. So no rule is
      applied twice to the same path with the same substitution.

16. *Decision.* **A phase that adds more than 200,000 arrows is taken not
    to converge.** C70 says only that the result is then undefined (p. 22).
    The printout says so, and the treatment stops. `--max-arrows` changes
    the limit.

17. *Source.* **The second phase removes the arrows of every path some rule
    was applied to, then every arrow no longer on a path from the entry to
    the exit** (C70 p. 22).
    - The graph keeps its entry and exit through every phase.
    - A graph left with no arrows is printed as empty, and the phases after
      it have nothing to do.

18. *Source.* **A treatment applies its phases in order, each to the result
    of the one before; data of several graphs, separated by `/`, are
    treated one graph at a time** (C70 p. 34).

## Printing

19. *Source, from the printed results.* **A graph is printed as paths
    without crossings, `-01- A + B -02-`, the vertices numbered afresh from
    01, two digits at least** (C70 pp. 38-40; T72).
    - A path without crossings runs between vertices that are the entry, the
      exit, or have other than one arrow in and one out (p. 19).
    - The numbering is a decision fitted to pp. 38-40:
      - start at the entry and go depth first;
      - print each path as it is reached;
      - number a vertex when it is first printed;
      - print a vertex's own paths as soon as the last path into it has been
        printed.

    With rule 20, this reproduces the 35 lines of p. 38-40 exactly, numbers
    and all.

20. *Decision.* **The paths leaving a vertex are printed in the order of
    their trees, compared character by character in display code.**
    - C70 does not say how the program ordered them.
    - At all six branching vertices of the synthesis graph (pp. 38-40) the
      paths are in ascending order, and this order reproduces the page.
    - But the eighteen sentences of the last phase (pp. 40-41) are printed
      in exactly the **descending** order, the reverse of what is printed
      here.
    - No simple order of making or keeping arrows fits both pages. The order
      was most likely a side effect of the lost program's lists. The test
      for pp. 40-41 checks the reversal (`c70_p40_paraphrases`).

21. *Decision.* **Headings.** The graphs after each phase are headed
    `PHASE n`, as in T72. The data graph is headed `DONNEES`; no example of
    that heading survives. The details printed for `-DET-` (C70 p. 34: "les
    détails de l'application de chaque règle") are in a form of our own:
    - the rule;
    - the path;
    - the substitution;
    - the arrows added.
