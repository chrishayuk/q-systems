//! Matching parameterised trees against trees, and building trees from
//! parameters (C70 sections 3 and 8, "élimination des paramètres").
//!
//! C70 defines a rule with parameters as the infinity of rules got by
//! substituting for its parameters, and counts two substitutions as two
//! rules "même si elles sont représentées par la même suite de caractères"
//! (p. 26). So matching returns every substitution, not the first.

use crate::syntax::{Cond, PItem, PLabel, PTree, Rel, Var};
use crate::tree::{dans, hors, is_digit, is_letter, Tree};
use std::rc::Rc;

#[derive(Clone, PartialEq, Debug)]
pub enum Val {
    Label(Rc<str>),
    Tree(Tree),
    List(Vec<Tree>),
}

/// A substitution: parameters and what stands for them.
#[derive(Clone, Default, PartialEq, Debug)]
pub struct Env(pub Vec<(Var, Val)>);

impl Env {
    pub fn get(&self, v: Var) -> Option<&Val> {
        self.0.iter().find(|(w, _)| *w == v).map(|(_, x)| x)
    }

    fn with(&self, v: Var, x: Val) -> Env {
        let mut e = self.clone();
        e.0.push((v, x));
        e
    }
}

impl std::fmt::Display for Env {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut v: Vec<_> = self.0.iter().collect();
        v.sort_by_key(|(w, _)| (w.letter, w.index));
        for (i, (w, x)) in v.into_iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            let s = match x {
                Val::Label(l) => l.to_string(),
                Val::Tree(t) => t.to_string(),
                Val::List(l) => crate::tree::list_string(l),
            };
            write!(f, "{w} = {s}")?;
        }
        Ok(())
    }
}

/// Every substitution that makes the string of patterns equal to the string
/// of trees, extending `env`.
pub fn match_chain(pats: &[PTree], trees: &[&Tree], env: &Env, out: &mut Vec<Env>) {
    if pats.len() != trees.len() {
        return;
    }
    if pats.is_empty() {
        out.push(env.clone());
        return;
    }
    let mut first = Vec::new();
    match_tree(&pats[0], trees[0], env, &mut first);
    for e in first {
        match_chain(&pats[1..], &trees[1..], &e, out);
    }
}

pub fn match_tree(p: &PTree, t: &Tree, env: &Env, out: &mut Vec<Env>) {
    match p {
        PTree::Var(v) => match env.get(*v) {
            Some(Val::Tree(u)) => {
                if u == t {
                    out.push(env.clone())
                }
            }
            Some(_) => unreachable!("a tree parameter bound to something else"),
            None => out.push(env.with(*v, Val::Tree(t.clone()))),
        },
        PTree::Node(lab, items) => {
            let env = match lab {
                PLabel::Lit(l) => {
                    if **l != *t.label {
                        return;
                    }
                    env.clone()
                }
                PLabel::Var(v) => match env.get(*v) {
                    Some(Val::Label(l)) => {
                        if *l != t.label {
                            return;
                        }
                        env.clone()
                    }
                    Some(_) => unreachable!(),
                    None => env.with(*v, Val::Label(t.label.clone())),
                },
            };
            match_list(items, &t.kids, &env, out);
        }
    }
}

/// The characters of a label, as leaves.
pub fn disintegrate(l: &str) -> Vec<Tree> {
    l.chars().map(|c| Tree::leaf(&c.to_string())).collect()
}

/// The label whose characters these leaves are, if there is one (C70 p. 9:
/// `$$e` stands for `c1,c2,…,cn` "à condition que … c1c2…cn soit une
/// étiquette").
fn integrate(ts: &[Tree]) -> Option<Rc<str>> {
    if ts.is_empty() {
        return None;
    }
    let mut s = String::new();
    for (i, t) in ts.iter().enumerate() {
        if !t.is_leaf() || t.label.chars().count() != 1 {
            return None;
        }
        let c = t.label.chars().next().unwrap();
        if i > 0 && !(is_letter(c) || is_digit(c)) {
            return None;
        }
        s.push(c);
    }
    Some(s.into())
}

