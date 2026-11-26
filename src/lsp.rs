//! A language server for Q-system texts, over standard input and output
//! (`qsys lsp`). A `.q` file is a treatment; a `.dat` file is its data, the
//! graphs of strings it is applied to.
//!
//! - **Problems**, as the text is typed: anything not in the grammar of C70's
//!   appendix, and C70's restriction on parameters (p. 27, or p. 33 under
//!   `-INV-`).
//! - **Hover.** On a rule, the rule as the machine applies it (`--` filled
//!   in, the members exchanged under `-INV-`), with its phase and its
//!   parameters. On a parameter, what it stands for. On a word of the
//!   language, what it does, with its page of C70.
//! - **Colours**, as semantic tokens, so the same server colours the editor
//!   and the playground.

use crate::json::{self, obj, Json};
use crate::machine::{Graph, System};
use crate::matching::{vars_chain, vars_cond};
use crate::syntax::{parse_data, parse_treatment, Kind, Rule, Var};
use std::collections::HashMap;
use std::io::{BufRead, Write};

pub const TOKEN_TYPES: &[&str] = &[
    "comment",   // 0 ** … .
    "keyword",   // 1 -INV- -REQ- -DET-
    "operator",  // 2 == + / -- $$ (. .)
    "parameter", // 3 A* I*1 U*
    "number",    // 4 the vertices of a graph, -0-
    "macro",     // 5 -DANS- -HORS- -ET- -OU- -NON- -NUL- = ≠
];

fn is_data(uri: &str) -> bool {
    uri.ends_with(".dat") || uri.ends_with(".qdat")
}

/// (start, length, type) in characters, over the whole text.
pub fn tokens(text: &str) -> Vec<(usize, usize, usize)> {
    let s: Vec<char> = text.chars().collect();
    let at = |i: usize, lit: &str| lit.chars().enumerate().all(|(k, c)| s.get(i + k) == Some(&c));
    let mut out = Vec::new();
    let mut i = 0;
    while i < s.len() {
        let c = s[i];
        if at(i, "**") {
            let mut j = i + 2;
            while j < s.len() && s[j] != '.' {
                j += 1;
            }
            let end = (j + 1).min(s.len());
            out.push((i, end - i, 0));
            i = end;
            continue;
        }
        let word = ["-INV-", "-REQ-", "-DET-"].iter().find(|w| at(i, w));
        if let Some(w) = word {
            out.push((i, w.len(), 1));
            i += w.len();
            continue;
        }
        let word = ["-DANS-", "-HORS-", "-ET-", "-OU-", "-NON-", "-NUL-"]
            .iter()
            .find(|w| at(i, w));
        if let Some(w) = word {
            out.push((i, w.len(), 5));
            i += w.len();
            continue;
        }
        if matches!(c, 'A'..='F' | 'I'..='N' | 'U'..='Z')
            && s.get(i + 1) == Some(&'*')
            && (i == 0 || !(s[i - 1].is_ascii_uppercase() || s[i - 1].is_ascii_digit()))
        {
            let n = if s.get(i + 2).is_some_and(|d| d.is_ascii_digit()) {
                3
            } else {
                2
            };
            out.push((i, n, 3));
            i += n;
            continue;
        }
        if c == '-' && s.get(i + 1).is_some_and(|d| d.is_ascii_digit()) {
            let mut j = i + 1;
            while j < s.len() && s[j].is_ascii_digit() {
                j += 1;
            }
            if s.get(j) == Some(&'-') {
                out.push((i, j + 1 - i, 4));
                i = j + 1;
                continue;
            }
        }
        if let Some(op) = ["==", "--", "$$", "(.", ".)"].iter().find(|op| at(i, op)) {
            out.push((i, op.len(), 2));
            i += op.len();
            continue;
        }
        if matches!(c, '≠' | '#') || (c == '=' && s.get(i + 1) != Some(&'=')) {
            out.push((i, 1, 5));
        } else if c == '/' && s.get(i + 1).is_some_and(|d| d.is_whitespace()) {
            out.push((i, 1, 2));
        }
        i += 1;
    }
    out
}

