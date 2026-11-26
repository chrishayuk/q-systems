//! Every example in the tutorial is run, and what it prints is compared with
//! the `output` block under it.
//!
//! An example is a ```` ```q ```` block (the treatment), then a
//! ```` ```data ```` block (the graphs), then an ```` ```output ```` block.
//! Flags on the info line of the `q` block are passed to `qsys`: `--det`,
//! `--max-arrows N`. `QSYS_BLESS=1 cargo test --test tutorial` rewrites the
//! output blocks from what the examples print.

use qsystems::machine::Limits;
use qsystems::report::{printout, run_all, Opts};
use std::fs;

fn output(info: &str, program: &str, data: &str) -> String {
    let max = info
        .split("--max-arrows ")
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
        .and_then(|n| n.parse().ok())
        .unwrap_or(Limits::default().max_arrows);
    let opts = Opts {
        det_all: info.contains("--det"),
        limits: Limits { max_arrows: max },
        ..Opts::default()
    };
    match run_all(program, data, opts.limits) {
        Ok((sys, runs)) => printout(&sys, &runs, opts),
        Err(e) => format!("qsys: {e}\n"),
    }
}

/// The text of the fenced block starting at line `i`, and the line after it.
fn block(lines: &[&str], i: usize) -> (String, usize) {
    let mut j = i + 1;
    while !lines[j].starts_with("```") {
        j += 1;
    }
    (lines[i + 1..j].join("\n") + "\n", j + 1)
}

fn skip_blank(lines: &[&str], mut k: usize) -> usize {
    while k < lines.len() && lines[k].trim().is_empty() {
        k += 1;
    }
    k
}

#[test]
fn tutorial() {
    let bless = std::env::var("QSYS_BLESS").is_ok();
    let mut checked = 0;
    let mut files: Vec<_> = fs::read_dir("tutorial").unwrap().map(|e| e.unwrap().path()).collect();
    files.sort();
    for path in files.iter().filter(|p| p.extension().is_some_and(|x| x == "md")) {
        let text = fs::read_to_string(path).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        let mut out: Vec<String> = Vec::new();
        let mut i = 0;
        while i < lines.len() {
            let Some(info) = lines[i].strip_prefix("```q") else {
                out.push(lines[i].to_string());
                i += 1;
                continue;
            };
            let (program, after) = block(&lines, i);
            out.extend(lines[i..after].iter().map(|s| s.to_string()));
            let k = skip_blank(&lines, after);
            assert!(
                k < lines.len() && lines[k].starts_with("```data"),
                "{}: a q block at line {} has no data block after it",
                path.display(),
                i + 1
            );
            let (data, after_data) = block(&lines, k);
            out.extend(lines[after..after_data].iter().map(|s| s.to_string()));
            let got = output(info, &program, &data);
            let m = skip_blank(&lines, after_data);
            let (want, next) = if m < lines.len() && lines[m].starts_with("```output") {
                let (w, n) = block(&lines, m);
                (Some(w), n)
            } else {
                (None, after_data)
            };
            out.extend(lines[after_data..m.min(lines.len())].iter().map(|s| s.to_string()));
            if bless {
                out.push("```output".into());
                out.extend(got.trim_end().lines().map(|s| s.to_string()));
                out.push("```".into());
            } else {
                let want = want.unwrap_or_else(|| panic!("{}: no output block after line {}", path.display(), k + 1));
                assert_eq!(got.trim_end(), want.trim_end(), "{} line {}", path.display(), i + 1);
                out.extend(lines[m..next].iter().map(|s| s.to_string()));
            }
            checked += 1;
            i = next;
        }
        if bless {
            fs::write(path, out.join("\n") + "\n").unwrap();
        }
    }
    assert!(checked > 0 || bless, "no examples found");
}
