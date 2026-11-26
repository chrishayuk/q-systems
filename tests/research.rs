//! The experiments of docs/research.md, with the numbers recorded there.

use qsystems::machine::{apply, run, strings_text, Arrow, Graph, Limits, System};
use qsystems::print::graph_text;
use qsystems::report::systems;
use qsystems::syntax::parse_tree;
use qsystems::tree::Tree;
use std::collections::{BTreeMap, BTreeSet};

fn paraphrase() -> Vec<System> {
    systems(&std::fs::read_to_string("examples/paraphrase.q").unwrap()).unwrap()
}

/// Every normalised sentence of C70 p. 35 with at most `depth` levels of
/// embedded PH.
fn sentences(depth: usize) -> Vec<String> {
    fn inner(depth: usize) -> Vec<String> {
        let mut args: Vec<String> = ["BEAUCEVILLE", "BENOIT", "GAETANE", "RICHARD"]
            .iter()
            .map(|n| format!("SN({n})"))
            .collect();
        if depth > 0 {
            args.extend(inner(depth - 1).into_iter().map(|i| format!("PH({i})")));
        }
        let mut out = Vec::new();
        for v in ["DETRUIT", "ETONNE", "CONSTATE"] {
            for a in &args {
                for b in &args {
                    out.push(format!("V({v}),{a},{b}"));
                }
            }
        }
        out
    }
    inner(depth).into_iter().map(|i| format!("PI({i})")).collect()
}

fn words(s: &str) -> Vec<Tree> {
    s.split(" + ").map(|w| parse_tree(w).unwrap()).collect()
}

/// QSYS-E03: every normalised sentence of C70 p. 35 with at most one
/// embedded clause, synthesised; no sentence comes from two trees, and every
/// sentence, analysed by the inverted system, gives back its one tree. Run
/// with `cargo test --release --test research -- --ignored e03`.
#[test]
#[ignore = "about a minute in release: cargo test --release --test research -- --ignored"]
fn e03_ambiguity() {
    let sys = paraphrase();
    let (analysis, synth) = (&sys[..1], &sys[1..2]);
    let mut from: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let all = sentences(1);
    assert_eq!(all.len(), 8112);
    for s in &all {
        let g = Graph::from_chain(&[parse_tree(s).unwrap(), Tree::leaf(".")]);
        let r = run(synth, &g, Limits::default());
        for st in strings_text(r.result()) {
            from.entry(st).or_default().push(s.clone());
        }
    }
    let amb = from.values().filter(|v| v.len() > 1).count();
    println!("{} trees, {} sentences, {} ambiguous", all.len(), from.len(), amb);
    assert_eq!(from.len(), 131_424);
    assert_eq!(amb, 0);
    let mut back = 0;
    for (sentence, trees) in &from {
        let r = run(analysis, &Graph::from_chain(&words(sentence)), Limits::default());
        let got = strings_text(r.result());
        assert_eq!(got, vec![format!("{} + .", trees[0])], "{sentence}");
        back += 1;
    }
    println!("{back} sentences analysed, each to its one tree");
}

/// QSYS-E04: how much the first phase grows the graph, and how much the
/// second phase throws away, in each phase of C70's treatment.
#[test]
fn e04_growth_and_pruning() {
    let sys = paraphrase();
    let src = "RICHARD + CONSTATE + QUE + LA + DESTRUCTION + DE + BEAUCEVILLE + PAR + BENOIT + ETONNE + GAETANE + .";
    let mut g = Graph::from_chain(&words(src));
    let mut rows = Vec::new();
    for (i, s) in sys.iter().enumerate() {
        let o = apply(s, &g, Limits::default());
        rows.push((
            g.arrows.len(),
            o.applications.len(),
            o.grown.arrows.len(),
            o.rewritten.len(),
            o.stranded.len(),
            o.result.arrows.len(),
            strings_text(&o.result).len(),
        ));
        println!(
            "phase {}: {} arrows in, {} applications, {} arrows after the first phase, {} removed as rewritten, {} stranded, {} left carrying {} strings",
            i + 1,
            g.arrows.len(),
            o.applications.len(),
            o.grown.arrows.len(),
            o.rewritten.len(),
            o.stranded.len(),
            o.result.arrows.len(),
            strings_text(&o.result).len()
        );
        g = o.result;
    }
    let total_words: usize = strings_text(&g).iter().map(|s| s.split(" + ").count()).sum();
    println!("18 sentences as separate strings: {total_words} arrows");
    assert_eq!(
        rows,
        [
            (12, 25, 50, 35, 13, 2, 1),
            (2, 156, 330, 191, 0, 139, 18),
            (139, 224, 363, 345, 0, 18, 18),
            (18, 264, 546, 264, 0, 282, 18)
        ]
    );
    assert_eq!(total_words, 282);
}

