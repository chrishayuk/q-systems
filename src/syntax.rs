//! Reading Q-system texts: a treatment (traitement-q) and its data, by the
//! grammar of C70's appendix (pp. 43-45).
//!
//! The grammar is written on characters, not on tokens, and a label may
//! begin with any character at all: `+`, `(`, `,`, `.` and `/` are all
//! labels where a tree is expected (C70 p. 7 gives `+` and `(` as correct
//! labels). So the reader works on characters and decides what a character
//! is by where it stands. See docs/decisions.md, rules 3 to 5.

use crate::tree::{is_character, is_digit, is_label, is_letter, Tree};
use std::fmt;
use std::rc::Rc;

// ---------------------------------------------------------------- the tree

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Var {
    pub letter: char,
    pub index: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Label,
    Tree,
    List,
}

impl Var {
    pub fn kind(&self) -> Kind {
        match self.letter {
            'A'..='F' => Kind::Label,
            'I'..='N' => Kind::Tree,
            _ => Kind::List,
        }
    }
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.index == 0 {
            write!(f, "{}*", self.letter)
        } else {
            write!(f, "{}*{}", self.letter, self.index)
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum PLabel {
    Lit(Rc<str>),
    Var(Var),
}

/// A parameterised tree (arbre paramétré).
#[derive(Clone, PartialEq, Debug)]
pub enum PTree {
    /// A tree parameter, `I*` to `N*`.
    Var(Var),
    Node(PLabel, Vec<PItem>),
}

/// A simple parameterised list (liste simple paramétrée). `-NUL-` is
/// dropped when it is read.
#[derive(Clone, PartialEq, Debug)]
pub enum PItem {
    Tree(PTree),
    /// A list parameter, `U*` to `Z*`.
    List(Var),
    /// `$$` and a label: the label's characters, as a list of leaves.
    Dis(PLabel),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Rel {
    Eq,
    Ne,
    Dans,
    Hors,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Cond {
    Or(Box<Cond>, Box<Cond>),
    And(Box<Cond>, Box<Cond>),
    Not(Box<Cond>),
    /// `(. … .)`, kept so that the condition prints as it was written.
    Paren(Box<Cond>),
    Rel(Vec<PItem>, Rel, Vec<PItem>),
}

#[derive(Clone, Debug)]
pub struct Rule {
    pub left: Vec<PTree>,
    pub right: Vec<PTree>,
    pub cond: Option<Cond>,
    /// The line of the source the rule starts on.
    pub line: usize,
    /// Where the rule is in the source, in characters: from its first
    /// character to just after its full stop.
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug)]
pub struct Phase {
    pub det: bool,
    pub inv: bool,
    pub rules: Vec<Rule>,
    /// The comments of the phase, in order.
    pub comments: Vec<String>,
    pub line: usize,
}

#[derive(Clone, Debug)]
pub struct Treatment {
    pub phases: Vec<Phase>,
}

/// A graph of strings as written: paths without crossings, `-i- chain -j-`.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphText {
    pub paths: Vec<(u32, Vec<Tree>, u32)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxError {
    pub line: usize,
    pub col: usize,
    pub msg: String,
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}: {}", self.line, self.col, self.msg)
    }
}

impl std::error::Error for SyntaxError {}

type R<T> = Result<T, SyntaxError>;

// ---------------------------------------------------------------- printing

impl fmt::Display for PLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PLabel::Lit(s) => f.write_str(s),
            PLabel::Var(v) => write!(f, "{v}"),
        }
    }
}

impl fmt::Display for PTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PTree::Var(v) => write!(f, "{v}"),
            PTree::Node(l, items) => {
                write!(f, "{l}")?;
                if !items.is_empty() {
                    write!(f, "({})", items_string(items))?;
                }
                Ok(())
            }
        }
    }
}

impl fmt::Display for PItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PItem::Tree(t) => write!(f, "{t}"),
            PItem::List(v) => write!(f, "{v}"),
            PItem::Dis(l) => write!(f, "$${l}"),
        }
    }
}

pub fn items_string(items: &[PItem]) -> String {
    if items.is_empty() {
        return "-NUL-".into();
    }
    items.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(",")
}

