//! **Not historical.** An open language model for the grammar of C70
//! section 11 (QSYS-E17): next-token prediction with no tree given, and no
//! bound on depth, by expanding only what the prefix needs.
//!
//! The synthesis system of pp. 35-37 is read as a grammar whose symbols are
//! the clause `PI(v,a,b)` and a preposition marker with its argument,
//! `SUJ + a`, `OBJ + a`, `PPA + a`, `DDE + a`:
//!
//! - `PI(v,a,b)` is active, `SUJ a v OBJ b`, or passive, `SUJ b EST v PPA a`;
//! - a marker before a name gives its prefix and the name;
//! - a marker before a clause gives the clause after `LE FAIT QUE`, `QUE`,
//!   `PAR LE FAIT QUE` or `DE CE QUE`, or the clause nominalised: the noun
//!   of its verb, then `DDE b`, then `PPA a`.
//!
//! This reading was made by hand from the sixteen rules. The test
//! `e17_open_model` checks that, for every tree it was tried on, it gives
//! exactly the strings the historical machine gives, path for derivation.
//!
//! An argument not yet known is drawn when it is first needed, from a
//! prior: one of the four names, or, with probability `q`, a clause whose
//! verb is one of the three and whose arguments are again unknown. Each
//! choice between two rewritings is taken with probability 1/2.

use std::collections::{BTreeMap, HashMap};

