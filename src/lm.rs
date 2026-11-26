//! **Not historical.** A graph of strings read as a language model
//! (QSYS-E16): the next-token distribution after a prefix, counted on the
//! graph by dynamic programming, without listing its strings.
//!
//! A token is the tree on an arrow, as text. The end of a string is the
//! token `END`. Two ways of counting are offered:
//!
//! - by **paths**: every path from the entry to the exit counts once, so a
//!   string carried by two paths counts twice;
//! - by **strings**: every distinct string counts once. This needs the
//!   graph made deterministic (the paths that spell the same prefix merged
//!   into one state), which is done lazily, only for the states asked.
//!
//! The graph is whatever a phase of the historical machine left; nothing
//! here changes how it was made.

use crate::machine::Graph;
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub const END: &str = "END";

pub struct Lm {
    pub graph: Graph,
    out: Vec<Vec<(String, usize)>>,
    /// Paths from each vertex to the exit (saturating).
    suffix: Vec<u128>,
    /// Distinct strings from each deterministic state.
    distinct: HashMap<Vec<usize>, u128>,
}

/// Where a prefix can stand: each vertex, with the number of paths from the
/// entry that spell the prefix and end there.
pub type State = BTreeMap<usize, u128>;

impl Lm {
    pub fn new(g: &Graph) -> Lm {
        let mut out: Vec<Vec<(String, usize)>> = vec![Vec::new(); g.nverts];
        let mut indeg = vec![0usize; g.nverts];
        for a in &g.arrows {
            out[a.from].push((a.tree.to_string(), a.to));
            indeg[a.to] += 1;
        }
        // Reverse topological order by Kahn's algorithm.
        let mut order = Vec::with_capacity(g.nverts);
        let mut stack: Vec<usize> = (0..g.nverts).filter(|&v| indeg[v] == 0).collect();
        while let Some(v) = stack.pop() {
            order.push(v);
            for (_, w) in &out[v] {
                indeg[*w] -= 1;
                if indeg[*w] == 0 {
                    stack.push(*w);
                }
            }
        }
        let mut suffix = vec![0u128; g.nverts];
        for &v in order.iter().rev() {
            suffix[v] = if v == g.exit {
                1
            } else {
                out[v].iter().fold(0u128, |s, (_, w)| s.saturating_add(suffix[*w]))
            };
        }
        Lm {
            graph: g.clone(),
            out,
            suffix,
            distinct: HashMap::new(),
        }
    }

    pub fn start(&self) -> State {
        BTreeMap::from([(self.graph.entry, 1)])
    }

    /// The state after one more token; empty if no path continues with it.
    pub fn step(&self, s: &State, token: &str) -> State {
        let mut next = State::new();
        for (&v, &w) in s {
            for (t, to) in &self.out[v] {
                if t == token {
                    let e = next.entry(*to).or_insert(0);
                    *e = e.saturating_add(w);
                }
            }
        }
        next
    }

    pub fn after(&self, prefix: &[&str]) -> State {
        prefix.iter().fold(self.start(), |s, t| self.step(&s, t))
    }

    /// Complete paths through a state: those that spell the prefix and go
    /// on to the exit.
    pub fn paths(&self, s: &State) -> u128 {
        s.iter()
            .fold(0u128, |n, (&v, &w)| n.saturating_add(w.saturating_mul(self.suffix[v])))
    }

    /// Next tokens counted by paths: for each token, the complete paths
    /// that continue the prefix with it. `END` counts the paths that stop.
    pub fn next_by_paths(&self, s: &State) -> BTreeMap<String, u128> {
        let mut counts: BTreeMap<String, u128> = BTreeMap::new();
        for (&v, &w) in s {
            if v == self.graph.exit {
                *counts.entry(END.into()).or_insert(0) += w;
            }
            for (t, to) in &self.out[v] {
                let e = counts.entry(t.clone()).or_insert(0);
                *e = e.saturating_add(w.saturating_mul(self.suffix[*to]));
            }
        }
        counts
    }

    /// Distinct strings that can follow a set of vertices.
    fn distinct_from(&mut self, set: &[usize]) -> u128 {
        if let Some(&n) = self.distinct.get(set) {
            return n;
        }
        let mut n: u128 = set.contains(&self.graph.exit) as u128;
        let mut by_token: BTreeMap<&str, BTreeSet<usize>> = BTreeMap::new();
        let out = std::mem::take(&mut self.out);
        for &v in set {
            for (t, to) in &out[v] {
                by_token.entry(t.as_str()).or_default().insert(*to);
            }
        }
        let nexts: Vec<Vec<usize>> = by_token.into_values().map(|s| s.into_iter().collect()).collect();
        self.out = out;
        for next in nexts {
            n = n.saturating_add(self.distinct_from(&next));
        }
        self.distinct.insert(set.to_vec(), n);
        n
    }

    /// Next tokens counted by distinct strings: for each token, how many
    /// different complete strings continue the prefix with it.
    pub fn next_by_strings(&mut self, s: &State) -> BTreeMap<String, u128> {
        let set: Vec<usize> = s.keys().copied().collect();
        let mut counts = BTreeMap::new();
        if set.contains(&self.graph.exit) {
            counts.insert(END.to_string(), 1);
        }
        let mut by_token: BTreeMap<String, BTreeSet<usize>> = BTreeMap::new();
        for &v in &set {
            for (t, to) in &self.out[v] {
                by_token.entry(t.clone()).or_default().insert(*to);
            }
        }
        for (t, next) in by_token {
            let next: Vec<usize> = next.into_iter().collect();
            let n = self.distinct_from(&next);
            counts.insert(t, n);
        }
        counts
    }

    /// States made deterministic so far (for measuring).
    pub fn states_made(&self) -> usize {
        self.distinct.len()
    }
}

/// Entropy in bits of a distribution given as counts.
pub fn entropy(counts: &BTreeMap<String, u128>) -> f64 {
    let total: f64 = counts.values().map(|&c| c as f64).sum();
    counts
        .values()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f64 / total;
            -p * p.log2()
        })
        .sum()
}

/// A small fixed-seed generator, so runs repeat.
pub struct Rng(pub u64);

impl Rng {
    pub fn draw(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 11
    }
    /// A token drawn in proportion to its count.
    pub fn pick(&mut self, counts: &BTreeMap<String, u128>) -> String {
        let total: u128 = counts.values().sum();
        // 53 random bits scaled to the total; exact enough for these totals.
        let mut r = ((self.draw() as u128) * total) >> 53;
        for (t, &c) in counts {
            if r < c {
                return t.clone();
            }
            r -= c;
        }
        counts.keys().last().unwrap().clone()
    }
}

/// Generate one string: greedy takes the most likely token (the first in
/// order on a tie), otherwise sample.
pub fn generate(lm: &Lm, greedy: bool, rng: &mut Rng) -> Vec<String> {
    let mut s = lm.start();
    let mut words = Vec::new();
    loop {
        let counts = lm.next_by_paths(&s);
        let t = if greedy {
            let best = counts.values().max().copied().unwrap_or(0);
            counts.iter().find(|(_, &c)| c == best).unwrap().0.clone()
        } else {
            rng.pick(&counts)
        };
        if t == END {
            return words;
        }
        s = lm.step(&s, &t);
        words.push(t);
    }
}