pub fn match_list(items: &[PItem], ts: &[Tree], env: &Env, out: &mut Vec<Env>) {
    let Some((first, rest)) = items.split_first() else {
        if ts.is_empty() {
            out.push(env.clone());
        }
        return;
    };
    match first {
        PItem::Tree(p) => {
            if let Some((t, more)) = ts.split_first() {
                let mut es = Vec::new();
                match_tree(p, t, env, &mut es);
                for e in es {
                    match_list(rest, more, &e, out);
                }
            }
        }
        PItem::List(v) => match env.get(*v) {
            Some(Val::List(l)) => {
                if ts.len() >= l.len() && ts[..l.len()] == l[..] {
                    match_list(rest, &ts[l.len()..], env, out);
                }
            }
            Some(_) => unreachable!(),
            None => {
                for k in 0..=ts.len() {
                    let e = env.with(*v, Val::List(ts[..k].to_vec()));
                    match_list(rest, &ts[k..], &e, out);
                }
            }
        },
        PItem::Dis(lab) => {
            let known: Option<Rc<str>> = match lab {
                PLabel::Lit(l) => Some(l.clone()),
                PLabel::Var(v) => match env.get(*v) {
                    Some(Val::Label(l)) => Some(l.clone()),
                    Some(_) => unreachable!(),
                    None => None,
                },
            };
            match known {
                Some(l) => {
                    let cs = disintegrate(&l);
                    if ts.len() >= cs.len() && ts[..cs.len()] == cs[..] {
                        match_list(rest, &ts[cs.len()..], env, out);
                    }
                }
                None => {
                    let PLabel::Var(v) = lab else { unreachable!() };
                    for k in 1..=ts.len() {
                        match integrate(&ts[..k]) {
                            Some(l) => {
                                let e = env.with(*v, Val::Label(l));
                                match_list(rest, &ts[k..], &e, out);
                            }
                            None => break,
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------- building

pub fn build_tree(p: &PTree, env: &Env) -> Tree {
    match p {
        PTree::Var(v) => match env.get(*v) {
            Some(Val::Tree(t)) => t.clone(),
            _ => panic!("{v} has no value"),
        },
        PTree::Node(lab, items) => Tree {
            label: build_label(lab, env),
            kids: build_list(items, env),
        },
    }
}

fn build_label(l: &PLabel, env: &Env) -> Rc<str> {
    match l {
        PLabel::Lit(s) => s.clone(),
        PLabel::Var(v) => match env.get(*v) {
            Some(Val::Label(s)) => s.clone(),
            _ => panic!("{v} has no value"),
        },
    }
}

pub fn build_list(items: &[PItem], env: &Env) -> Vec<Tree> {
    let mut out = Vec::new();
    for i in items {
        match i {
            PItem::Tree(p) => out.push(build_tree(p, env)),
            PItem::List(v) => match env.get(*v) {
                Some(Val::List(l)) => out.extend(l.iter().cloned()),
                _ => panic!("{v} has no value"),
            },
            PItem::Dis(l) => out.extend(disintegrate(&build_label(l, env))),
        }
    }
    out
}

// ---------------------------------------------------------------- conditions

/// The value of a condition under a substitution (C70 p. 27).
pub fn holds(c: &Cond, env: &Env) -> bool {
    match c {
        Cond::Or(a, b) => holds(a, env) || holds(b, env),
        Cond::And(a, b) => holds(a, env) && holds(b, env),
        Cond::Not(a) => !holds(a, env),
        Cond::Paren(a) => holds(a, env),
        Cond::Rel(l, r, m) => {
            let l = build_list(l, env);
            let m = build_list(m, env);
            match r {
                Rel::Eq => l == m,
                Rel::Ne => l != m,
                Rel::Dans => dans(&l, &m),
                Rel::Hors => hors(&l, &m),
            }
        }
    }
}

// ---------------------------------------------------------------- parameters

pub fn vars_chain(c: &[PTree], out: &mut Vec<Var>) {
    for t in c {
        vars_tree(t, out);
    }
}

pub fn vars_tree(t: &PTree, out: &mut Vec<Var>) {
    match t {
        PTree::Var(v) => out.push(*v),
        PTree::Node(l, items) => {
            if let PLabel::Var(v) = l {
                out.push(*v);
            }
            vars_list(items, out);
        }
    }
}

pub fn vars_list(items: &[PItem], out: &mut Vec<Var>) {
    for i in items {
        match i {
            PItem::Tree(t) => vars_tree(t, out),
            PItem::List(v) => out.push(*v),
            PItem::Dis(PLabel::Var(v)) => out.push(*v),
            PItem::Dis(_) => {}
        }
    }
}

pub fn vars_cond(c: &Cond, out: &mut Vec<Var>) {
    match c {
        Cond::Or(a, b) | Cond::And(a, b) => {
            vars_cond(a, out);
            vars_cond(b, out);
        }
        Cond::Not(a) | Cond::Paren(a) => vars_cond(a, out),
        Cond::Rel(l, _, m) => {
            vars_list(l, out);
            vars_list(m, out);
        }
    }
}
