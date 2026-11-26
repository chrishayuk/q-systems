//! Graphs of strings (C70 section 6) and the application of a Q-system to
//! one (C70 section 7): first arrows are added, until no rule applies to a
//! path it has not been applied to; then the arrows of every path a rule
//! was applied to are removed, and with them every arrow no longer on a
//! path from the entry to the exit.

use crate::matching::{build_tree, holds, match_chain, vars_chain, vars_cond, Env};
use crate::syntax::{GraphText, Phase, Rule, Var};
use crate::tree::{chain_string, Tree};
use std::collections::{BTreeSet, HashMap};
use std::fmt;

#[derive(Clone, Debug)]
pub struct Arrow {
    pub from: usize,
    pub to: usize,
    pub tree: Tree,
}

/// A graph of strings. Vertices are numbers; arrows are kept in the order
/// they were made.
#[derive(Clone, Debug)]
pub struct Graph {
    pub arrows: Vec<Arrow>,
    pub nverts: usize,
    pub entry: usize,
    pub exit: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphError(pub String);

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for GraphError {}

impl Graph {
    /// The graph a text describes, checked against the two conditions of
    /// C70 p. 17 and the order of p. 20.
    pub fn from_text(g: &GraphText) -> Result<Graph, GraphError> {
        if g.paths.is_empty() {
            return Err(GraphError("a graph of strings has at least one arrow".into()));
        }
        // C70 p. 20: "quels que soient les entiers p et q, ip = jq entraîne
        // p > q": a path from a vertex is written after every path into it.
        for (p, (ip, _, _)) in g.paths.iter().enumerate() {
            for (q, (_, _, jq)) in g.paths.iter().enumerate() {
                if ip == jq && p <= q {
                    return Err(GraphError(format!(
                        "path {} starts at -{ip}-, which path {} ends at; C70 p. 20 requires every path into a vertex to be written before any path from it (so there is no circuit)",
                        p + 1,
                        q + 1
                    )));
                }
            }
        }
        let mut ids: HashMap<u32, usize> = HashMap::new();
        let mut nverts = 0;
        let mut id = |n: u32, nverts: &mut usize| -> usize {
            *ids.entry(n).or_insert_with(|| {
                *nverts += 1;
                *nverts - 1
            })
        };
        let mut arrows = Vec::new();
        for (a, chain, b) in &g.paths {
            let start = id(*a, &mut nverts);
            let end = id(*b, &mut nverts);
            let mut cur = start;
            for (k, t) in chain.iter().enumerate() {
                let next = if k + 1 == chain.len() {
                    end
                } else {
                    nverts += 1;
                    nverts - 1
                };
                arrows.push(Arrow {
                    from: cur,
                    to: next,
                    tree: t.clone(),
                });
                cur = next;
            }
        }
        let mut indeg = vec![0; nverts];
        let mut outdeg = vec![0; nverts];
        for a in &arrows {
            outdeg[a.from] += 1;
            indeg[a.to] += 1;
        }
        let entries: Vec<_> = (0..nverts).filter(|&v| indeg[v] == 0).collect();
        let exits: Vec<_> = (0..nverts).filter(|&v| outdeg[v] == 0).collect();
        if entries.len() != 1 || exits.len() != 1 {
            return Err(GraphError(format!(
                "a graph of strings has one entry and one exit (C70 p. 17); this one has {} vertices with no arrow in and {} with no arrow out",
                entries.len(),
                exits.len()
            )));
        }
        Ok(Graph {
            arrows,
            nverts,
            entry: entries[0],
            exit: exits[0],
        })
    }

    /// The single path carrying a string, from a new entry to a new exit.
    pub fn from_chain(chain: &[Tree]) -> Graph {
        Graph::from_text(&GraphText {
            paths: vec![(0, chain.to_vec(), 1)],
        })
        .unwrap()
    }

