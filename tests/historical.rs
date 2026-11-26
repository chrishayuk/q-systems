//! The acceptance tests: every computation C70 gives a result for, run, and
//! compared with what C70 says. Each test names its page (the printed page
//! number of the report, not the page of the scan).

use qsystems::machine::{apply, run, strings_text, Graph, Limits, System};
use qsystems::matching::{build_tree, Env, Val};
use qsystems::print::graph_text;
use qsystems::report::{graphs, run_all, systems};
use qsystems::syntax::{parse_list, parse_system, parse_tree, PTree, Var};
use qsystems::tree::{chain_string, dans, hors, is_label, Tree};

fn list(s: &str) -> Vec<Tree> {
    parse_list(s).unwrap()
}

fn tree(s: &str) -> Tree {
    parse_tree(s).unwrap()
}

fn sys(src: &str) -> System {
    System::new(&parse_system(src).unwrap()).unwrap()
}

fn one(src: &str, data: &str) -> Graph {
    let g = &graphs(data).unwrap()[0];
    apply(&sys(src), g, Limits::default()).result
}

fn printed(g: &Graph) -> String {
    graph_text(g, 0)
}

fn example(name: &str) -> (String, String) {
    let p = std::fs::read_to_string(format!("examples/{name}.q")).unwrap();
    let d = std::fs::read_to_string(format!("examples/{name}.dat")).unwrap();
    (p, d)
}

/// C70 p. 7: five correct labels and five incorrect ones.
#[test]
fn c70_p7_labels() {
    for l in ["+", "-PL3", "(", "HIBOU", ",A"] {
        assert!(is_label(l), "{l} is correct");
    }
    for l in ["/OUI/", "(A)", "DIT-IL", "3.14", "Z."] {
        assert!(!is_label(l), "{l} is incorrect");
    }
}

/// C70 pp. 9-10: three trees, written as expressions.
#[test]
fn c70_p9_trees() {
    let t = tree("PHRASE(SN(IL),SV(V(MANGE),SN(LA,CHOUCROUTE)))");
    assert_eq!(t.label.as_ref(), "PHRASE");
    assert_eq!(t.kids.len(), 2);
    assert_eq!(tree("BATEAU"), Tree::leaf("BATEAU"));
    let arith = tree("+(10,*(-30,45))");
    assert_eq!(arith.label.as_ref(), "+");
    assert_eq!(arith.kids[1].kids[0], Tree::leaf("-30"));
}

/// C70 pp. 9-10: `l,-NUL-`, `-NUL-,l` and `l` are equivalent, as are
/// `e(-NUL-)` and `e`, and `$$c1c2…cn` and `c1,c2,…,cn`. C70 p. 10 writes
/// one tree three ways.
#[test]
fn c70_p10_ways_of_writing() {
    let a = tree("AB(*,C(+,A,B),D,3)");
    assert_eq!(a, tree("AB(*(-NUL-),C($$+AB),D,3)"));
    assert_eq!(a, tree("AB(*,-NUL-,-NUL-,C(+,A,B),D,3)"));
    assert_eq!(list("A,B,-NUL-"), list("-NUL-,A,B"));
}

/// C70 p. 13: the parameterised tree PH(X*,SN(A*(LE,CHAT)),I*), with
/// -NUL-, NGM and SV(V(MANGE)) for X*, A* and I*.
#[test]
fn c70_p12_substitution() {
    let r = parse_system("PH(X*,SN(A*(LE,CHAT)),I*) == Z.").unwrap();
    let p: &PTree = &r.rules[0].left[0];
    let v = |c| Var { letter: c, index: 0 };
    let env = Env(vec![
        (v('X'), Val::List(vec![])),
        (v('A'), Val::Label("NGM".into())),
        (v('I'), Val::Tree(tree("SV(V(MANGE))"))),
    ]);
    assert_eq!(build_tree(p, &env), tree("PH(SN(NGM(LE,CHAT)),SV(V(MANGE)))"));
}

/// C70 p. 14: four (in)equalities between lists.
#[test]
fn c70_p14_equality() {
    assert_eq!(list("0,1,2,3,4,5,6,7,8,9"), list("$$0123456789"));
    assert_eq!(list("A(-NUL-)"), list("A"));
    assert_ne!(list("AB"), list("AB(X)"));
    assert_eq!(list("-NUL-"), list("-NUL-,-NUL-"));
}