pub const END: &str = "END";
pub const NAMES: [&str; 4] = ["BEAUCEVILLE", "BENOIT", "GAETANE", "RICHARD"];
pub const VERBS: [&str; 3] = ["DETRUIT", "ETONNE", "CONSTATE"];

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Arg {
    Name(&'static str),
    Clause(&'static str, Box<Arg>, Box<Arg>),
    /// Not yet drawn; how many more levels of clause it may hold (None: no
    /// bound).
    Unknown(Option<u32>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Marker {
    Suj,
    Obj,
    Ppa,
    Dde,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Sym {
    Word(&'static str),
    Pi(&'static str, Arg, Arg),
    Mark(Marker, Arg),
    /// The whole sentence, its verb not yet drawn.
    Top(Option<u32>),
}

fn clause_prefix(m: Marker) -> &'static [&'static str] {
    match m {
        Marker::Suj => &["LE", "FAIT", "QUE"],
        Marker::Obj => &["QUE"],
        Marker::Ppa => &["PAR", "LE", "FAIT", "QUE"],
        Marker::Dde => &["DE", "CE", "QUE"],
    }
}

fn name_prefix(m: Marker) -> &'static [&'static str] {
    match m {
        Marker::Suj | Marker::Obj => &[],
        Marker::Ppa => &["PAR"],
        Marker::Dde => &["DE"],
    }
}

fn noun(v: &str) -> &'static [&'static str] {
    match v {
        "DETRUIT" => &["LA", "DESTRUCTION"],
        "ETONNE" => &["L", "ETONNEMENT"],
        _ => &["LA", "CONSTATATION"],
    }
}

/// The prior and the probability of each choice.
#[derive(Clone, Copy, Debug)]
pub struct Prior {
    /// Probability that an unknown argument is a clause.
    pub q: f64,
}

/// The rewritings of a symbol, with their probabilities. Each is a list of
/// symbols in order.
pub fn expand(s: &Sym, p: Prior) -> Vec<(f64, Vec<Sym>)> {
    let words = |ws: &[&'static str]| ws.iter().map(|w| Sym::Word(w)).collect::<Vec<_>>();
    match s {
        Sym::Word(_) => vec![],
        Sym::Top(d) => VERBS
            .iter()
            .map(|v| {
                (
                    1.0 / 3.0,
                    vec![Sym::Pi(v, Arg::Unknown(*d), Arg::Unknown(*d)), Sym::Word(".")],
                )
            })
            .collect(),
        Sym::Pi(v, a, b) => vec![
            (
                0.5,
                vec![
                    Sym::Mark(Marker::Suj, a.clone()),
                    Sym::Word(v),
                    Sym::Mark(Marker::Obj, b.clone()),
                ],
            ),
            (
                0.5,
                vec![
                    Sym::Mark(Marker::Suj, b.clone()),
                    Sym::Word("EST"),
                    Sym::Word(v),
                    Sym::Mark(Marker::Ppa, a.clone()),
                ],
            ),
        ],
        Sym::Mark(m, Arg::Unknown(d)) => {
            let deeper = d.map(|d| d.checked_sub(1));
            let q = if deeper == Some(None) { 0.0 } else { p.q };
            let inner = deeper.flatten();
            let mut out: Vec<(f64, Vec<Sym>)> = NAMES
                .iter()
                .map(|n| ((1.0 - q) / 4.0, vec![Sym::Mark(*m, Arg::Name(n))]))
                .collect();
            if q > 0.0 {
                for v in VERBS {
                    out.push((
                        q / 3.0,
                        vec![Sym::Mark(
                            *m,
                            Arg::Clause(v, Box::new(Arg::Unknown(inner)), Box::new(Arg::Unknown(inner))),
                        )],
                    ));
                }
            }
            out
        }
        Sym::Mark(m, Arg::Name(n)) => {
            let mut w = words(name_prefix(*m));
            w.push(Sym::Word(n));
            vec![(1.0, w)]
        }
        Sym::Mark(m, Arg::Clause(v, a, b)) => {
            let mut clausal = words(clause_prefix(*m));
            clausal.push(Sym::Pi(v, (**a).clone(), (**b).clone()));
            let mut nominal = words(name_prefix(*m));
            nominal.extend(words(noun(v)));
            nominal.push(Sym::Mark(Marker::Dde, (**b).clone()));
            nominal.push(Sym::Mark(Marker::Ppa, (**a).clone()));
            vec![(0.5, clausal), (0.5, nominal)]
        }
    }
}

/// A partial derivation: the symbols still to produce, the next at the end.
pub type Stack = Vec<Sym>;

/// The hypotheses consistent with a prefix: each stack with its
/// probability. `merge` keeps equal stacks as one, adding their
/// probabilities; without it every derivation is kept apart.
pub struct Model {
    pub prior: Prior,
    pub merge: bool,
    pub hyps: Vec<(Stack, f64)>,
}

impl Model {
    pub fn new(prior: Prior, depth: Option<u32>, merge: bool) -> Model {
        Model {
            prior,
            merge,
            hyps: vec![(vec![Sym::Top(depth)], 1.0)],
        }
    }

    /// Every hypothesis expanded until its next symbol is a word or it is
    /// finished. No expansion of this grammar begins with itself, so this
    /// stops.
    fn frontier(&self) -> Vec<(Stack, f64)> {
        let mut done = Vec::new();
        let mut work = self.hyps.clone();
        while let Some((mut st, pr)) = work.pop() {
            match st.last() {
                None | Some(Sym::Word(_)) => done.push((st, pr)),
                Some(s) => {
                    let s = s.clone();
                    st.pop();
                    for (p, rhs) in expand(&s, self.prior) {
                        let mut n = st.clone();
                        n.extend(rhs.into_iter().rev());
                        work.push((n, pr * p));
                    }
                }
            }
        }
        done
    }

    /// The probability of each next token (and of END), not normalised:
    /// together they sum to the probability of the prefix.
    pub fn next(&self) -> BTreeMap<String, f64> {
        let mut out: BTreeMap<String, f64> = BTreeMap::new();
        for (st, pr) in self.frontier() {
            let t = match st.last() {
                Some(Sym::Word(w)) => w.to_string(),
                _ => END.to_string(),
            };
            *out.entry(t).or_insert(0.0) += pr;
        }
        out
    }

    /// Read one token; the hypotheses that do not produce it are dropped.
    pub fn step(&mut self, token: &str) {
        let mut kept: Vec<(Stack, f64)> = Vec::new();
        let mut index: HashMap<Stack, usize> = HashMap::new();
        for (mut st, pr) in self.frontier() {
            if matches!(st.last(), Some(Sym::Word(w)) if *w == token) {
                st.pop();
                if self.merge {
                    if let Some(&i) = index.get(&st) {
                        kept[i].1 += pr;
                        continue;
                    }
                    index.insert(st.clone(), kept.len());
                }
                kept.push((st, pr));
            }
        }
        self.hyps = kept;
    }

    pub fn prob(&self) -> f64 {
        self.hyps.iter().map(|(_, p)| p).sum()
    }
}

/// Every string of a fully known tree, with the number of derivations of
/// each: what the historical machine's synthesis graph should carry as
/// paths.
pub fn strings_of(pi: &Sym) -> BTreeMap<Vec<String>, u64> {
    let mut out = BTreeMap::new();
    let mut work: Vec<(Vec<String>, Stack)> = vec![(Vec::new(), vec![Sym::Word("."), pi.clone()])];
    let p = Prior { q: 0.0 };
    while let Some((words, mut st)) = work.pop() {
        match st.pop() {
            None => *out.entry(words).or_insert(0) += 1,
            Some(Sym::Word(w)) => {
                let mut ws = words;
                ws.push(w.to_string());
                work.push((ws, st));
            }
            Some(s) => {
                for (_, rhs) in expand(&s, p) {
                    let mut n = st.clone();
                    n.extend(rhs.into_iter().rev());
                    work.push((words.clone(), n));
                }
            }
        }
    }
    out
}

/// A known argument from the text of a tree as C70 writes it: `SN(NAME)` or
/// `PH(V(VERB),a,b)`.
pub fn arg_of(t: &crate::tree::Tree) -> Arg {
    let label = |s: &str| -> &'static str {
        NAMES
            .iter()
            .chain(VERBS.iter())
            .find(|n| **n == s)
            .copied()
            .expect("a name or a verb of C70 p. 35")
    };
    match &*t.label {
        "SN" => Arg::Name(label(&t.kids[0].label)),
        "PH" => Arg::Clause(
            label(&t.kids[0].kids[0].label),
            Box::new(arg_of(&t.kids[1])),
            Box::new(arg_of(&t.kids[2])),
        ),
        l => panic!("not an argument: {l}"),
    }
}

/// The clause symbol for `PI(V(v),a,b)`.
pub fn pi_of(t: &crate::tree::Tree) -> Sym {
    let v = VERBS
        .iter()
        .find(|n| **n == &*t.kids[0].kids[0].label)
        .copied()
        .unwrap();
    Sym::Pi(v, arg_of(&t.kids[1]), arg_of(&t.kids[2]))
}

/// A small fixed-seed sampler.
pub fn sample(prior: Prior, depth: Option<u32>, seed: &mut u64) -> Vec<String> {
    let mut st: Stack = vec![Sym::Top(depth)];
    let mut words = Vec::new();
    while let Some(s) = st.pop() {
        if let Sym::Word(w) = s {
            words.push(w.to_string());
            continue;
        }
        let opts = expand(&s, prior);
        *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let mut r = (*seed >> 11) as f64 / (1u64 << 53) as f64;
        let mut pick = opts.len() - 1;
        for (i, (p, _)) in opts.iter().enumerate() {
            if r < *p {
                pick = i;
                break;
            }
            r -= p;
        }
        st.extend(opts[pick].1.iter().rev().cloned());
    }
    words
}
