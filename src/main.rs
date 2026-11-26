use qsystems::machine::Limits;
use qsystems::play::play;
use qsystems::report::{printout, run_all, systems, Opts};
use std::io::Read;
use std::process::exit;

const USAGE: &str = "\
qsys — Colmerauer's Q-systems (Montréal, 1969-70)

usage:
  qsys PROGRAM DATA        run the treatment on every graph of DATA, printing
                           the data and the graph after each phase
  qsys check PROGRAM       read a treatment and list its phases and rules,
                           as they are applied (-- filled in, -INV- done)
  qsys json PROGRAM DATA   the run as JSON, for the playground
  qsys lsp                 a language server on standard input and output

  PROGRAM may be - for the standard input; --data TEXT gives the data
  in place of a DATA file.

options:
  --det                    print every rule application of every phase, as
                           if each phase were marked -DET-
  --width N                fold lines at N characters, as the printer did
  --max-arrows N           the first phase of a system is taken not to
                           converge after adding N arrows (default 200000)
";

fn read(path: &str) -> String {
    if path == "-" {
        let mut s = String::new();
        if let Err(e) = std::io::stdin().read_to_string(&mut s) {
            eprintln!("qsys: standard input: {e}");
            exit(2)
        }
        return s;
    }
    std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("qsys: {path}: {e}");
        exit(2)
    })
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut opts = Opts::default();
    let mut pos = Vec::new();
    let mut data: Option<String> = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--det" => opts.det_all = true,
            "--width" => opts.width = it.next().and_then(|s| s.parse().ok()).unwrap_or(0),
            "--max-arrows" => {
                opts.limits = Limits {
                    max_arrows: it.next().and_then(|s| s.parse().ok()).unwrap_or(200_000),
                }
            }
            "--data" => data = it.next().cloned(),
            "-h" | "--help" => {
                print!("{USAGE}");
                return;
            }
            _ => pos.push(a.clone()),
        }
    }
    let data_of = |d: &str| data.clone().unwrap_or_else(|| read(d));
    // With --data, the DATA argument may be left out.
    if data.is_some() && (pos.len() == 1 || (pos.len() == 2 && pos[0] == "json")) {
        pos.push(String::new());
    }
    match pos.iter().map(|s| s.as_str()).collect::<Vec<_>>().as_slice() {
        ["lsp"] => {
            let stdin = std::io::stdin();
            if let Err(e) = qsystems::lsp::serve(&mut stdin.lock(), &mut std::io::stdout()) {
                eprintln!("qsys: {e}");
                exit(1)
            }
        }
        ["json", p, d] => println!("{}", play(&read(p), &data_of(d), opts.limits).to_text()),
        ["check", p] => match systems(&read(p)) {
            Ok(sys) => {
                for (i, s) in sys.iter().enumerate() {
                    println!(
                        "PHASE {}{}{}",
                        i + 1,
                        if s.det { "  -DET-" } else { "" },
                        if s.inv { "  -INV- (rules shown inverted)" } else { "" }
                    );
                    for c in &s.comments {
                        println!("  {c}.");
                    }
                    for (k, r) in s.rules.iter().enumerate() {
                        println!("  {:>3}. {r}", k + 1);
                    }
                }
            }
            Err(e) => {
                eprintln!("qsys: {p}: {e}");
                exit(1)
            }
        },
        [p, d] => match run_all(&read(p), &data_of(d), opts.limits) {
            Ok((sys, runs)) => print!("{}", printout(&sys, &runs, opts)),
            Err(e) => {
                eprintln!("qsys: {e}");
                exit(1)
            }
        },
        _ => {
            eprint!("{USAGE}");
            exit(2)
        }
    }
}