/// C70 p. 15: the seven examples of -DANS- and -HORS-.
#[test]
fn c70_p15_dans_hors() {
    assert!(hors(&list("1,9,7,1"), &list("0,2,3")));
    assert!(dans(&list("$$1971"), &list("$$0123456789")));
    assert!(dans(&list("1,9,7,1"), &list("0,1,2,3,4,5,6,7,8,9")));
    assert!(dans(&list("PH"), &list("PH(SN(BORIS),SV(DORT))")));
    assert!(hors(&list("SN"), &list("PH(SN(BORIS),SV(DORT))")));
    assert!(dans(&list("A,B(A,B),A"), &list("B(B(A),A),A(C)")));
    assert!(hors(&list("A,B(A,A),A"), &list("B(B(A),B),B(C)")));
}

/// C70 p. 14: -HORS- "n'est pas l'exacte négation de -DANS-"; p. 15: it is
/// symmetric, and on lists of labels it is disjointness of non-empty sets.
#[test]
fn c70_p15_hors_is_not_the_negation_of_dans() {
    assert!(!dans(&list("A"), &list("B")) && hors(&list("A"), &list("B")));
    assert!(dans(&list("A"), &list("A(B)")) && !hors(&list("A"), &list("A(B)")));
    // both false
    assert!(!dans(&list("A(C)"), &list("A(B)")) || !hors(&list("A(C)"), &list("A(B)")));
    assert!(!dans(&list("A,B"), &list("A")) && !hors(&list("A,B"), &list("A")));
    assert!(!hors(&list("-NUL-"), &list("A")) && !hors(&list("A"), &list("-NUL-")));
    for (a, b) in [
        ("A,B(A,A),A", "B(B(A),B),B(C)"),
        ("1,9", "0,2"),
        ("SN", "PH(SN)"),
        ("A,B", "A"),
    ] {
        assert_eq!(hors(&list(a), &list(b)), hors(&list(b), &list(a)), "{a} / {b}");
    }
}

/// C70 pp. 17-20: a graph of strings carries six strings, and may be
/// written arrow by arrow or with paths without crossings.
#[test]
fn c70_p17_graph_of_strings() {
    let arrows = "-1- OUI -2- -1- NON -2- -2- SN(PAUL) -3- -3- EST -4- -4- VENU -5- \
                  -3- V(MANGER,3PSII) -5- -5- HIER -6- -3- A -7- -7- JOUE -8- -8- MARDI -6- -6- . -9-";
    let paths = "-1- OUI -2- -1- NON -2- -2- SN(PAUL) -3- -3- EST + VENU -4- -3- V(MANGER,3PSII) -4- \
                 -4- HIER -5- -3- A + JOUE + MARDI -5- -5- . -6-";
    let mut a = strings_text(&graphs(arrows).unwrap()[0]);
    let mut b = strings_text(&graphs(paths).unwrap()[0]);
    a.sort();
    b.sort();
    assert_eq!(a, b);
    let mut want = vec![
        "OUI + SN(PAUL) + EST + VENU + HIER + .",
        "OUI + SN(PAUL) + A + JOUE + MARDI + .",
        "OUI + SN(PAUL) + V(MANGER,3PSII) + HIER + .",
        "NON + SN(PAUL) + EST + VENU + HIER + .",
        "NON + SN(PAUL) + A + JOUE + MARDI + .",
        "NON + SN(PAUL) + V(MANGER,3PSII) + HIER + .",
    ];
    want.sort();
    assert_eq!(a, want);
}

/// C70 p. 20: a path from a vertex must be written after every path into it.
#[test]
fn c70_p20_order_of_paths() {
    assert!(graphs("-1- B -2- -0- A -1-").is_err());
    assert!(graphs("-0- A -1- -1- B -2-").is_ok());
}

const ABC: &str = "A* + A*(U*) == A*(1,U*) / A* -DANS- A,B,C.\nA(U*) + B(U*) + C(U*) == S(1,U*).\n";