fn line_starts(text: &str) -> Vec<usize> {
    let mut v = vec![0];
    for (i, c) in text.chars().enumerate() {
        if c == '\n' {
            v.push(i + 1);
        }
    }
    v
}

/// (line, column) of a character offset. Every character of the language
/// is in the Basic Multilingual Plane, so a character is one UTF-16 unit.
fn line_col(starts: &[usize], at: usize) -> (usize, usize) {
    let line = starts.partition_point(|&s| s <= at) - 1;
    (line, at - starts[line])
}

fn position(line: usize, character: usize) -> Json {
    obj([("line", line.into()), ("character", character.into())])
}

fn range(a: (usize, usize), b: (usize, usize)) -> Json {
    obj([("start", position(a.0, a.1)), ("end", position(b.0, b.1))])
}

/// (line, start, length, type), one line at a time, in order.
pub fn semantic(text: &str) -> Vec<(usize, usize, usize, usize)> {
    let starts = line_starts(text);
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    for (from, len, kind) in tokens(text) {
        // a comment may run over several lines: one token a line
        let mut a = from;
        let end = from + len;
        while a < end {
            let (line, col) = line_col(&starts, a);
            let mut b = a;
            while b < end && chars[b] != '\n' {
                b += 1;
            }
            if b > a {
                out.push((line, col, b - a, kind));
            }
            a = b + 1;
        }
    }
    out
}

fn semantic_tokens(text: &str) -> Json {
    let mut data = Vec::new();
    let (mut last_line, mut last_start) = (0, 0);
    for (line, start, length, kind) in semantic(text) {
        let delta = if line == last_line { start - last_start } else { start };
        data.extend([line - last_line, delta, length, kind, 0].map(Json::from));
        (last_line, last_start) = (line, start);
    }
    obj([("data", data.into())])
}

/// Every problem in a text: (line, column, message), 0-based.
pub fn problems(uri: &str, text: &str) -> Vec<(usize, usize, String)> {
    if is_data(uri) {
        return match parse_data(text) {
            Err(e) => vec![(e.line - 1, e.col - 1, e.msg)],
            Ok(gs) => gs
                .iter()
                .enumerate()
                .filter_map(|(i, g)| {
                    Graph::from_text(g).err().map(|e| {
                        let line = text
                            .lines()
                            .position(|l| l.contains(&format!("-{}-", g.paths.first().map_or(0, |p| p.0))))
                            .unwrap_or(0);
                        let msg = if gs.len() > 1 {
                            format!("graph {}: {}", i + 1, e.0)
                        } else {
                            e.0
                        };
                        (line, 0, msg)
                    })
                })
                .collect(),
        };
    }
    let t = match parse_treatment(text) {
        Ok(t) => t,
        Err(e) => return vec![(e.line - 1, e.col - 1, e.msg)],
    };
    let mut out = Vec::new();
    for p in &t.phases {
        if let Err(e) = System::new(p) {
            let line =
                e.0.strip_prefix("line ")
                    .and_then(|r| r.split(':').next())
                    .and_then(|n| n.parse::<usize>().ok())
                    .unwrap_or(p.line);
            let msg = e.0.split_once(": ").map_or(e.0.clone(), |(_, m)| m.to_string());
            out.push((line - 1, 0, msg));
        }
    }
    out
}

fn diagnostics(uri: &str, text: &str) -> Json {
    let list: Vec<Json> = problems(uri, text)
        .into_iter()
        .map(|(l, c, msg)| {
            let width = text
                .lines()
                .nth(l)
                .map_or(1, |s| s.chars().count().saturating_sub(c).max(1));
            obj([
                ("range", range((l, c), (l, c + width))),
                ("severity", 1.into()),
                ("source", "qsys".into()),
                ("message", msg.into()),
            ])
        })
        .collect();
    obj([
        ("jsonrpc", "2.0".into()),
        ("method", "textDocument/publishDiagnostics".into()),
        ("params", obj([("uri", uri.into()), ("diagnostics", list.into())])),
    ])
}