pub fn pchain_string(c: &[PTree]) -> String {
    c.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(" + ")
}

impl fmt::Display for Rel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Rel::Eq => "=",
            Rel::Ne => "≠",
            Rel::Dans => "-DANS-",
            Rel::Hors => "-HORS-",
        })
    }
}

impl fmt::Display for Cond {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cond::Or(a, b) => write!(f, "{a} -OU- {b}"),
            Cond::And(a, b) => write!(f, "{a} -ET- {b}"),
            Cond::Not(a) => write!(f, "-NON- {a}"),
            Cond::Paren(a) => write!(f, "(. {a} .)"),
            Cond::Rel(l, r, m) => write!(f, "{} {r} {}", items_string(l), items_string(m)),
        }
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} == {}", pchain_string(&self.left), pchain_string(&self.right))?;
        if let Some(c) = &self.cond {
            write!(f, " / {c}")?;
        }
        f.write_str(".")
    }
}

impl Rule {
    /// The rule with its members exchanged (C70 section 9).
    pub fn inverted(&self) -> Rule {
        Rule {
            left: self.right.clone(),
            right: self.left.clone(),
            cond: self.cond.clone(),
            line: self.line,
            start: self.start,
            end: self.end,
        }
    }
}

// ---------------------------------------------------------------- reading

struct P<'a> {
    s: Vec<char>,
    i: usize,
    _src: &'a str,
}

impl<'a> P<'a> {
    fn new(src: &'a str) -> Self {
        P {
            s: src.chars().collect(),
            i: 0,
            _src: src,
        }
    }

    fn pos(&self, at: usize) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;
        for &c in &self.s[..at.min(self.s.len())] {
            if c == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }

    fn err<T>(&self, msg: impl Into<String>) -> R<T> {
        self.err_at(self.i, msg)
    }

    fn err_at<T>(&self, at: usize, msg: impl Into<String>) -> R<T> {
        let (line, col) = self.pos(at);
        Err(SyntaxError {
            line,
            col,
            msg: msg.into(),
        })
    }

    fn peek(&self) -> Option<char> {
        self.s.get(self.i).copied()
    }

    fn peek_at(&self, k: usize) -> Option<char> {
        self.s.get(self.i + k).copied()
    }

