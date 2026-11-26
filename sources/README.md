# Sources

No program text of the Q-systems is known to survive. Colmerauer wrote the
first implementation in ALGOL for the CDC 6400 of the Université de Montréal,
and it was working by October 1969. Michel van Caneghem and François Stellin
then translated it into FORTRAN, and Gilles Stewart wrote a fast version in
machine code (Colmerauer and Roussel 1996). This reconstruction rests on
Colmerauer's own description of the formalism, which is complete enough to
build from: it gives a grammar, the meaning of every construct and worked
examples with their printed results.

Copies of the documents are not kept in the repository. `sources/downloads/`
is ignored by git. The list below says where each one can be had. What was
read from each, and how, is in `notes/`.

## Primary source

| | source | what was used | access |
|---|---|---|---|
| **C70** | Alain Colmerauer, *Les systèmes-Q ou un formalisme pour analyser et synthétiser des phrases sur ordinateur*, publication interne n° 43, Département d'Informatique, Université de Montréal, September 1970. 45 numbered pages and a cover. Republished in *Traitement Automatique des Langues* 33(1-2) (1992) 105-148, according to "A Short Bibliography of Alain Colmerauer" (TPLP, 2004); that version has not been seen | all of it, read from the page scans. Every example was transcribed from the images, not the OCR; [notes](notes/colmerauer-1970.md) | `http://alain.colmerauer.free.fr/alcol/ArchivesPublications/SystemesQ/SystemesQ.pdf` (a 46-page scan, 69 MB, linked from the Computer History Museum's Prolog archive). **The scan is missing printed page 23 and has page 16 twice** |

## A printout of the system in use

| | source | what was used | access |
|---|---|---|---|
| **T72** | Irena Findeisen-Bellert, interactive session with TAUM translating a sentence from English to French, c. 1972. One long sheet of line-printer output, the phases 7, 10, 12 and 21 of a TAUM treatment. Gift of Janusz Bień to the CHM | the print format: `PHASE n`, then each graph as `-01- … -02-` with two-digit vertex numbers, lines folded at a fixed width wherever they fall; and the structured strings `[(PH) + … + ]` of C70 p. 30 in real use | `https://softwarepreservation.computerhistory.org/prolog/montreal/src/Findeisen-Bellert-TAUM-1972.pdf` |

## Secondary sources

- A. Colmerauer and P. Roussel, "The birth of Prolog", in *History of
  Programming Languages II* (ACM, 1996). It gives the history quoted above,
  and calls the Q-systems an ancestor of Prolog. It was read as quoted on the
  CHM page.
- The Computer History Museum's Prolog archive,
  `https://softwarepreservation.computerhistory.org/prolog/`, section
  "Université de Montréal". It is the source of both documents above.

## Not seen

- *TAUM 71*, ed. A. Colmerauer (Projet de Traduction Automatique de
  l'Université de Montréal, January 1971), 223 pages. The CHM lists one part
  of it: Harris and Bélisle's POLYGRAM English morphology (pp. 46-105).
  Colmerauer and Roussel name the other TAUM grammars written in Q-systems:
  - Kittredge's analysis of English;
  - Stewart's transfer;
  - Dansereau's French synthesis;
  - van Caneghem's French morphology.

  Whether those grammars are in *TAUM 71* has not been checked. They are the
  earliest large Q-system programs, and none has been seen.
- Any manual of the FORTRAN or machine-code versions, or of the later
  Q-systems of METEO, the weather-forecast translator later written in
  Q-systems (Colmerauer and Roussel). Nothing here describes them, or says
  whether they changed the formalism.
- Printed page 23 of C70. Page 22 ends with the second phase of a simplified
  system, and page 24 begins mid-sentence: "que si le graphe d'arbres initial
  est de la forme -0- A + A + … + C -1- et que le nombre de A est le même que
  celui de B et que celui de C." So page 23 most likely held a first example,
  a simplified system recognising AⁿBⁿCⁿ. Section 8 gives the parameterised
  form of such a system on pp. 27-29, and that is what is tested here.