fn capabilities() -> Json {
    obj([
        (
            "capabilities",
            obj([
                ("textDocumentSync", 1.into()),
                ("hoverProvider", true.into()),
                ("documentSymbolProvider", true.into()),
                (
                    "semanticTokensProvider",
                    obj([
                        (
                            "legend",
                            obj([
                                (
                                    "tokenTypes",
                                    TOKEN_TYPES.iter().map(|t| Json::from(*t)).collect::<Vec<_>>().into(),
                                ),
                                ("tokenModifiers", Vec::new().into()),
                            ]),
                        ),
                        ("full", true.into()),
                    ]),
                ),
                ("completionProvider", obj([("resolveProvider", false.into())])),
            ]),
        ),
        (
            "serverInfo",
            obj([("name", "qsys".into()), ("version", env!("CARGO_PKG_VERSION").into())]),
        ),
    ])
}

/// What a word of the language does, and where C70 says so.
pub fn word_help(w: &str) -> Option<&'static str> {
    Some(match w {
        "-INV-" => "**-INV-** inverts the phase: every rule is applied with its members exchanged, after `--` is filled in. The same system then does the inverse job, analysis for synthesis (C70 section 9, pp. 32-33).",
        "-REQ-" => "**-REQ-** separates the phases of a treatment. Each phase is applied to the graph the one before left (C70 section 10, p. 34).",
        "-DET-" => "**-DET-** asks for the details of the phase: every application of every rule (C70 p. 34).",
        "-DANS-" => "**l1 -DANS- l2**: every tree of l1 is found in l2, each label equal and its list -DANS- the other's. `PH -DANS- PH(SN(BORIS),SV(DORT))` (C70 pp. 14-15).",
        "-HORS-" => "**l1 -HORS- l2**: every tree of l1 differs from every tree of l2, by its label or, recursively, its list. False if either is `-NUL-`. Not the negation of -DANS-: `A -HORS- A(B)` and `A(C) -DANS- A(B)` are both false (C70 pp. 14-15).",
        "-ET-" => "**-ET-**, and. Binds tighter than -OU-, looser than -NON- and the relations (C70 p. 27).",
        "-OU-" => "**-OU-**, or. Binds loosest (C70 p. 27).",
        "-NON-" => "**-NON-**, not (C70 p. 27).",
        "-NUL-" => "**-NUL-**, the empty list. `e(-NUL-)` is `e`, and `l,-NUL-` is `l` (C70 p. 9). A graph given as data may not use it (p. 20).",
        "==" => "**==** reads \"se réécrit en\": for every path whose trees match the left member, a path carrying the right member is *added* beside it. Nothing is replaced until the second phase, when every arrow of a path some rule was applied to is removed (C70 pp. 21-22).",
        "--" => "**--** stands for the same part of the rule before: its left member, its right member, or `(.` its condition `.)` (C70 p. 26).",
        "$$" => "**$$**, the disintegrator: `$$ABC` is the list `A,B,C` of the label's characters. `A* == MOT($$A*).` takes a label apart; `MOT($$A*) == A*.` puts it back (C70 pp. 9, 29).",
        "(." | ".)" => "**(.** and **.)** group a condition (C70 p. 25).",
        "/" => "**/** begins the condition of a rule. A rule applies only under the substitutions that make its condition true (C70 pp. 25-27).",
        "=" => "**=**, two lists are equal, after `-NUL-` and `$$` are written out (C70 p. 14).",
        "≠" | "#" => "**≠** (`#` may be typed), two lists differ (C70 p. 14).",
        _ => return None,
    })
}

fn var_help(v: Var) -> String {
    let what = match v.kind() {
        Kind::Label => {
            "a **label parameter** (A to F): any label. Standing as a tree it is `A*(-NUL-)`, so it matches a leaf only"
        }
        Kind::Tree => "a **tree parameter** (I to N): any tree",
        Kind::List => "a **list parameter** (U to Z): any list of trees, `-NUL-` included",
    };
    format!("`{v}` is {what} (C70 pp. 11-13). Every occurrence in a rule stands for the same thing; every substitution that matches is a rule of its own (p. 26).")
}

