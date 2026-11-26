//! The printout of a treatment: the data graph, then the graph after each
//! phase (C70 p. 34), and, for a phase marked `-DET-`, every application of
//! every rule.

use crate::machine::{run, Graph, Limits, Outcome, Run, System};
use crate::print::graph_text;
use crate::syntax::{parse_data, parse_treatment, pchain_string};
use crate::tree::chain_string;

#[derive(Clone, Copy, Debug, Default)]
pub struct Opts {
    pub width: usize,
    /// Print details for every phase, as if each were marked `-DET-`.
    pub det_all: bool,
    pub limits: Limits,
}

pub fn systems(program: &str) -> Result<Vec<System>, String> {
    let t = parse_treatment(program).map_err(|e| e.to_string())?;
    t.phases
        .iter()
        .map(|p| System::new(p).map_err(|e| e.to_string()))
        .collect()
}

pub fn graphs(data: &str) -> Result<Vec<Graph>, String> {
    let gs = parse_data(data).map_err(|e| format!("data: {e}"))?;
    gs.iter()
        .enumerate()
        .map(|(i, g)| Graph::from_text(g).map_err(|e| format!("data, graph {}: {e}", i + 1)))
        .collect()
}

/// Run a treatment on every graph of its data.
pub fn run_all(program: &str, data: &str, lim: Limits) -> Result<(Vec<System>, Vec<Run>), String> {
    let sys = systems(program)?;
    let gs = graphs(data)?;
    let runs = gs.iter().map(|g| run(&sys, g, lim)).collect();
    Ok((sys, runs))
}

fn graph_or_empty(g: &Graph, width: usize) -> String {
    if g.is_empty() {
        "(no arrows: the graph is empty)\n".into()
    } else {
        graph_text(g, width)
    }
}

pub fn details(sys: &System, o: &Outcome) -> String {
    let mut s = String::new();
    let g = &o.grown;
    for (k, a) in o.applications.iter().enumerate() {
        let r = &sys.rules[a.rule];
        let path: Vec<_> = a.path.iter().map(|&i| g.arrows[i].tree.clone()).collect();
        let made: Vec<_> = a.added.iter().map(|&i| g.arrows[i].tree.clone()).collect();
        s.push_str(&format!(
            "  {:>3}. rule {} (line {}) {} == {}\n       on   {}\n",
            k + 1,
            a.rule + 1,
            r.line,
            pchain_string(&r.left),
            pchain_string(&r.right),
            chain_string(&path)
        ));
        if !a.env.0.is_empty() {
            s.push_str(&format!("       with {}\n", a.env));
        }
        s.push_str(&format!("       adds {}\n", chain_string(&made)));
    }
    s.push_str(&format!(
        "  {} applications; {} arrows added; {} removed as rewritten, {} as off every path from entry to exit\n",
        o.applications.len(),
        o.applications.iter().map(|a| a.added.len()).sum::<usize>(),
        o.rewritten.len(),
        o.stranded.len()
    ));
    s
}

pub fn printout(sys: &[System], runs: &[Run], opts: Opts) -> String {
    let mut s = String::new();
    for (gi, r) in runs.iter().enumerate() {
        if gi > 0 {
            s.push_str("\n/\n\n");
        }
        s.push_str("DONNEES\n\n");
        s.push_str(&graph_or_empty(&r.input, opts.width));
        for (pi, o) in r.phases.iter().enumerate() {
            s.push_str(&format!("\nPHASE {}\n\n", pi + 1));
            if sys[pi].det || opts.det_all {
                s.push_str(&details(&sys[pi], o));
                s.push('\n');
            }
            if !o.converged {
                s.push_str(&format!(
                    "(the phase did not converge within {} new arrows: C70 p. 22, \"le résultat est indéfini\")\n",
                    opts.limits.max_arrows
                ));
                break;
            }
            s.push_str(&graph_or_empty(&o.result, opts.width));
        }
    }
    s
}