    /// Blanks and line ends separate, and are otherwise ignored (decision 3).
    fn ws(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.i += 1;
            } else {
                break;
            }
        }
    }

    fn at(&self, lit: &str) -> bool {
        lit.chars().enumerate().all(|(k, c)| self.peek_at(k) == Some(c))
    }

    fn eat(&mut self, lit: &str) -> bool {
        if self.at(lit) {
            self.i += lit.chars().count();
            true
        } else {
            false
        }
    }

    fn eof(&mut self) -> bool {
        self.ws();
        self.i >= self.s.len()
    }

    /// A parameter: one of A-F, I-N, U-Z, then `*`, then perhaps a digit.
    fn at_param(&self) -> bool {
        matches!(self.peek(), Some('A'..='F' | 'I'..='N' | 'U'..='Z')) && self.peek_at(1) == Some('*')
    }

    fn param(&mut self) -> Var {
        let letter = self.s[self.i];
        self.i += 2;
        let mut index = 0;
        if let Some(c) = self.peek() {
            if is_digit(c) {
                index = c as u8 - b'0';
                self.i += 1;
            }
        }
        Var { letter, index }
    }

    /// A label: any character, then letters and digits.
    fn label(&mut self) -> R<Rc<str>> {
        let start = self.i;
        match self.peek() {
            None => return self.err("a label was expected, but the text ends"),
            Some(c) if c.is_ascii_lowercase() => {
                return self.err(format!(
                    "'{c}': there are no small letters (C70 p. 6: the CDC 6400 has none)"
                ))
            }
            Some(c) if !is_character(c) => return self.err(format!("'{c}' is not a character of C70 p. 6")),
            Some(_) => self.i += 1,
        }
        while let Some(c) = self.peek() {
            if is_letter(c) || is_digit(c) {
                self.i += 1;
            } else if c.is_ascii_lowercase() {
                return self.err(format!("'{c}': there are no small letters (C70 p. 6)"));
            } else {
                break;
            }
        }
        let s: String = self.s[start..self.i].iter().collect();
        Ok(s.into())
    }

    /// A tree, parameterised if `params`.
    fn ptree(&mut self, params: bool) -> R<PTree> {
        self.ws();
        let start = self.i;
        if self.at("-NUL-") {
            return self.err("-NUL- is a list, not a tree");
        }
        if self.at("$$") {
            return self.err("$$ makes a list, not a tree");
        }
        let lab = if self.at_param() {
            if !params {
                return self.err("parameters are not allowed here");
            }
            let v = self.param();
            match v.kind() {
                Kind::Tree => return Ok(PTree::Var(v)),
                Kind::List => return self.err_at(start, format!("{v} is a list parameter; a tree is expected here")),
                Kind::Label => PLabel::Var(v),
            }
        } else {
            PLabel::Lit(self.label()?)
        };
        let save = self.i;
        self.ws();
        if self.peek() == Some('(') {
            self.i += 1;
            let items = self.plist(params)?;
            self.ws();
            if !self.eat(")") {
                return self.err("')' expected");
            }
            Ok(PTree::Node(lab, items))
        } else {
            self.i = save;
            Ok(PTree::Node(lab, Vec::new()))
        }
    }

    /// A list: simple lists separated by commas.
    fn plist(&mut self, params: bool) -> R<Vec<PItem>> {
        let mut out = Vec::new();
        loop {
            self.ws();
            if self.eat("-NUL-") {
            } else if self.eat("$$") {
                self.ws();
                if self.at_param() {
                    if !params {
                        return self.err("parameters are not allowed here");
                    }
                    let at = self.i;
                    let v = self.param();
                    if v.kind() != Kind::Label {
                        return self.err_at(at, format!("$$ takes a label; {v} is not a label parameter"));
                    }
                    out.push(PItem::Dis(PLabel::Var(v)));
                } else {
                    let l = self.label()?;
                    if params {
                        out.push(PItem::Dis(PLabel::Lit(l)));
                    } else {
                        for c in l.chars() {
                            out.push(PItem::Tree(PTree::Node(PLabel::Lit(c.to_string().into()), vec![])));
                        }
                    }
                }
            } else if self.at_param() {
                if !params {
                    return self.err("parameters are not allowed here");
                }
                let save = self.i;
                let v = self.param();
                if v.kind() == Kind::List {
                    out.push(PItem::List(v));
                } else {
                    self.i = save;
                    out.push(PItem::Tree(self.ptree(params)?));
                }
            } else {
                out.push(PItem::Tree(self.ptree(params)?));
            }
            self.ws();
            if self.peek() == Some(',') {
                self.i += 1;
            } else {
                return Ok(out);
            }
        }
    }

    /// A parameterised string: trees separated by `+`.
    fn pchain(&mut self, params: bool) -> R<Vec<PTree>> {
        let mut out = vec![self.ptree(params)?];
        loop {
            self.ws();
            if self.peek() == Some('+') {
                self.i += 1;
                out.push(self.ptree(params)?);
            } else {
                return Ok(out);
            }
        }
    }

    fn relation(&mut self) -> R<Rel> {
        self.ws();
        if self.eat("-DANS-") {
            Ok(Rel::Dans)
        } else if self.eat("-HORS-") {
            Ok(Rel::Hors)
        } else if self.eat("≠") || self.eat("#") {
            Ok(Rel::Ne)
        } else if self.peek() == Some('=') && self.peek_at(1) != Some('=') {
            self.i += 1;
            Ok(Rel::Eq)
        } else {
            self.err("a relation (=, ≠, -DANS- or -HORS-) was expected")
        }
    }

    fn expr(&mut self, prev: Option<&Cond>, depth: usize) -> R<Cond> {
        let a = self.secondary(prev, depth)?;
        self.ws();
        if self.eat("-OU-") {
            let b = self.expr(prev, depth)?;
            Ok(Cond::Or(Box::new(a), Box::new(b)))
        } else {
            Ok(a)
        }
    }

    fn secondary(&mut self, prev: Option<&Cond>, depth: usize) -> R<Cond> {
        let a = self.primary(prev, depth)?;
        self.ws();
        if self.eat("-ET-") {
            let b = self.secondary(prev, depth)?;
            Ok(Cond::And(Box::new(a), Box::new(b)))
        } else {
            Ok(a)
        }
    }

    fn primary(&mut self, prev: Option<&Cond>, depth: usize) -> R<Cond> {
        self.ws();
        if self.at("--") {
            let at = self.i;
            self.i += 2;
            return match prev {
                Some(c) => Ok(Cond::Paren(Box::new(c.clone()))),
                None => self.err_at(
                    at,
                    "-- in a condition stands for the condition of the rule before, and that rule has none (C70 p. 26)",
                ),
            };
        }
        if self.eat("-NON-") {
            return Ok(Cond::Not(Box::new(self.primary(prev, depth)?)));
        }
        if self.at("(.") {
            self.i += 2;
            let e = self.expr(prev, depth + 1)?;
            self.ws();
            if !self.eat(".)") {
                return self.err("'.)' expected");
            }
            return Ok(Cond::Paren(Box::new(e)));
        }
        let l = self.plist(true)?;
        let r = self.relation()?;
        let m = self.plist(true)?;
        Ok(Cond::Rel(l, r, m))
    }

    /// One rule, with `--` replaced by the parts of the rule before.
    fn rule(&mut self, prev: Option<&Rule>) -> R<Rule> {
        self.ws();
        let start = self.i;
        let (line, _) = self.pos(self.i);
        let left = if self.at("--") && self.peek_at(2) != Some('-') {
            let at = self.i;
            self.i += 2;
            match prev {
                Some(p) => p.left.clone(),
                None => return self.err_at(at, "the first rule of a system cannot contain -- (C70 p. 26)"),
            }
        } else {
            self.pchain(true)?
        };
        self.ws();
        if !self.eat("==") {
            return self.err("'==' expected");
        }
        self.ws();
        let right = if self.at("--") {
            let at = self.i;
            self.i += 2;
            match prev {
                Some(p) => p.right.clone(),
                None => return self.err_at(at, "the first rule of a system cannot contain -- (C70 p. 26)"),
            }
        } else {
            self.pchain(true)?
        };
        self.ws();
        let cond = if self.peek() == Some('/') {
            self.i += 1;
            Some(self.expr(prev.and_then(|p| p.cond.as_ref()), 0)?)
        } else {
            None
        };
        self.ws();
        if !self.eat(".") {
            return self.err("'.' expected at the end of the rule");
        }
        Ok(Rule {
            left,
            right,
            cond,
            line,
            start,
            end: self.i,
        })
    }

    /// A comment: `**`, then any characters but `.` and `=`, then `.`.
    fn comment(&mut self) -> R<String> {
        let start = self.i;
        self.i += 2;
        while let Some(c) = self.peek() {
            match c {
                '.' => {
                    let s: String = self.s[start..self.i].iter().collect();
                    self.i += 1;
                    return Ok(s.split_whitespace().collect::<Vec<_>>().join(" "));
                }
                '=' => return self.err("a comment cannot contain '=' (C70 p. 25)"),
                _ => self.i += 1,
            }
        }
        self.err_at(start, "this comment has no '.' to end it")
    }

    fn phase(&mut self) -> R<Phase> {
        self.ws();
        let (line, _) = self.pos(self.i);
        let det = self.eat("-DET-");
        self.ws();
        let inv = self.eat("-INV-");
        let mut rules: Vec<Rule> = Vec::new();
        let mut comments = Vec::new();
        loop {
            if self.eof() || self.at("-REQ-") {
                break;
            }
            if self.at("**") {
                comments.push(self.comment()?);
                continue;
            }
            if self.at("-DET-") || self.at("-INV-") {
                return self.err("-DET- and -INV- come at the start of a phase, -DET- first (C70 p. 34)");
            }
            let r = self.rule(rules.last())?;
            rules.push(r);
        }
        Ok(Phase {
            det,
            inv,
            rules,
            comments,
            line,
        })
    }

    fn vertex(&mut self) -> R<u32> {
        self.ws();
        let at = self.i;
        if self.peek() != Some('-') {
            return self.err("a vertex -n- was expected");
        }
        self.i += 1;
        let st = self.i;
        while matches!(self.peek(), Some(c) if is_digit(c)) {
            self.i += 1;
        }
        if st == self.i || self.peek() != Some('-') {
            return self.err_at(at, "a vertex is a whole number between hyphens, as -0- (C70 p. 20)");
        }
        let n: String = self.s[st..self.i].iter().collect();
        self.i += 1;
        n.parse()
            .or_else(|_| self.err_at(at, "this vertex number is too large"))
    }
}

