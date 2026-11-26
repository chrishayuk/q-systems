//! Trees and lists (C70 section 2), and the four relations between lists
//! (C70 section 4).
//!
//! A tree is a label with a list of trees under it. The empty list `-NUL-`
//! and the disintegrator `$$` are only ways of writing: `A(-NUL-)` is `A`,
//! and `$$ABC` is `A,B,C`. They are removed when a text is read, so two
//! trees are equal exactly when they are equal as Rust values.

use std::fmt;
use std::rc::Rc;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Tree {
    pub label: Rc<str>,
    pub kids: Vec<Tree>,
}

impl Tree {
    pub fn leaf(label: &str) -> Tree {
        Tree {
            label: label.into(),
            kids: Vec::new(),
        }
    }

    pub fn node(label: &str, kids: Vec<Tree>) -> Tree {
        Tree {
            label: label.into(),
            kids,
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.kids.is_empty()
    }
}

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label)?;
        if !self.kids.is_empty() {
            f.write_str("(")?;
            write_list(f, &self.kids)?;
            f.write_str(")")?;
        }
        Ok(())
    }
}

fn write_list(f: &mut fmt::Formatter<'_>, l: &[Tree]) -> fmt::Result {
    for (i, t) in l.iter().enumerate() {
        if i > 0 {
            f.write_str(",")?;
        }
        write!(f, "{t}")?;
    }
    Ok(())
}

/// A list as C70 writes it: trees separated by commas, `-NUL-` if empty.
pub fn list_string(l: &[Tree]) -> String {
    if l.is_empty() {
        return "-NUL-".into();
    }
    l.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(",")
}

/// A string (chaîne) as C70 writes it: trees separated by ` + `.
pub fn chain_string(l: &[Tree]) -> String {
    l.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(" + ")
}

// ---- characters (C70 section 1) ----

/// The 26 capital letters. C70 p. 6: there are no small letters, "car elles
/// n'existent pas sur notre ordinateur CDC 6400".
pub fn is_letter(c: char) -> bool {
    c.is_ascii_uppercase()
}

pub fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

/// The significant signs, C70 p. 6.
pub const SIGNIFICANT: &[char] = &['+', '-', '*', '/', '(', ')', '$', '=', '≠', '.', ','];

/// The insignificant signs, C70 p. 6: "le reste des caractères disponibles
/// sur notre ordinateur". They are the rest of the CDC 6000 display code,
/// in its order; the arrow C70 draws as a hooked arrow is display code 65,
/// written `→` here.
pub const INSIGNIFICANT: &[char] = &[
    '≡', '[', ']', ':', '→', '∨', '∧', '↑', '↓', '<', '>', '≤', '≥', '¬', ';',
];

/// The CDC 6000 display code of a character (63-character set): A-Z are
/// 01-32 octal, 0-9 are 33-44, then `+ - * / ( ) $ =`, blank, `, .`, and
/// the insignificant signs. C70 p. 6 lists the characters in this order.
pub fn display_code(c: char) -> u32 {
    const ORDER: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+-*/()$= ,.≡[]:≠→∨∧↑↓<>≤≥¬;";
    ORDER
        .chars()
        .position(|d| d == c)
        .map(|p| p as u32 + 1)
        .unwrap_or(0x10000 + c as u32)
}

/// Compare two texts character by character in display code; a text
/// before any longer text it begins.
pub fn collate(a: &str, b: &str) -> std::cmp::Ordering {
    a.chars().map(display_code).cmp(b.chars().map(display_code))
}

pub fn is_character(c: char) -> bool {
    is_letter(c) || is_digit(c) || SIGNIFICANT.contains(&c) || INSIGNIFICANT.contains(&c)
}

/// A label: any character, then letters and digits (C70 p. 6).
pub fn is_label(s: &str) -> bool {
    let mut cs = s.chars();
    match cs.next() {
        Some(c) if is_character(c) => cs.all(|c| is_letter(c) || is_digit(c)),
        _ => false,
    }
}

// ---- the relations (C70 section 4, pp. 14-15) ----

/// `l1 -DANS- l2`, by the five equivalences of C70 p. 14, taken in order.
pub fn dans(l1: &[Tree], l2: &[Tree]) -> bool {
    if l1.is_empty() {
        return true; // -NUL- -DANS- l2 ≡ vrai
    }
    if l2.is_empty() {
        return false; // l1 -DANS- -NUL- ≡ l1 = -NUL-
    }
    if l1.len() > 1 {
        // a1,l1 -DANS- l2 ≡ a1 -DANS- l2 et l1 -DANS- l2
        return l1.iter().all(|a| dans(std::slice::from_ref(a), l2));
    }
    // a1 -DANS- a2,l2 ≡ a1 -DANS- a2 ou a1 -DANS- l2
    let a = &l1[0];
    l2.iter().any(|b| tree_dans(a, b))
}

/// `e1(l1) -DANS- e2(l2) ≡ e1 = e2 et l1 -DANS- l2`
fn tree_dans(a: &Tree, b: &Tree) -> bool {
    a.label == b.label && dans(&a.kids, &b.kids)
}

/// `l1 -HORS- l2`, by the five equivalences of C70 p. 15, taken in order.
/// It is not the negation of `-DANS-`: both are false when either side is
/// `-NUL-`, and `A -HORS- A(B)` is false while `A -DANS- A(B)` is true.
pub fn hors(l1: &[Tree], l2: &[Tree]) -> bool {
    if l1.is_empty() || l2.is_empty() {
        return false;
    }
    if l1.len() > 1 {
        // a1,l1 -HORS- l2 ≡ a1 -HORS- l2 et (l1 -HORS- l2 ou l1 = -NUL-)
        return hors(&l1[..1], l2) && hors(&l1[1..], l2);
    }
    let a = &l1[0];
    if l2.len() > 1 {
        // a1 -HORS- a2,l2 ≡ a1 -HORS- a2 et (a1 -HORS- l2 ou l2 = -NUL-)
        return tree_hors(a, &l2[0]) && hors(l1, &l2[1..]);
    }
    tree_hors(a, &l2[0])
}

/// `e1(l1) -HORS- e2(l2) ≡ e1 ≠ e2 ou l1 -HORS- l2`
fn tree_hors(a: &Tree, b: &Tree) -> bool {
    a.label != b.label || hors(&a.kids, &b.kids)
}
