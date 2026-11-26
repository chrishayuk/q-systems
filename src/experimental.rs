//! Variants of the machine that are **not** Colmerauer's, for the
//! experiments of docs/research.md (QSYS-E11 to E13). The historical
//! machine is `machine::apply`, and nothing here changes it.
//!
//! Two things may be varied:
//!
//! - **Merging.** C70 p. 22 adds "le tout nouveau chemin" for every
//!   application, so two applications that make the same thing make two
//!   copies of it. `Merge::Intern` reuses what is already there instead: an
//!   arrow with the same ends and tree, or a path made earlier with the same
//!   ends and trees. `Merge::Keep` merges in the same way but remembers every
//!   application that made each arrow, so the derivations can still be
//!   counted (`derivations`).
//! - **Purifying.** C70's second phase removes every *arrow* on a path a rule
//!   was applied to. `Purify::Paths` instead keeps every string from the entry
//!   to the exit that does not run through such a path. To say "this string
//!   but not that one" of strings that share arrows, the graph has to be
//!   unshared: its vertices become pairs of a vertex and the arrows just
//!   crossed.
//!
//! With `Merge::None` and `Purify::Arrows` this is the historical machine,
//! and a test checks that it gives the same graphs.

use crate::machine::{Application, Arrow, Graph, Limits, System};
use crate::matching::{build_tree, holds, match_chain, Env};
use crate::tree::Tree;
use std::collections::{BTreeSet, HashMap};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Merge {
    /// C70: every application adds a new path.
    None,
    /// Reuse an equal arrow or an equal made path; remember one maker.
    Intern,
    /// Reuse, and remember every application that made each arrow.
    Keep,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Purify {
    /// C70: remove the arrows of every rewritten path, then stranded arrows.
    Arrows,
    /// Keep every entry-to-exit string that does not run through a
    /// rewritten path, unsharing the graph as needed.
    Paths,
}

#[derive(Clone, Copy, Debug)]
pub struct Variant {
    pub merge: Merge,
    pub purify: Purify,
}

pub const C70: Variant = Variant {
    merge: Merge::None,
    purify: Purify::Arrows,
};

#[derive(Clone, Debug)]
pub struct Outcome {
    pub grown: Graph,
    pub applications: Vec<Application>,
    /// For each arrow of `grown`, the applications that made it (none for
    /// the arrows of the data). Under `Merge::Intern`, at most one.
    pub makers: Vec<Vec<usize>>,
    pub result: Graph,
    /// Under `Purify::Arrows`, the arrow of `grown` each arrow of `result`
    /// is.
    pub kept: Vec<usize>,
    pub converged: bool,
}

fn paths_into(
    v: usize,
    n: usize,
    before: usize,
    inn: &[Vec<usize>],
    arrows: &[Arrow],
    cur: &mut Vec<usize>,
    out: &mut Vec<Vec<usize>>,
) {
    out.push(cur.iter().rev().copied().collect());
    if n == 0 {
        return;
    }
    for &a in &inn[v] {
        if a < before {
            cur.push(a);
            paths_into(arrows[a].from, n - 1, before, inn, arrows, cur, out);
            cur.pop();
        }
    }
}

fn paths_from(
    v: usize,
    n: usize,
    before: usize,
    out_l: &[Vec<usize>],
    arrows: &[Arrow],
    cur: &mut Vec<usize>,
    out: &mut Vec<Vec<usize>>,
) {
    out.push(cur.clone());
    if n == 0 {
        return;
    }
    for &a in &out_l[v] {
        if a < before {
            cur.push(a);
            paths_from(arrows[a].to, n - 1, before, out_l, arrows, cur, out);
            cur.pop();
        }
    }
}

type PathKey = (usize, usize, Vec<String>);

/// Apply a system with the given variant. The first phase is the historical
/// one (each path once, at its newest arrow, every rule, every
/// substitution) except for what `merge` reuses.
pub fn apply(sys: &System, g: &Graph, lim: Limits, v: Variant) -> Outcome {
    let mut arrows = g.arrows.clone();
    let mut nverts = g.nverts;
    let mut inn: Vec<Vec<usize>> = vec![Vec::new(); nverts];
    let mut out_l: Vec<Vec<usize>> = vec![Vec::new(); nverts];
    let mut makers: Vec<Vec<usize>> = vec![Vec::new(); arrows.len()];
    let mut by_arrow: HashMap<(usize, usize, String), usize> = HashMap::new();
    let mut by_path: HashMap<PathKey, Vec<usize>> = HashMap::new();
    for (i, a) in arrows.iter().enumerate() {
        out_l[a.from].push(i);
        inn[a.to].push(i);
        by_arrow.entry((a.from, a.to, a.tree.to_string())).or_insert(i);
    }
    let maxl = sys.rules.iter().map(|r| r.left.len()).max().unwrap_or(0);
    let mut apps: Vec<Application> = Vec::new();
    let mut converged = true;
    let mut i = 0;
    while i < arrows.len() && maxl > 0 {
        if arrows.len() > g.arrows.len() + lim.max_arrows {
            converged = false;
            break;
        }
        let mut lefts = Vec::new();
        paths_into(arrows[i].from, maxl - 1, i, &inn, &arrows, &mut Vec::new(), &mut lefts);
        let mut rights = Vec::new();
        paths_from(arrows[i].to, maxl - 1, i, &out_l, &arrows, &mut Vec::new(), &mut rights);
        let mut paths: Vec<Vec<usize>> = Vec::new();
        for l in &lefts {
            for r in &rights {
                if l.len() + 1 + r.len() <= maxl {
                    let mut p = l.clone();
                    p.push(i);
                    p.extend(r);
                    paths.push(p);
                }
            }
        }
        for path in paths {
            let owned: Vec<Tree> = path.iter().map(|&a| arrows[a].tree.clone()).collect();
            let trees: Vec<&Tree> = owned.iter().collect();
            for (ri, rule) in sys.rules.iter().enumerate() {
                if rule.left.len() != path.len() {
                    continue;
                }
                let mut envs = Vec::new();
                match_chain(&rule.left, &trees, &Env::default(), &mut envs);
                for env in envs {
                    if let Some(c) = &rule.cond {
                        if !holds(c, &env) {
                            continue;
                        }
                    }
                    let start = arrows[path[0]].from;
                    let end = arrows[*path.last().unwrap()].to;
                    let new: Vec<Tree> = rule.right.iter().map(|p| build_tree(p, &env)).collect();
                    let app = apps.len();
                    let reuse = match v.merge {
                        Merge::None => None,
                        _ if new.len() == 1 => by_arrow.get(&(start, end, new[0].to_string())).map(|&a| vec![a]),
                        _ => by_path
                            .get(&(start, end, new.iter().map(|t| t.to_string()).collect()))
                            .cloned(),
                    };
                    let added = if let Some(ids) = reuse {
                        if v.merge == Merge::Keep {
                            for &a in &ids {
                                makers[a].push(app);
                            }
                        }
                        ids
                    } else {
                        let key: PathKey = (start, end, new.iter().map(|t| t.to_string()).collect());
                        let len = new.len();
                        let mut added = Vec::new();
                        let mut cur = start;
                        for (k, t) in new.into_iter().enumerate() {
                            let next = if k + 1 == len {
                                end
                            } else {
                                nverts += 1;
                                inn.push(Vec::new());
                                out_l.push(Vec::new());
                                nverts - 1
                            };
                            let id = arrows.len();
                            by_arrow.entry((cur, next, t.to_string())).or_insert(id);
                            arrows.push(Arrow {
                                from: cur,
                                to: next,
                                tree: t,
                            });
                            makers.push(vec![app]);
                            out_l[cur].push(id);
                            inn[next].push(id);
                            added.push(id);
                            cur = next;
                        }
                        by_path.entry(key).or_insert_with(|| added.clone());
                        added
                    };
                    apps.push(Application {
                        rule: ri,
                        env,
                        path: path.clone(),
                        added,
                    });
                }
            }
        }
        i += 1;
    }
    let grown = Graph {
        arrows: arrows.clone(),
        nverts,
        entry: g.entry,
        exit: g.exit,
    };
    let (result, kept) = match v.purify {
        Purify::Arrows => purify_arrows(&grown, &apps),
        Purify::Paths => (purify_paths(&grown, &apps, maxl), Vec::new()),
    };
    Outcome {
        grown,
        applications: apps,
        makers,
        result,
        kept,
        converged,
    }
}

/// Only the arrows on some path from the entry to the exit, with no vertex
/// numbers changed.
fn trim(g: &Graph, alive: &[bool]) -> (Graph, Vec<usize>) {
    let mut fwd = vec![false; g.nverts];
    let mut bwd = vec![false; g.nverts];
    fwd[g.entry] = true;
    bwd[g.exit] = true;
    let mut changed = true;
    while changed {
        changed = false;
        for (k, a) in g.arrows.iter().enumerate() {
            if !alive[k] {
                continue;
            }
            if fwd[a.from] && !fwd[a.to] {
                fwd[a.to] = true;
                changed = true;
            }
            if bwd[a.to] && !bwd[a.from] {
                bwd[a.from] = true;
                changed = true;
            }
        }
    }
    let kept: Vec<usize> = (0..g.arrows.len())
        .filter(|&k| alive[k] && fwd[g.arrows[k].from] && bwd[g.arrows[k].to])
        .collect();
    let graph = Graph {
        arrows: kept.iter().map(|&k| g.arrows[k].clone()).collect(),
        nverts: g.nverts,
        entry: g.entry,
        exit: g.exit,
    };
    (graph, kept)
}

fn purify_arrows(g: &Graph, apps: &[Application]) -> (Graph, Vec<usize>) {
    let rewritten: BTreeSet<usize> = apps.iter().flat_map(|a| a.path.iter().copied()).collect();
    let alive: Vec<bool> = (0..g.arrows.len()).map(|a| !rewritten.contains(&a)).collect();
    trim(g, &alive)
}

/// Every string from the entry to the exit that has no rewritten path in
/// it. A vertex of the result is a vertex of `g` and the last arrows (at
/// most `maxl - 1`) crossed to reach it; an arrow is allowed unless it
/// completes a rewritten path.
fn purify_paths(g: &Graph, apps: &[Application], maxl: usize) -> Graph {
    let forbidden: BTreeSet<Vec<usize>> = apps.iter().map(|a| a.path.clone()).collect();
    let keep = maxl.saturating_sub(1);
    let mut out_l: Vec<Vec<usize>> = vec![Vec::new(); g.nverts];
    for (i, a) in g.arrows.iter().enumerate() {
        out_l[a.from].push(i);
    }
    let mut ids: HashMap<(usize, Vec<usize>), usize> = HashMap::new();
    let mut states: Vec<(usize, Vec<usize>)> = vec![(g.entry, Vec::new())];
    ids.insert((g.entry, Vec::new()), 0);
    let mut arrows = Vec::new();
    let mut exits = Vec::new();
    let mut k = 0;
    while k < states.len() {
        let (v, hist) = states[k].clone();
        if v == g.exit {
            exits.push(k);
        }
        for &a in &out_l[v] {
            let mut h = hist.clone();
            h.push(a);
            if (0..h.len()).any(|s| forbidden.contains(&h[s..])) {
                continue;
            }
            if h.len() > keep {
                h.drain(..h.len() - keep);
            }
            let to = g.arrows[a].to;
            let key = (to, h);
            let id = *ids.entry(key.clone()).or_insert_with(|| {
                states.push(key);
                states.len() - 1
            });
            arrows.push(Arrow {
                from: k,
                to: id,
                tree: g.arrows[a].tree.clone(),
            });
        }
        k += 1;
    }
    // The exit may have been reached in several states; join them in one.
    let exit = states.len();
    for a in arrows.iter_mut() {
        if exits.contains(&a.to) {
            a.to = exit;
        }
    }
    let g2 = Graph {
        arrows,
        nverts: states.len() + 1,
        entry: 0,
        exit,
    };
    let alive = vec![true; g2.arrows.len()];
    trim(&g2, &alive).0
}

/// How many derivations an arrow has: 1 for an arrow of the data, and for a
/// made arrow the sum over its makers of the product of the derivations of
/// the path each was applied to. Under `Merge::None` every made arrow has
/// one maker; under `Merge::Keep` the count is what `Merge::None` would have
/// made copies of.
pub fn derivations(o: &Outcome, arrow: usize) -> u128 {
    let mut memo: HashMap<usize, u128> = HashMap::new();
    fn go(o: &Outcome, a: usize, memo: &mut HashMap<usize, u128>) -> u128 {
        if let Some(&n) = memo.get(&a) {
            return n;
        }
        let n = if o.makers[a].is_empty() {
            1
        } else {
            o.makers[a]
                .iter()
                .map(|&m| {
                    o.applications[m]
                        .path
                        .iter()
                        .map(|&p| go(o, p, memo))
                        .fold(1u128, |x, y| x.saturating_mul(y))
                })
                .fold(0u128, |x, y| x.saturating_add(y))
        };
        memo.insert(a, n);
        n
    }
    go(o, arrow, &mut memo)
}