    /// Every string carried by a path from the entry to the exit, in the
    /// order of the printed graph. There may be very many.
    pub fn strings(&self, limit: usize) -> Vec<Vec<Tree>> {
        let out = self.out_lists();
        let mut res = Vec::new();
        let mut stack: Vec<&Tree> = Vec::new();
        fn go<'a>(
            g: &'a Graph,
            v: usize,
            out: &[Vec<usize>],
            stack: &mut Vec<&'a Tree>,
            res: &mut Vec<Vec<Tree>>,
            limit: usize,
        ) {
            if res.len() >= limit {
                return;
            }
            if v == g.exit {
                res.push(stack.iter().map(|t| (*t).clone()).collect());
                return;
            }
            for &a in &out[v] {
                stack.push(&g.arrows[a].tree);
                go(g, g.arrows[a].to, out, stack, res, limit);
                stack.pop();
            }
        }
        go(self, self.entry, &out, &mut stack, &mut res, limit);
        res
    }

    /// For each vertex, the arrows leaving it, most recently made first.
    /// The printer sorts them (decision 20).
    pub fn out_lists(&self) -> Vec<Vec<usize>> {
        let mut out = vec![Vec::new(); self.nverts];
        for (i, a) in self.arrows.iter().enumerate().rev() {
            out[a.from].push(i);
        }
        out
    }

    pub fn is_empty(&self) -> bool {
        self.arrows.is_empty()
    }
}

// ---------------------------------------------------------------- systems

/// A system ready to apply: its rules, inverted if the phase says `-INV-`.
#[derive(Clone, Debug)]
pub struct System {
    pub rules: Vec<Rule>,
    pub det: bool,
    pub inv: bool,
    pub comments: Vec<String>,
}

impl System {
    /// Check C70's restriction on parameters (p. 27, and p. 33 when the
    /// system is inverted), and invert the rules if asked.
    pub fn new(p: &Phase) -> Result<System, GraphError> {
        let rules: Vec<Rule> = if p.inv {
            p.rules.iter().map(|r| r.inverted()).collect()
        } else {
            p.rules.clone()
        };
        for r in &rules {
            let mut left = Vec::new();
            vars_chain(&r.left, &mut left);
            let mut other: Vec<Var> = Vec::new();
            vars_chain(&r.right, &mut other);
            if let Some(c) = &r.cond {
                vars_cond(c, &mut other);
            }
            if let Some(v) = other.iter().find(|v| !left.contains(v)) {
                let (this, that) = if p.inv {
                    ("left member or condition", "right member")
                } else {
                    ("right member or condition", "left member")
                };
                return Err(GraphError(format!(
                    "line {}: {v} appears in the {this} of the rule but not in its {that} (C70 p. {})",
                    r.line,
                    if p.inv { 33 } else { 27 }
                )));
            }
        }
        Ok(System {
            rules,
            det: p.det,
            inv: p.inv,
            comments: p.comments.clone(),
        })
    }

    fn max_left(&self) -> usize {
        self.rules.iter().map(|r| r.left.len()).max().unwrap_or(0)
    }
}

/// One application of one rule (with one substitution) to one path.
#[derive(Clone, Debug)]
pub struct Application {
    pub rule: usize,
    pub env: Env,
    /// The arrows of the path it was applied to.
    pub path: Vec<usize>,
    /// The arrows it added.
    pub added: Vec<usize>,
}

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// The most arrows the first phase may add before the process is
    /// taken not to converge.
    pub max_arrows: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Limits { max_arrows: 200_000 }
    }
}

/// What applying a system to a graph did.
#[derive(Clone, Debug)]
pub struct Outcome {
    /// The graph after the first phase: every arrow, old and new.
    pub grown: Graph,
    pub applications: Vec<Application>,
    /// Arrows removed because a rule was applied to a path they are on.
    pub rewritten: BTreeSet<usize>,
    /// Arrows removed because they were no longer on a path from the
    /// entry to the exit.
    pub stranded: BTreeSet<usize>,
    /// The graph after the second phase.
    pub result: Graph,
    /// False if the first phase was stopped by the limit: C70 p. 22, "si
    /// le processus ne converge pas le résultat est indéfini".
    pub converged: bool,
}