/// C70 pp. 27-29: the example system applied to A+A+A+B+B+B+C+C+C. After
/// the first phase the graph of p. 29 has two A(1), one A(1,1), the same
/// for B and C, and S(1,1,1); after the second, only S(1,1,1).
#[test]
fn c70_p28_abc() {
    let g = &graphs("-0- A + A + A + B + B + B + C + C + C -1-").unwrap()[0];
    let o = apply(&sys(ABC), g, Limits::default());
    let count = |s: &str| o.grown.arrows.iter().filter(|a| a.tree.to_string() == s).count();
    for x in ["A", "B", "C"] {
        assert_eq!(count(x), 3);
        assert_eq!(count(&format!("{x}(1)")), 2);
        assert_eq!(count(&format!("{x}(1,1)")), 1);
    }
    assert_eq!(count("S(1,1,1)"), 1);
    assert_eq!(o.grown.arrows.len(), 9 + 9 + 1);
    assert_eq!(printed(&o.result), "-01- S(1,1,1) -02-\n");
}

/// C70 p. 24: S(1,…) is obtained only if the string is n A, n B, n C.
#[test]
fn c70_p24_only_if_as_many_a_b_c() {
    for a in 1..5 {
        for b in 1..5 {
            for c in 1..5 {
                let s: Vec<&str> = std::iter::repeat_n("A", a)
                    .chain(std::iter::repeat_n("B", b))
                    .chain(std::iter::repeat_n("C", c))
                    .collect();
                let r = one(ABC, &format!("-0- {} -1-", s.join(" + ")));
                let st = strings_text(&r);
                let is_s = st.len() == 1 && st[0].starts_with("S(");
                assert_eq!(is_s, a == b && b == c, "{a} {b} {c}: {st:?}");
            }
        }
    }
}

/// C70 pp. 32-33: the inverse system gives back A+A+A+B+B+B+C+C+C from
/// S(1,1,1), and -INV- does the inverting.
#[test]
fn c70_p32_inverse() {
    let r = one(&format!("-INV- {ABC}"), "-0- S(1,1,1) -1-");
    assert_eq!(printed(&r), "-01- A + A + A + B + B + B + C + C + C -02-\n");
    let by_hand = "A*(1,U*) == A* + A*(U*) / A* -DANS- A,B,C.\nS(1,U*) == A(U*) + B(U*) + C(U*).\n";
    assert_eq!(printed(&one(by_hand, "-0- S(1,1,1) -1-")), printed(&r));
}

/// C70 p. 33: with -INV-, a parameter of the left member or the condition
/// must be in the right member; without it, the other way.
#[test]
fn c70_p33_parameter_restriction() {
    assert!(System::new(&parse_system("A* == B(A*,I*).").unwrap()).is_err());
    assert!(System::new(&parse_system("-INV- A* == B(A*,I*).").unwrap()).is_ok());
    assert!(System::new(&parse_system("B(A*,I*) == A*.").unwrap()).is_ok());
    assert!(System::new(&parse_system("-INV- B(A*,I*) == A*.").unwrap()).is_err());
}

/// C70 p. 29: A* == MOT($$A*) takes VERCINGETORIX apart, and
/// MOT($$A*) == A* puts it back.
#[test]
fn c70_p29_disintegrator() {
    let r = one("A* == MOT($$A*).", "-0- VERCINGETORIX -1-");
    assert_eq!(printed(&r), "-01- MOT(V,E,R,C,I,N,G,E,T,O,R,I,X) -02-\n");
    let back = one("MOT($$A*) == A*.", "-0- MOT(V,E,R,C,I,N,G,E,T,O,R,I,X) -1-");
    assert_eq!(printed(&back), "-01- VERCINGETORIX -02-\n");
}

/// C70 p. 30: the inverse rules find the tree again from its structured
/// string.
#[test]
fn c70_p30_structured_string_to_tree() {
    let (p, d) = example("unstructure");
    let (s, runs) = run_all(&p, &d, Limits::default()).unwrap();
    let _ = s;
    assert_eq!(printed(runs[0].result()), "-01- A(B,C(A,E(F))) -02-\n");
}

