** QSYS-E15, AN EXPERIMENT: ONE STEP OF THE CELLULAR AUTOMATON RULE 110.
** THE ROW IS READ LEFT TO RIGHT, THREE CELLS AT A TIME, INTO ACC.
S(1,1,1,U*) + ACC(V*) == S(1,1,U*) + ACC(V*,0).
S(1,1,0,U*) + ACC(V*) == S(1,0,U*) + ACC(V*,1).
S(1,0,1,U*) + ACC(V*) == S(0,1,U*) + ACC(V*,1).
S(1,0,0,U*) + ACC(V*) == S(0,0,U*) + ACC(V*,0).
S(0,1,1,U*) + ACC(V*) == S(1,1,U*) + ACC(V*,1).
S(0,1,0,U*) + ACC(V*) == S(1,0,U*) + ACC(V*,1).
S(0,0,1,U*) + ACC(V*) == S(0,1,U*) + ACC(V*,1).
S(0,0,0,U*) + ACC(V*) == S(0,0,U*) + ACC(V*,0).
S(A*,B*) + ACC(V*) == ROW(V*).