/// The rule under a place, with its phase.
fn rule_at(text: &str, at: usize) -> Option<(usize, usize, Rule, bool)> {
    let t = parse_treatment(text).ok()?;
    for (pi, p) in t.phases.iter().enumerate() {
        for (ri, r) in p.rules.iter().enumerate() {
            if r.start <= at && at < r.end {
                return Some((pi, ri, r.clone(), p.inv));
            }
        }
    }
    None
}

fn rule_help(pi: usize, ri: usize, r: &Rule, inv: bool, written: &str) -> String {
    let applied = if inv { r.inverted() } else { r.clone() };
    let mut s = format!("**Phase {}, rule {}**", pi + 1, ri + 1);
    if inv {
        s.push_str(" — the phase is `-INV-`, so the rule is applied with its members exchanged:");
    }
    s.push_str(&format!("\n\n`{applied}`"));
    if written.contains("--") {
        s.push_str("\n\n`--` filled in from the rule before.");
    }
    let mut vs = Vec::new();
    vars_chain(&applied.left, &mut vs);
    vars_chain(&applied.right, &mut vs);
    if let Some(c) = &applied.cond {
        vars_cond(c, &mut vs);
    }
    let mut seen = Vec::new();
    for v in vs {
        if !seen.contains(&v) {
            seen.push(v);
        }
    }
    if !seen.is_empty() {
        let kinds: Vec<String> = seen
            .iter()
            .map(|v| {
                format!(
                    "`{v}` {}",
                    match v.kind() {
                        Kind::Label => "a label",
                        Kind::Tree => "a tree",
                        Kind::List => "a list",
                    }
                )
            })
            .collect();
        s.push_str(&format!("\n\nParameters: {}.", kinds.join(", ")));
    }
    s.push_str(&format!(
        "\n\nFor every path of {} arrow{} whose trees match the left member{}, a path of {} arrow{} carrying the right member is added beside it.",
        applied.left.len(),
        if applied.left.len() == 1 { "" } else { "s" },
        if applied.cond.is_some() { " and make the condition true" } else { "" },
        applied.right.len(),
        if applied.right.len() == 1 { "" } else { "s" },
    ));
    s
}

/// The hover text at a place (0-based line, UTF-16 column).
pub fn hover(uri: &str, text: &str, line: usize, col: usize) -> Option<String> {
    let starts = line_starts(text);
    let at = starts.get(line)? + col;
    let chars: Vec<char> = text.chars().collect();
    // a token under the place
    let tok = tokens(text).into_iter().find(|&(s, l, _)| s <= at && at < s + l.max(1));
    if let Some((s, l, kind)) = tok {
        let word: String = chars[s..s + l].iter().collect();
        match kind {
            0 => return None,
            3 => {
                let letter = word.chars().next()?;
                let index = word.chars().nth(2).and_then(|d| d.to_digit(10)).unwrap_or(0) as u8;
                return Some(var_help(Var { letter, index }));
            }
            4 => return Some(format!("`{word}`, a vertex. A graph is written as paths without crossings, `-i- A + B -j-`; every path into a vertex must be written before any path from it (C70 p. 20).")),
            _ => {
                if let Some(h) = word_help(&word) {
                    let more = if word == "--" && !is_data(uri) {
                        rule_at(text, at).map(|(pi, ri, r, inv)| format!("\n\n---\n\n{}", rule_help(pi, ri, &r, inv, "--"))).unwrap_or_default()
                    } else {
                        String::new()
                    };
                    return Some(format!("{h}{more}"));
                }
            }
        }
    }
    if is_data(uri) {
        return None;
    }
    let (pi, ri, r, inv) = rule_at(text, at)?;
    let written: String = chars[r.start..r.end].iter().collect();
    Some(rule_help(pi, ri, &r, inv, &written))
}

fn completion() -> Json {
    let words = [
        "-INV-", "-REQ-", "-DET-", "-DANS-", "-HORS-", "-ET-", "-OU-", "-NON-", "-NUL-", "$$", "--",
    ];
    words
        .iter()
        .map(|w| {
            obj([
                ("label", (*w).into()),
                ("kind", 14.into()),
                ("detail", word_help(w).unwrap_or("").replace("**", "").into()),
            ])
        })
        .collect::<Vec<_>>()
        .into()
}