/// Paths of up to `n` arrows ending at vertex `v`, using arrows before `before`.
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

/// Apply a system to a graph (C70 section 7).
///
/// Every path is considered once, when the last of its arrows to be made
/// is reached; so no rule is applied twice to the same path with the same
/// substitution, and every path, old arrows and new, is tried.
pub fn apply(sys: &System, g: &Graph, lim: Limits) -> Outcome {
    let mut arrows = g.arrows.clone();
    let mut nverts = g.nverts;
    let mut inn: Vec<Vec<usize>> = vec![Vec::new(); nverts];
    let mut out_l: Vec<Vec<usize>> = vec![Vec::new(); nverts];
    for (i, a) in arrows.iter().enumerate() {
        out_l[a.from].push(i);
        inn[a.to].push(i);
    }
    let maxl = sys.max_left();
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
        // Shorter paths first, then left to right.
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
                    let mut added = Vec::new();
                    let mut cur = start;
                    for (k, t) in new.into_iter().enumerate() {
                        let next = if k + 1 == rule.right.len() {
                            end
                        } else {
                            nverts += 1;
                            inn.push(Vec::new());
                            out_l.push(Vec::new());
                            nverts - 1
                        };
                        let id = arrows.len();
                        arrows.push(Arrow {
                            from: cur,
                            to: next,
                            tree: t,
                        });
                        out_l[cur].push(id);
                        inn[next].push(id);
                        added.push(id);
                        cur = next;
                    }
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
    // Second phase.
    let rewritten: BTreeSet<usize> = apps.iter().flat_map(|a| a.path.iter().copied()).collect();
    let alive: Vec<bool> = (0..arrows.len()).map(|a| !rewritten.contains(&a)).collect();
    let mut fwd = vec![false; nverts];
    let mut bwd = vec![false; nverts];
    fwd[g.entry] = true;
    bwd[g.exit] = true;
    // Arrows only ever go from older structure to newer vertices or to old
    // ones; there is no circuit, so iterate to a fixed point.
    let mut changed = true;
    while changed {
        changed = false;
        for (k, a) in arrows.iter().enumerate() {
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
    let mut stranded = BTreeSet::new();
    let mut kept = Vec::new();
    for (k, a) in arrows.iter().enumerate() {
        if !alive[k] {
            continue;
        }
        if fwd[a.from] && bwd[a.to] {
            kept.push(a.clone());
        } else {
            stranded.insert(k);
        }
    }
    let result = Graph {
        arrows: kept,
        nverts,
        entry: g.entry,
        exit: g.exit,
    };
    Outcome {
        grown,
        applications: apps,
        rewritten,
        stranded,
        result,
        converged,
    }
}

/// The result of a whole treatment on one graph.
#[derive(Clone, Debug)]
pub struct Run {
    pub input: Graph,
    pub phases: Vec<Outcome>,
}

impl Run {
    pub fn result(&self) -> &Graph {
        self.phases.last().map(|o| &o.result).unwrap_or(&self.input)
    }
}

/// Apply the phases of a treatment one after the other (C70 section 10).
/// A phase that does not converge ends the treatment.
pub fn run(systems: &[System], g: &Graph, lim: Limits) -> Run {
    let mut phases: Vec<Outcome> = Vec::new();
    let mut cur = g.clone();
    for s in systems {
        let o = apply(s, &cur, lim);
        cur = o.result.clone();
        let done = !o.converged;
        phases.push(o);
        if done {
            break;
        }
    }
    Run {
        input: g.clone(),
        phases,
    }
}

/// The strings of a graph, one per line, for tests.
pub fn strings_text(g: &Graph) -> Vec<String> {
    g.strings(usize::MAX).iter().map(|s| chain_string(s)).collect()
}
