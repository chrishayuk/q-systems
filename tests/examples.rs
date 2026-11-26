//! Every example, run, and compared with its .out file.

use qsystems::machine::Limits;
use qsystems::report::{printout, run_all, Opts};

#[test]
fn examples() {
    let mut n = 0;
    for e in std::fs::read_dir("examples").unwrap() {
        let p = e.unwrap().path();
        if p.extension().is_none_or(|x| x != "q") {
            continue;
        }
        let prog = std::fs::read_to_string(&p).unwrap();
        let data = std::fs::read_to_string(p.with_extension("dat")).unwrap();
        let want = std::fs::read_to_string(p.with_extension("out")).unwrap();
        let (sys, runs) = run_all(&prog, &data, Limits::default()).unwrap();
        assert_eq!(printout(&sys, &runs, Opts::default()), want, "{}", p.display());
        n += 1;
    }
    assert!(n >= 7);
}