fn symbols(uri: &str, text: &str) -> Json {
    if is_data(uri) {
        return Vec::new().into();
    }
    let Ok(t) = parse_treatment(text) else {
        return Vec::new().into();
    };
    let starts = line_starts(text);
    t.phases
        .iter()
        .enumerate()
        .map(|(pi, p)| {
            let kids: Vec<Json> = p
                .rules
                .iter()
                .enumerate()
                .map(|(ri, r)| {
                    let a = line_col(&starts, r.start);
                    let b = line_col(&starts, r.end);
                    let applied = if p.inv { r.inverted() } else { r.clone() };
                    obj([
                        ("name", format!("rule {}", ri + 1).into()),
                        ("detail", applied.to_string().into()),
                        ("kind", 12.into()),
                        ("range", range(a, b)),
                        ("selectionRange", range(a, a)),
                    ])
                })
                .collect();
            let a = p.rules.first().map_or((p.line - 1, 0), |r| line_col(&starts, r.start));
            let b = p.rules.last().map_or(a, |r| line_col(&starts, r.end));
            let a = a.min((p.line - 1, 0));
            obj([
                ("name", format!("phase {}", pi + 1).into()),
                (
                    "detail",
                    format!("{}{} rules", if p.inv { "-INV-, " } else { "" }, p.rules.len()).into(),
                ),
                ("kind", 2.into()),
                ("range", range(a, b)),
                ("selectionRange", range(a, a)),
                ("children", kids.into()),
            ])
        })
        .collect::<Vec<_>>()
        .into()
}

fn answer(method: &str, params: &Json, docs: &HashMap<String, String>) -> Option<Json> {
    let uri = params.get("textDocument").get("uri").str().unwrap_or("");
    let text = docs.get(uri).map(String::as_str).unwrap_or("");
    let line = params.get("position").get("line").num().unwrap_or(0);
    let col = params.get("position").get("character").num().unwrap_or(0);
    Some(match method {
        "initialize" => capabilities(),
        "shutdown" => Json::Null,
        "textDocument/hover" => match hover(uri, text, line, col) {
            Some(h) => obj([("contents", obj([("kind", "markdown".into()), ("value", h.into())]))]),
            None => Json::Null,
        },
        "textDocument/semanticTokens/full" => semantic_tokens(text),
        "textDocument/documentSymbol" => symbols(uri, text),
        "textDocument/completion" => completion(),
        _ => return None,
    })
}

fn read_message(input: &mut impl BufRead) -> Option<Json> {
    let mut length = None;
    loop {
        let mut line = String::new();
        if input.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                length = value.trim().parse::<usize>().ok();
            }
        }
    }
    let mut body = vec![0; length?];
    input.read_exact(&mut body).ok()?;
    json::parse(&String::from_utf8_lossy(&body)).ok()
}

fn send(output: &mut impl Write, message: Json) {
    let mut body = String::new();
    message.write(&mut body);
    let _ = write!(output, "Content-Length: {}\r\n\r\n{}", body.len(), body);
    let _ = output.flush();
}