/// Read a treatment: phases separated by `-REQ-`.
pub fn parse_treatment(src: &str) -> Result<Treatment, SyntaxError> {
    let mut p = P::new(src);
    let mut phases = vec![p.phase()?];
    while !p.eof() {
        if !p.eat("-REQ-") {
            return p.err("-REQ- expected");
        }
        phases.push(p.phase()?);
    }
    Ok(Treatment { phases })
}

/// Read a single system, for tests and the REPL.
pub fn parse_system(src: &str) -> Result<Phase, SyntaxError> {
    let t = parse_treatment(src)?;
    if t.phases.len() != 1 {
        return Err(SyntaxError {
            line: 1,
            col: 1,
            msg: "one system was expected, not a treatment".into(),
        });
    }
    Ok(t.phases.into_iter().next().unwrap())
}

/// Read the data of a treatment: graphs of strings separated by `/`.
pub fn parse_data(src: &str) -> Result<Vec<GraphText>, SyntaxError> {
    let mut p = P::new(src);
    let mut graphs = Vec::new();
    let mut cur = GraphText { paths: Vec::new() };
    loop {
        if p.eof() {
            break;
        }
        if p.peek() == Some('/') {
            p.i += 1;
            graphs.push(std::mem::replace(&mut cur, GraphText { paths: Vec::new() }));
            continue;
        }
        let a = p.vertex()?;
        p.ws();
        let at = p.i;
        let chain = p.pchain(false)?;
        let b = p.vertex()?;
        let trees = chain.iter().map(ground).collect::<Option<Vec<_>>>();
        match trees {
            Some(t) => cur.paths.push((a, t, b)),
            None => return p.err_at(at, "a graph cannot contain parameters"),
        }
    }
    if !cur.paths.is_empty() || graphs.is_empty() {
        graphs.push(cur);
    }
    Ok(graphs)
}