/// QSYS-E05 (an experiment, not historical): an ambiguous grammar of our
/// own. Both readings come out of the one analysis, as two arrows between
/// the same two vertices.
#[test]
fn e05_two_readings_at_once() {
    let src = "\
** PP ATTACHMENT: WITH THE TELESCOPE GOES WITH THE SEEING OR THE MAN.
JEAN == SN(JEAN).
L + HOMME == SN(HOMME).
LE + TELESCOPE == SN(TELESCOPE).
AVEC + SN(I*) == SP(AVEC,I*).
SN(I*) + SP(J*,K*) == SN(I*,SP(J*,K*)).
VOIT + SN(U*) == SV(VOIT,SN(U*)).
SV(I*,J*) + SP(K*,L*) == SV(I*,J*,SP(K*,L*)).
SN(U*) + SV(W*) + . == PH(SN(U*),SV(W*)).
";
    let sys = systems(src).unwrap();
    let g = Graph::from_chain(&words("JEAN + VOIT + L + HOMME + AVEC + LE + TELESCOPE + ."));
    let o = apply(&sys[0], &g, Limits::default());
    let out = graph_text(&o.result, 0);
    println!("{out}");
    assert_eq!(out.lines().count(), 2);
}

/// QSYS-E06: C70 p. 22, "si le processus converge … il est indépendant de
/// l'ordre dans lequel on a appliqué les règles". The rules of every phase
/// of the paraphrase treatment, in every rotation and reversed, give the
/// same printed graphs.
#[test]
fn e06_order_independence() {
    let sys = paraphrase();
    let src = "RICHARD + CONSTATE + QUE + LA + DESTRUCTION + DE + BEAUCEVILLE + PAR + BENOIT + ETONNE + GAETANE + .";
    let g = Graph::from_chain(&words(src));
    let base: Vec<String> = run(&sys, &g, Limits::default())
        .phases
        .iter()
        .map(|o| graph_text(&o.result, 0))
        .collect();
    let n = sys[1].rules.len();
    let mut tried = 0;
    for k in 0..n {
        for rev in [false, true] {
            let alt: Vec<System> = sys
                .iter()
                .map(|s| {
                    let mut s = s.clone();
                    let m = s.rules.len();
                    s.rules.rotate_left(k % m);
                    if rev {
                        s.rules.reverse();
                    }
                    s
                })
                .collect();
            let got: Vec<String> = run(&alt, &g, Limits::default())
                .phases
                .iter()
                .map(|o| graph_text(&o.result, 0))
                .collect();
            assert_eq!(got, base, "rotation {k}, reversed {rev}");
            tried += 1;
        }
    }
    println!("{tried} orders of the rules, one printout");
    assert_eq!(tried, 32);
}

/// QSYS-E07: the cul-de-sac rules of C70 pp. 30-31 remove "tout chemin
/// portant" LE + FAIT + QUE + LE + FAIT + QUE, and with it every other
/// string that shares an arrow of that path: here LE + FAIT + QUE + BENOIT.
#[test]
fn e07_removal_takes_shared_arrows() {
    let p = std::fs::read_to_string("research/e07-shared-arrows.q").unwrap();
    let d = std::fs::read_to_string("research/e07-shared-arrows.dat").unwrap();
    let (_, runs) = qsystems::report::run_all(&p, &d, Limits::default()).unwrap();
    let before = strings_text(&runs[0].input);
    assert_eq!(before.len(), 2);
    assert!(runs[0].result().is_empty());
}