pub fn serve(input: &mut impl BufRead, output: &mut impl Write) -> Result<(), String> {
    let mut docs: HashMap<String, String> = HashMap::new();
    let mut shutting_down = false;
    while let Some(message) = read_message(input) {
        let method = message.get("method").str().unwrap_or("").to_string();
        let params = message.get("params");
        let id = message.get("id").clone();
        let uri = params.get("textDocument").get("uri").str().unwrap_or("").to_string();
        match method.as_str() {
            "exit" => {
                return if shutting_down {
                    Ok(())
                } else {
                    Err("exit without shutdown".into())
                }
            }
            "textDocument/didOpen" => {
                let text = params.get("textDocument").get("text").str().unwrap_or("").to_string();
                send(output, diagnostics(&uri, &text));
                docs.insert(uri, text);
            }
            "textDocument/didChange" => {
                if let Some(text) = params
                    .get("contentChanges")
                    .items()
                    .last()
                    .and_then(|c| c.get("text").str())
                {
                    send(output, diagnostics(&uri, text));
                    docs.insert(uri, text.to_string());
                }
            }
            "textDocument/didClose" => {
                docs.remove(&uri);
                send(output, diagnostics(&uri, ""));
            }
            _ if id == Json::Null => {}
            _ => {
                shutting_down |= method == "shutdown";
                let reply = match answer(&method, params, &docs) {
                    Some(result) => obj([("jsonrpc", "2.0".into()), ("id", id), ("result", result)]),
                    None => obj([
                        ("jsonrpc", "2.0".into()),
                        ("id", id),
                        (
                            "error",
                            obj([
                                ("code", Json::Num(-32601.0)),
                                ("message", format!("method not found: {method}").into()),
                            ]),
                        ),
                    ]),
                };
                send(output, reply);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const PARA: &str = include_str!("../examples/paraphrase.q");

    fn place(text: &str, needle: &str, nth: usize) -> (usize, usize) {
        let at = text.match_indices(needle).nth(nth).unwrap().0;
        let before = &text[..at];
        let line = before.matches('\n').count();
        let col = before[before.rfind('\n').map_or(0, |i| i + 1)..].chars().count();
        (line, col)
    }

    #[test]
    fn hover_on_a_rule_of_an_inverted_phase_shows_it_inverted() {
        let (l, c) = place(PARA, "SUJ + PH(U*)", 0);
        let h = hover("file:///p.q", PARA, l, c + 2).unwrap();
        assert!(h.contains("Phase 1, rule 4"), "{h}");
        assert!(h.contains("`LE + FAIT + QUE + PI(U*) == SUJ + PH(U*).`"), "{h}");
        let (l, c) = place(PARA, "SUJ + PH(U*)", 1);
        let h = hover("file:///p.q", PARA, l, c + 2).unwrap();
        assert!(
            h.contains("Phase 2, rule 4") && h.contains("`SUJ + PH(U*) == LE + FAIT + QUE + PI(U*).`"),
            "{h}"
        );
    }

    #[test]
    fn hover_on_ditto_shows_what_it_stands_for() {
        let (l, c) = place(PARA, "-- ==", 1);
        let h = hover("file:///p.q", PARA, l, c).unwrap();
        assert!(
            h.contains("stands for") && h.contains("`PI(V(A*),I*,J*) == SUJ + J* + EST + V(A*) + PPA + I*.`"),
            "{h}"
        );
    }

    #[test]
    fn hover_on_parameters_and_words() {
        let (l, c) = place(PARA, "U*", 0);
        assert!(hover("file:///p.q", PARA, l, c).unwrap().contains("list parameter"));
        let (l, c) = place(PARA, "-DANS-", 0);
        assert!(hover("file:///p.q", PARA, l, c + 1).unwrap().contains("pp. 14-15"));
    }

    #[test]
    fn problems_in_programs_and_data() {
        assert!(problems("file:///p.q", PARA).is_empty());
        let p = problems("file:///p.q", "A* == B(I*).\n");
        assert_eq!(p.len(), 1);
        assert!(p[0].2.contains("I*"), "{:?}", p);
        let p = problems("file:///p.q", "A == B\nC == D.\n");
        assert_eq!(p[0].0, 1, "{:?}", p);
        assert!(problems("file:///d.dat", "-0- A + B -1-\n").is_empty());
        assert!(!problems("file:///d.dat", "-1- B -2- -0- A -1-\n").is_empty());
    }

    #[test]
    fn tokens_of_a_rule() {
        let kinds: Vec<usize> = tokens("** NOTE. A* + A*(U*) == A*(1,U*) / A* -DANS- A,B,C.")
            .into_iter()
            .map(|t| t.2)
            .collect();
        assert_eq!(kinds, vec![0, 3, 3, 3, 2, 3, 3, 2, 3, 5]);
        let kinds: Vec<usize> = tokens("-0- A + B -12-").into_iter().map(|t| t.2).collect();
        assert_eq!(kinds, vec![4, 4]);
    }

    #[test]
    fn symbols_list_phases_and_rules() {
        let s = symbols("file:///p.q", PARA);
        assert_eq!(s.items().len(), 4);
        assert_eq!(s.items()[0].get("children").items().len(), 16);
    }
}