/// Read one tree, for tests.
pub fn parse_tree(src: &str) -> Result<Tree, SyntaxError> {
    let mut p = P::new(src);
    let t = p.ptree(false)?;
    if !p.eof() {
        return p.err("text after the tree");
    }
    Ok(ground(&t).expect("no parameters"))
}

/// Read one list, for tests: `-NUL-` and `$$` allowed.
pub fn parse_list(src: &str) -> Result<Vec<Tree>, SyntaxError> {
    let mut p = P::new(src);
    let items = p.plist(false)?;
    if !p.eof() {
        return p.err("text after the list");
    }
    Ok(items
        .iter()
        .map(|i| match i {
            PItem::Tree(t) => ground(t).unwrap(),
            _ => unreachable!(),
        })
        .collect())
}

/// A parameterised tree with no parameters, as a tree.
pub fn ground(t: &PTree) -> Option<Tree> {
    match t {
        PTree::Var(_) => None,
        PTree::Node(PLabel::Var(_), _) => None,
        PTree::Node(PLabel::Lit(l), items) => {
            let mut kids = Vec::new();
            for i in items {
                match i {
                    PItem::Tree(t) => kids.push(ground(t)?),
                    PItem::Dis(PLabel::Lit(l)) => kids.extend(l.chars().map(|c| Tree::leaf(&c.to_string()))),
                    _ => return None,
                }
            }
            debug_assert!(is_label(l));
            Some(Tree { label: l.clone(), kids })
        }
    }
}