/// `n` independent choices between A and B: n + 1 vertices, 2n arrows,
/// 2^n strings. With `start`, an arrow S comes first.
fn choices(n: usize, start: bool) -> Graph {
    let o = start as usize;
    let mut arrows = Vec::new();
    if start {
        arrows.push(Arrow {
            from: 0,
            to: 1,
            tree: Tree::leaf("S"),
        });
    }
    for i in 0..n {
        for l in ["A", "B"] {
            arrows.push(Arrow {
                from: i + o,
                to: i + o + 1,
                tree: Tree::leaf(l),
            });
        }
    }
    Graph {
        arrows,
        nverts: n + o + 1,
        entry: 0,
        exit: n + o,
    }
}

fn distinct(g: &Graph) -> usize {
    strings_text(g).into_iter().collect::<BTreeSet<_>>().len()
}

/// Arrows with the same ends and the same tree as an earlier arrow.
fn repeated(g: &Graph) -> usize {
    let mut seen = BTreeSet::new();
    g.arrows
        .iter()
        .filter(|a| !seen.insert((a.from, a.to, a.tree.to_string())))
        .count()
}

const PARITY: &str = "\
S + A == O.
S + B == E.
E + A == O.
E + B == E.
O + A == E.
O + B == O.
";

/// QSYS-E08 (an experiment, not historical): what a phase costs on a graph
/// of 2^n strings. A rule that looks at a bounded stretch costs in
/// proportion to the arrows; a rule chain that carries a result along the
/// string costs one application per prefix, even when the results are only
/// two, because equal arrows are never merged.
#[test]
fn e08_work_on_shared_alternatives() {
    let run1 = |src: &str, g: &Graph| apply(&systems(src).unwrap()[0], g, Limits::default());
    // One arrow at a time, and one pair at a time.
    for n in [10, 20, 40] {
        let g = choices(n, false);
        assert_eq!(g.count_strings(), 1u128 << n);
        let o = run1("A == X.\nB == Y.\n", &g);
        println!(
            "relabel  n={n}: {} applications, {} arrows kept, {} strings",
            o.applications.len(),
            o.result.arrows.len(),
            o.result.count_strings()
        );
        assert_eq!(
            (o.applications.len(), o.result.arrows.len(), o.result.count_strings()),
            (2 * n, 2 * n, 1u128 << n)
        );
        let o = run1("A + B == C.\n", &g);
        println!(
            "pair     n={n}: {} applications, {} arrows kept, {} strings",
            o.applications.len(),
            o.result.arrows.len(),
            o.result.count_strings()
        );
        assert_eq!(
            (o.applications.len(), o.result.arrows.len(), o.result.count_strings()),
            (n - 1, n + 1, 2)
        );
        // Fan out: one string in, 2^n out.
        let g = Graph::from_chain(&vec![Tree::leaf("A"); n]);
        let o = run1("A == X.\nA == Y.\n", &g);
        println!(
            "fan out  n={n}: {} applications, {} arrows kept, {} strings",
            o.applications.len(),
            o.result.arrows.len(),
            o.result.count_strings()
        );
        assert_eq!(
            (o.applications.len(), o.result.arrows.len(), o.result.count_strings()),
            (2 * n, 2 * n, 1u128 << n)
        );
    }
    // Carried from left to right: the parity of the number of A, and the
    // whole string packed into one tree.
    for n in [4, 8, 12] {
        let g = choices(n, true);
        let p = run1(PARITY, &g);
        let k = run1(
            "S + A == P(A).\nS + B == P(B).\nP(U*) + A == P(U*,A).\nP(U*) + B == P(U*,B).\n",
            &g,
        );
        for (name, o) in [("parity", &p), ("pack", &k)] {
            println!(
                "{name:8} n={n}: {} applications, {} arrows made, {} of them repeats, {} kept, {} distinct strings",
                o.applications.len(),
                o.grown.arrows.len() - g.arrows.len(),
                repeated(&o.grown),
                o.result.arrows.len(),
                distinct(&o.result)
            );
            assert_eq!(o.applications.len(), (1 << (n + 1)) - 2);
            assert_eq!(o.result.arrows.len(), 1 << n);
        }
        assert_eq!(distinct(&p.result), 2);
        assert_eq!(repeated(&p.grown), (1 << (n + 1)) - 2 - 2 * n);
        assert_eq!(distinct(&k.result), 1 << n);
        assert_eq!(repeated(&k.grown), 0);
    }
    // None of C70's own examples ever makes two equal arrows, so its printed
    // results cannot say whether the lost program merged them.
    for e in ["abc", "mot", "structure", "paraphrase"] {
        let p = std::fs::read_to_string(format!("examples/{e}.q")).unwrap();
        let d = std::fs::read_to_string(format!("examples/{e}.dat")).unwrap();
        let (_, runs) = qsystems::report::run_all(&p, &d, Limits { max_arrows: 2000 }).unwrap();
        for r in &runs {
            for o in &r.phases {
                assert_eq!(repeated(&o.grown), 0, "{e}");
            }
        }
    }
}

