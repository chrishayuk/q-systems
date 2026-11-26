# Notes on C70

Colmerauer, *Les systèmes-Q ou un formalisme pour analyser et synthétiser
des phrases sur ordinateur*, Montréal, September 1970.

## How it was read

- The PDF is a scan with no text layer: 46 page images at about 300 dpi.
- The pages were rasterised and OCR'd with Tesseract (French and English),
  and the OCR was read through.
- Every page that carries a rule, an example, a grammar or a printed result
  was then read from the image:
  - pp. 6-7: characters and labels;
  - pp. 14-15: the relations;
  - pp. 22, 24: the two phases, and the missing page;
  - pp. 27-30: the example systems;
  - pp. 37-40: the paraphrase system and its results;
  - p. 45: the grammar.
- The OCR is unreliable on symbols (`≠`, `*`, `$$`, `[ ]`) and on some
  words, so no example here was taken from it.

Page numbers below are the printed ones. Scan page = printed page + 1, up to
p. 16. The scan has p. 16 twice, which shifts the count by one more; then
p. 23 is missing, which shifts it back.

## Structure of the report

| § | pages | content |
|---|---|---|
| — | 4-5 | Introduction. The combinatorial problem of analysis, and a graph to hold every partial grouping. Reversibility. Chaining systems: TAUM puts an English sentence through fifteen of them. The ALGOL program ran from October 1969; "actuellement" a FORTRAN translation is used |
| 1 | 6-7 | Characters, in four classes, and labels |
| 2 | 8-10 | Trees and lists; `-NUL-`; the disintegrator `$$` |
| 3 | 11-13 | Parameters: A-F labels, I-N trees, U-Z lists, with an optional digit |
| 4 | 14-15 | The relations `=`, `≠`, `-DANS-`, `-HORS-` |
| 5 | 16 | Strings: trees separated by `+` |
| 6 | 17-20 | Graphs of strings; written notation; paths without crossings |
| 7 | 21-24 | Simplified Q-systems and their two phases (p. 23 missing) |
| 8 | 25-31 | Q-systems: comments, `--`, parameters, conditions. Examples |
| 9 | 32-33 | Reversibility and `-INV-` |
| 10 | 34 | Q-treatments: `-REQ-`, `-DET-`, data separated by `/` |
| 11 | 35-42 | The paraphrase example, with its results; p. 42 is a drawing of the synthesis graph |
| App. | 43-45 | The whole grammar |

## Points that matter for the reconstruction

- **p. 6.** The characters are those of the CDC 6400: capitals, digits,
  `+ - * / ( ) $ = ≠ . ,` ("signes significatifs"), and `≡ [ ] : → ∨ ∧ ↑ ↓
  < > ≤ ≥ ¬ ;` ("signes non significatifs"). This is CDC display code, in
  display-code order. The arrow drawn after `:` is code 65.
- **p. 6.** A label is "une suite de caractères: le premier peut être
  quelconque mais les autres … doivent être des chiffres ou des lettres".
- **p. 7.** Correct labels: `+ -PL3 ( HIBOU ,A`. Incorrect: `/OUI/ (A)
  DIT-IL 3.14 Z.` The last is a barred Z, typed to tell it from 2.
- **p. 10.** `-NUL-`, `e(-NUL-) ≡ e`, and `$$c1…cn ≡ c1,…,cn` "à condition
  que les ci soient des caractères et que c1c2…cn soit une étiquette".
- **p. 13.** "Par 'substituer' j'entends que l'on remplace différentes
  occurrences d'un même paramètre par la même chose."
- **pp. 14-15.** The five equivalences each for `-DANS-` and `-HORS-`, and
  the seven examples. `-HORS-` "n'est pas l'exacte négation de -DANS-" and
  is symmetric.
- **p. 17.** The two conditions on a graph: no circuit; one entry and one
  exit, with every arrow on a path between them.
- **p. 20.** `ip = jq ⇒ p > q`. Graphs given as data may not use `-NUL-`
  or `$$`.
- **pp. 21-22.** The first phase: "On prend soin de ne pas appliquer
  plusieurs fois la même règle sur le même chemin et on continue le
  processus jusqu'à ce qu'il n'y ait plus de règles à appliquer. Si le
  processus ne converge pas le résultat est indéfini sinon il est
  indépendant de l'ordre dans lequel on a appliqué les règles." The second
  phase: "On supprime tout d'abord les flèches figurant dans des chemins sur
  lesquels s'est appliquée au moins une règle. On supprime ensuite les
  flèches qui ne figurent plus dans un chemin allant de l'entrée à la
  sortie."
- **p. 26.** Two rules got by different substitutions "sont distinctes même
  si elles sont représentées par la même suite de caractères".
- **p. 27.** Precedence: relations, then `-NON-`, `-ET-`, `-OU-`. The
  restriction on parameters is "pour des raisons d'efficacité".
- **p. 29.** The figure of the first phase on AAABBBCCC: two arrows A(1), one
  A(1,1), the same for B and C, and S(1,1,1).
- **p. 30.** The structured-string rules, and the cul-de-sac rules. See
  research QSYS-E01 and E07.
- **p. 33.** Under `-INV-` the restriction on parameters is turned round.
- **p. 34.** "Lors de l'exécution effective d'un traitement-q l'ordinateur
  imprime le ou les graphes qui lui sont fournis en données et les graphes
  obtenus après l'application de chaque phase."
- **pp. 37-38.** The paraphrase system. Its second rule begins with `--`, so
  `--` is in use, not only defined. The comment `** NOMINALISATION D UNE
  PHRASE.` has no apostrophe, because `'` is not a character.
- **pp. 38-40.** The results of the four phases. The second is printed as 35
  paths without crossings over 19 vertices, numbered from `-01-`.

## Misprints and oddities

- p. 22 calls it a "graphe d'arbres" where it means a graph of strings. The
  report uses both names.
- p. 41: one result has a stray full stop, `GAETANE +. EST`. It was read as
  `GAETANE + EST`.
- p. 38: `DE & GAETANE` in the scan is an overstrike of `+`.
