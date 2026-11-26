//! The experiments of docs/research.md, with the numbers recorded there.

use qsystems::machine::{apply, run, strings_text, Graph, Limits, System};
use qsystems::print::graph_text;
use qsystems::report::systems;
use qsystems::syntax::parse_tree;
use qsystems::tree::Tree;
use std::collections::BTreeMap;

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