/// C70 p. 30 says the two rules "produiraient" the structured string
/// [(A) + B + [(C) + A + [(E) + F + ] + ] + ]. As written, they do not
/// converge: [(A) is itself a tree A*(U*), with A* = [ and U* = A. With
/// the condition A* -HORS- [,] added, they produce exactly that string.
/// (docs/research.md, QSYS-E01.)
#[test]
fn c70_p30_tree_to_structured_string() {
    let (p, d) = example("structure");
    let (_, runs) = run_all(&p, &d, Limits { max_arrows: 5000 }).unwrap();
    assert!(!runs[0].phases[0].converged);
    let guarded = "A*(U*) == [(A*) + ](U*) / U* ≠ -NUL- -ET- A* -HORS- [,].\n](I*,U*) == I* + ](U*).\n";
    let r = one(guarded, &d);
    assert_eq!(printed(&r), "-01- [(A) + B + [(C) + A + [(E) + F + ] + ] + ] -02-\n");
}

/// C70 pp. 30-31: three rules remove every path carrying
/// LE + FAIT + QUE + LE + FAIT + QUE; "il ne reste plus rien".
#[test]
fn c70_p30_cul_de_sac() {
    let src = "LE + FAIT + QUE + LE + FAIT + QUE == /CUL + /DE + /SAC.\n\
               /CUL + /DE == /CULDE. /DE + /SAC == /DESAC.\n";
    let g = &graphs("-0- LE + FAIT + QUE + LE + FAIT + QUE -1-").unwrap()[0];
    let o = apply(&sys(src), g, Limits::default());
    let made: Vec<String> = o.grown.arrows.iter().skip(6).map(|a| a.tree.to_string()).collect();
    assert_eq!(made, ["/CUL", "/DE", "/SAC", "/CULDE", "/DESAC"]);
    assert!(o.result.is_empty());
}

fn paraphrase() -> Vec<String> {
    let (p, d) = example("paraphrase");
    let (_, runs) = run_all(&p, &d, Limits::default()).unwrap();
    runs[0].phases.iter().map(|o| printed(&o.result)).collect()
}

/// C70 p. 38: the analysis phase (the system of p. 37, inverted) gives the
/// normalised sentence drawn on p. 36.
#[test]
fn c70_p38_analysis() {
    assert_eq!(
        paraphrase()[0],
        "-01- PI(V(CONSTATE),SN(RICHARD),PH(V(ETONNE),PH(V(DETRUIT),SN(BENOIT),SN(BEAUCEVILLE)),SN(GAETANE))) + . -02-\n"
    );
}

/// C70 pp. 38-40: the synthesis phase gives the graph of 19 vertices and 35
/// paths without crossings printed there, line for line.
#[test]
fn c70_p38_synthesis_graph() {
    let want = std::fs::read_to_string("tests/c70/paraphrase-phase2.txt").unwrap();
    assert_eq!(paraphrase()[1], want);
}

/// C70 pp. 40-41: the last two phases give the eighteen paraphrases, each
/// on a path of its own. They are printed there in exactly the reverse of
/// the order they are printed here (docs/decisions.md, rule 20).
#[test]
fn c70_p40_paraphrases() {
    let want = std::fs::read_to_string("tests/c70/paraphrase-phase4.txt").unwrap();
    let mut want: Vec<&str> = want.lines().collect();
    want.reverse();
    let got = paraphrase()[3].clone();
    assert_eq!(got.lines().collect::<Vec<_>>(), want);
}

/// C70 p. 4 and p. 32: "pratiquement le même système-q peut être utilisé
/// pour décrire une transformation et la transformation inverse". Every
/// paraphrase the synthesis produces analyses back to the one sentence.
#[test]
fn c70_p4_reversibility_of_the_paraphrases() {
    let (p, _) = example("paraphrase");
    let analysis = &systems(&p).unwrap()[..1];
    let want = paraphrase()[0].clone();
    for line in std::fs::read_to_string("tests/c70/paraphrase-phase4.txt")
        .unwrap()
        .lines()
    {
        let words: Vec<Tree> = line
            .trim_start_matches("-01- ")
            .trim_end_matches(" -02-")
            .split(" + ")
            .map(tree)
            .collect();
        let r = run(analysis, &Graph::from_chain(&words), Limits::default());
        assert_eq!(printed(r.result()), want, "{}", chain_string(&words));
    }
}