/// QSYS-E09 (an experiment, not historical): what the second phase loses.
#[test]
fn e09_what_the_second_phase_loses() {
    let set = |g: &Graph| strings_text(g).into_iter().collect::<BTreeSet<_>>();
    // 1. A rule removes strings it never matched, because they share its
    //    arrows. 2. Copying every arrow into a new alphabet keeps them.
    let pair = systems("A + B == C.\n").unwrap();
    let carry = systems("A == X.\nB == Y.\nA + B == C.\n").unwrap();
    let mut rows = Vec::new();
    for n in [2, 3, 4, 6, 8, 10] {
        let g = choices(n, false);
        let before = set(&g);
        let untouched: Vec<&String> = before.iter().filter(|s| !s.contains("A + B")).collect();
        let after = set(&apply(&pair[0], &g, Limits::default()).result);
        let survived = untouched.iter().filter(|s| after.contains(**s)).count();
        let carried = set(&apply(&carry[0], &g, Limits::default()).result);
        let relabelled = before
            .iter()
            .filter(|s| carried.contains(&s.replace('A', "X").replace('B', "Y")))
            .count();
        println!(
            "n={n}: {} strings, {} with no A + B; A + B == C leaves {} strings, {} of those {}; with the copies, {} strings, all {} originals relabelled: {}",
            before.len(), untouched.len(), after.len(), survived, untouched.len(), carried.len(), before.len(), relabelled == before.len()
        );
        assert_eq!(relabelled, before.len());
        rows.push((untouched.len(), survived, after.len(), carried.len()));
    }
    assert_eq!(
        rows,
        [
            (3, 1, 2, 5),
            (4, 0, 2, 12),
            (5, 0, 2, 29),
            (7, 0, 2, 169),
            (9, 0, 2, 985),
            (11, 0, 2, 5741)
        ]
    );
    // 3. Round trips: a system, then the same system under -INV-, on every
    //    string of six choices.
    let n = 6;
    let mut got = Vec::new();
    for (name, src, start) in [
        ("one to one", "A == X.\nB == Y.\n", false),
        ("two to one", "A == X.\nB == X.\n", false),
        ("parity", PARITY, true),
    ] {
        let fwd = systems(src).unwrap();
        let inv = systems(&format!("-INV-\n{src}")).unwrap();
        let (mut exact, mut stopped, mut sizes) = (0, 0, BTreeSet::new());
        for k in 0..1u32 << n {
            let mut chain: Vec<Tree> = (0..n)
                .map(|i| Tree::leaf(if k >> i & 1 == 1 { "B" } else { "A" }))
                .collect();
            if start {
                chain.insert(0, Tree::leaf("S"));
            }
            let g = Graph::from_chain(&chain);
            let there = run(&fwd, &g, Limits::default());
            let back = run(&inv, there.result(), Limits { max_arrows: 5000 });
            if !back.phases[0].converged {
                stopped += 1;
                continue;
            }
            let b = set(back.result());
            sizes.insert(b.len());
            exact += (b == set(&g)) as usize;
        }
        println!("{name}: of {} strings, {exact} come back exactly; {stopped} inverse runs do not converge; strings back per input {sizes:?}", 1 << n);
        got.push((exact, stopped, sizes.into_iter().collect::<Vec<_>>()));
    }
    assert_eq!(got, [(64, 0, vec![1]), (0, 0, vec![64]), (0, 64, vec![])]);
    // 4. A copy rule never converges: its new arrow is itself a path the
    //    rule applies to.
    let o = apply(
        &systems("A == A.\n").unwrap()[0],
        &Graph::from_chain(&[Tree::leaf("A")]),
        Limits { max_arrows: 100 },
    );
    assert!(!o.converged);
}
