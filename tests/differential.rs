//! The mainline gate: differential test of the Rust port vs the pure-Python
//! reference (`reference/reference.py`) on 300 random graphs (n up to 5000,
//! including tie-heavy and forced-deep-recursion regimes). The final label
//! array over the reduced graph, the settlement log of the top-level call,
//! the chosen parameters, and the readback lengths must all match exactly
//! (bit-for-bit on the f64s) — see SPEC.md "Determinism pins".
//!
//! Protocol: this test runs every graph in Rust, writes per-seed results to
//! a file, then invokes `tests/diff_driver.py`, which regenerates the
//! identical graphs from the seeds (draw-for-draw mirror of
//! `common::gen_diff_graph`), runs the reference, and compares. On
//! divergence the driver reports the first differing index with context and
//! exits nonzero. The algorithm has no RNG, so unlike the first project's
//! harness there is nothing to pin beyond iteration orders.

mod common;

use beyond_dijkstra::label::Label;
use beyond_dijkstra::solver::sssp_run;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 300 by default: 100 per regime (plain / tie-heavy / forced-deep).
/// Override with BEYOND_DIJKSTRA_DIFF_GRAPHS + _DIFF_BASE for debugging.
fn num_graphs() -> u64 {
    std::env::var("BEYOND_DIJKSTRA_DIFF_GRAPHS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100)
}

fn base_seeds() -> Vec<u64> {
    match std::env::var("BEYOND_DIJKSTRA_DIFF_BASE")
        .ok()
        .and_then(|s| s.parse().ok())
    {
        Some(b) => vec![b],
        None => vec![0, 1000, 2000],
    }
}

fn find_python(manifest: &Path) -> Option<PathBuf> {
    if let Ok(p) = std::env::var("BEYOND_DIJKSTRA_PYTHON") {
        let p = PathBuf::from(p);
        if p.exists() {
            return Some(p);
        }
    }
    for candidate in [".venv/bin/python", ".venv/Scripts/python.exe"] {
        let p = manifest.join(candidate);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn label_tok(l: &Label) -> String {
    match l {
        Label::Inf => "I".to_string(),
        Label::Fin(l) => format!("{:x}:{}:{}:{}", l.len.to_bits(), l.hops, l.curr, l.pred),
    }
}

#[test]
fn differential_vs_python_reference() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let Some(python) = find_python(&manifest) else {
        panic!(
            "no Python interpreter found (.venv missing and BEYOND_DIJKSTRA_PYTHON \
             not set); the mainline gate cannot run"
        );
    };

    let num_graphs = num_graphs();
    let mut seeds: Vec<u64> = Vec::new();
    for base in base_seeds() {
        seeds.extend(base..base + num_graphs);
    }

    let mut out = String::new();
    for &seed in &seeds {
        let g = common::gen_diff_graph(seed);
        let run = sssp_run(g.n, &g.edges, g.source, g.override_params);

        match g.override_params {
            Some((d, t, k)) => writeln!(out, "GRAPH {seed} OV {d} {t} {k}").unwrap(),
            None => writeln!(out, "GRAPH {seed}").unwrap(),
        }
        writeln!(
            out,
            "PARAMS {} {} {} {} {}",
            run.delta, run.t, run.k, run.l_top, run.n2
        )
        .unwrap();
        out.push_str("LABELS");
        for l in &run.labels {
            out.push(' ');
            out.push_str(&label_tok(l));
        }
        out.push('\n');
        out.push_str("SETTLE");
        for &v in &run.settle {
            write!(out, " {v}").unwrap();
        }
        out.push('\n');
        out.push_str("DIST");
        for d in &run.dist {
            write!(out, " {:x}", d.to_bits()).unwrap();
        }
        out.push('\n');
        writeln!(out, "END").unwrap();
    }

    let results_path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("differential.txt");
    std::fs::write(&results_path, out).expect("write results file");

    let driver = manifest.join("tests").join("diff_driver.py");
    let seed_args: Vec<String> = seeds.iter().map(|s| s.to_string()).collect();
    let output = Command::new(&python)
        .arg(&driver)
        .arg(&results_path)
        .args(&seed_args)
        .output()
        .expect("failed to spawn the Python differential driver");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "differential driver reported divergence or crashed \
         (status {:?})\n--- driver stdout ---\n{stdout}\n--- driver stderr ---\n{stderr}",
        output.status.code()
    );
    let expect = format!("ALL OK {}/{}", seeds.len(), seeds.len());
    assert!(
        stdout.contains(&expect),
        "driver did not confirm all graphs:\n{stdout}\n{stderr}"
    );
}
