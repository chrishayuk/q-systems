# 9. Reference

C70's appendix gives the whole grammar (pp. 43-45). In short:

## Text

| | |
|---|---|
| label | any character, then capital letters and digits: `HIBOU`, `-30`, `+`, `.` |
| tree | `label` or `label(list)` |
| list | simple lists separated by `,`; a simple list is `-NUL-`, a tree, or `$$label` |
| string | trees separated by `+` |
| graph | paths without crossings `-i- string -j-`, every path into a vertex before any path out of it |
| data | graphs separated by `/` |

## Treatments

| | |
|---|---|
| treatment | phases separated by `-REQ-` |
| phase | `-DET-`? `-INV-`? then rules and comments |
| rule | `left == right.` or `left == right / condition.` |
| `--` | the left member, right member or condition of the rule before |
| comment | `**`, any characters but `.` and `=`, then `.` |

## Parameters

| | |
|---|---|
| `A*` … `F*` | a label; standing as a tree, a leaf |
| `I*` … `N*` | a tree |
| `U*` … `Z*` | a list, perhaps empty |
| `X*1` | a digit makes another parameter of the same kind |

## Conditions

These are listed from tightest binding to loosest:

1. `=` and `≠` (or `#`), `-DANS-` and `-HORS-`, between lists;
2. `-NON-`;
3. `-ET-`;
4. `-OU-`.

`(.` and `.)` group.

## The machine

1. **Adding.** For every path matching the left member, under every
   substitution that makes the condition true, a path carrying the right
   member is added beside it. This goes on until no rule applies to a path
   it has not been applied to.
2. **Removing.** The arrows of every path a rule was applied to are removed,
   then every arrow off every path from the entry to the exit.

Every rule, with the page of C70 it rests on, is in `docs/decisions.md`.
