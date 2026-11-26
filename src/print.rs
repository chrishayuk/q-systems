//! Printing a graph of strings as C70 and the TAUM printouts do: paths
//! without crossings, `-01- A + B -02-`, the vertices numbered afresh.
//!
//! The order is decisions 19 and 20: from the entry, depth first, each path
//! printed as it is reached, and a vertex's own paths printed as soon as the
//! last path into it has been; the paths leaving a vertex in the order of
//! their trees, compared in display code. This reproduces the graph of
//! C70 pp. 38-40 line for line.

use crate::machine::Graph;
use crate::tree::{chain_string, collate, Tree};
use std::cmp::Ordering;

/// Paths compared tree by tree, each tree by its text in display code.
pub fn path_order(a: &[Tree], b: &[Tree]) -> Ordering {
    for (x, y) in a.iter().zip(b) {
        match collate(&x.to_string(), &y.to_string()) {
            Ordering::Equal => continue,
            o => return o,
        }
    }
    a.len().cmp(&b.len())
}

/// A path without crossings: its first vertex, its trees, its last vertex.
pub struct Path {
    pub from: usize,
    pub trees: Vec<Tree>,
    pub to: usize,
    /// The arrows it is made of.
    pub arrows: Vec<usize>,
}

/// The graph as paths without crossings, and the vertex numbers to print
/// them with, in printing order.
pub fn layout(g: &Graph) -> (Vec<Path>, Vec<usize>) {
    if g.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let out = g.out_lists();
    let mut indeg = vec![0usize; g.nverts];
    for a in &g.arrows {
        indeg[a.to] += 1;
    }
    let junction = |v: usize| v == g.entry || v == g.exit || indeg[v] != 1 || out[v].len() != 1;
    // The paths leaving each junction, in the order of its arrows.
    let mut from: Vec<Vec<Path>> = (0..g.nverts).map(|_| Vec::new()).collect();
    for v in 0..g.nverts {
        if !junction(v) {
            continue;
        }
        for &a in &out[v] {
            let mut trees = vec![g.arrows[a].tree.clone()];
            let mut arrows = vec![a];
            let mut w = g.arrows[a].to;
            while !junction(w) {
                let b = out[w][0];
                trees.push(g.arrows[b].tree.clone());
                arrows.push(b);
                w = g.arrows[b].to;
            }
            from[v].push(Path {
                from: v,
                trees,
                to: w,
                arrows,
            });
        }
    }
    for ps in from.iter_mut() {
        ps.sort_by(|a, b| path_order(&a.trees, &b.trees));
    }
    let mut pending = vec![0usize; g.nverts];
    for ps in &from {
        for p in ps {
            pending[p.to] += 1;
        }
    }
    let mut number = vec![0usize; g.nverts];
    let mut next = 1;
    number[g.entry] = next;
    next += 1;
    let mut printed = Vec::new();
    let mut stack = vec![g.entry];
    // Depth first, by an explicit stack of vertices whose paths are being
    // printed and how far each has got.
    let mut from: Vec<std::collections::VecDeque<Path>> = from.into_iter().map(|v| v.into()).collect();
    while let Some(&v) = stack.last() {
        match from[v].pop_front() {
            None => {
                stack.pop();
            }
            Some(p) => {
                let w = p.to;
                if number[w] == 0 {
                    number[w] = next;
                    next += 1;
                }
                printed.push(p);
                pending[w] -= 1;
                if pending[w] == 0 {
                    stack.push(w);
                }
            }
        }
    }
    (printed, number)
}

fn vertex(n: usize) -> String {
    format!("-{n:02}-")
}

/// The graph, one path without crossings to a line. `width` folds long
/// lines as the line printer did, anywhere, at that many characters;
/// 0 does not fold.
pub fn graph_text(g: &Graph, width: usize) -> String {
    let (paths, number) = layout(g);
    let mut s = String::new();
    for p in &paths {
        let line = format!(
            "{} {} {}",
            vertex(number[p.from]),
            chain_string(&p.trees),
            vertex(number[p.to])
        );
        s.push_str(&fold(&line, width));
        s.push('\n');
    }
    s
}

fn fold(line: &str, width: usize) -> String {
    if width == 0 {
        return line.to_string();
    }
    let cs: Vec<char> = line.chars().collect();
    cs.chunks(width)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}
