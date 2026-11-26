//! A treatment's run as JSON, for the playground (`qsys json`): for every
//! graph of the data and every phase, each arrow the first phase made, the
//! application that made it, and what the second phase did with it.

use crate::json::{obj, Json};
use crate::machine::{run, Graph, Limits, Outcome, System};
use crate::print::graph_text;
use crate::report::{printout, Opts};
use crate::syntax::{parse_data, parse_treatment, pchain_string};

/// Most arrows of one phase sent to the page; past this the page shows the
/// printout only.
const MAX_ARROWS: usize = 4000;

fn error(at: &str, line: usize, column: usize, message: String) -> Json {
    obj([
        ("in", at.into()),
        ("line", line.into()),
        ("column", column.into()),
        ("message", message.into()),
    ])
}

fn phase_json(sys: &System, o: &Outcome) -> Json {
    let g = &o.grown;
    let mut made_by = vec![None; g.arrows.len()];
    for (k, a) in o.applications.iter().enumerate() {
        for &id in &a.added {
            made_by[id] = Some(k);
        }
    }
    let send = g.arrows.len() <= MAX_ARROWS;
    let arrows: Vec<Json> = if send {
        g.arrows
            .iter()
            .enumerate()
            .map(|(i, a)| {
                let fate = if o.rewritten.contains(&i) {
                    "rewritten"
                } else if o.stranded.contains(&i) {
                    "stranded"
                } else {
                    "kept"
                };
                obj([
                    ("from", a.from.into()),
                    ("to", a.to.into()),
                    ("tree", a.tree.to_string().into()),
                    ("app", made_by[i].map(Json::from).unwrap_or(Json::Null)),
                    ("fate", fate.into()),
                ])
            })
            .collect()
    } else {
        Vec::new()
    };
    let apps: Vec<Json> = if send {
        o.applications
            .iter()
            .map(|a| {
                obj([
                    ("rule", a.rule.into()),
                    ("path", a.path.iter().map(|&i| Json::from(i)).collect::<Vec<_>>().into()),
                    (
                        "added",
                        a.added.iter().map(|&i| Json::from(i)).collect::<Vec<_>>().into(),
                    ),
                    ("env", a.env.to_string().into()),
                ])
            })
            .collect()
    } else {
        Vec::new()
    };
    let _ = sys;
    obj([
        ("converged", o.converged.into()),
        ("drawn", send.into()),
        ("nverts", g.nverts.into()),
        ("entry", g.entry.into()),
        ("exit", g.exit.into()),
        (
            "inputArrows",
            (g.arrows.len() - o.applications.iter().map(|a| a.added.len()).sum::<usize>()).into(),
        ),
        ("arrowCount", g.arrows.len().into()),
        ("applicationCount", o.applications.len().into()),
        ("rewritten", o.rewritten.len().into()),
        ("stranded", o.stranded.len().into()),
        ("kept", o.result.arrows.len().into()),
        ("strings", count_text(&o.result).into()),
        ("arrows", arrows.into()),
        ("applications", apps.into()),
        ("printed", graph_text(&o.result, 0).into()),
    ])
}

/// How many strings a graph carries, as text: there may be very many.
pub fn count_text(g: &Graph) -> String {
    let n = g.count_strings();
    if n == u128::MAX {
        "more than 10^38".into()
    } else {
        n.to_string()
    }
}

pub fn play(program: &str, data: &str, lim: Limits) -> Json {
    let t = match parse_treatment(program) {
        Ok(t) => t,
        Err(e) => return obj([("error", error("program", e.line, e.col, e.msg))]),
    };
    let mut systems = Vec::new();
    for p in &t.phases {
        match System::new(p) {
            Ok(s) => systems.push(s),
            Err(e) => return obj([("error", error("program", p.line, 1, e.0))]),
        }
    }
    let texts = match parse_data(data) {
        Ok(g) => g,
        Err(e) => return obj([("error", error("data", e.line, e.col, e.msg))]),
    };
    let mut graphs = Vec::new();
    for (i, gt) in texts.iter().enumerate() {
        match Graph::from_text(gt) {
            Ok(g) => graphs.push(g),
            Err(e) => return obj([("error", error("data", 1, 1, format!("graph {}: {}", i + 1, e.0)))]),
        }
    }
    let runs: Vec<_> = graphs.iter().map(|g| run(&systems, g, lim)).collect();
    let phases: Vec<Json> = t
        .phases
        .iter()
        .zip(&systems)
        .map(|(p, s)| {
            obj([
                ("line", p.line.into()),
                ("det", s.det.into()),
                ("inv", s.inv.into()),
                (
                    "comments",
                    s.comments
                        .iter()
                        .map(|c| Json::from(c.as_str()))
                        .collect::<Vec<_>>()
                        .into(),
                ),
                (
                    "rules",
                    s.rules
                        .iter()
                        .map(|r| {
                            obj([
                                ("line", r.line.into()),
                                ("text", r.to_string().into()),
                                ("left", pchain_string(&r.left).into()),
                                ("right", pchain_string(&r.right).into()),
                            ])
                        })
                        .collect::<Vec<_>>()
                        .into(),
                ),
            ])
        })
        .collect();
    let graphs_json: Vec<Json> = runs
        .iter()
        .map(|r| {
            obj([
                ("printed", graph_text(&r.input, 0).into()),
                ("strings", count_text(&r.input).into()),
                (
                    "phases",
                    r.phases
                        .iter()
                        .zip(&systems)
                        .map(|(o, s)| phase_json(s, o))
                        .collect::<Vec<_>>()
                        .into(),
                ),
            ])
        })
        .collect();
    obj([
        ("error", Json::Null),
        ("limit", lim.max_arrows.into()),
        ("phases", phases.into()),
        ("graphs", graphs_json.into()),
        (
            "printout",
            printout(
                &systems,
                &runs,
                Opts {
                    limits: lim,
                    ..Opts::default()
                },
            )
            .into(),
        ),
    ])
}
