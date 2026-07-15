//! Scratch driver: trace one path vertex through the cascade construction
//! to see where the re-settlement wave front dies. Not part of the
//! recorded experiments; see `cascade.rs` for the measured runs.
//!
//! ```text
//! cargo run --release --features q11-trace --example cascade_trace [vertex]
//! ```

use beyond_dijkstra::label::{Lab, Label};
use beyond_dijkstra::solver::Solver;

fn cascade_graph(l_path: usize, g_ballast: usize) -> (usize, Vec<(u32, u32, f64)>) {
    let l = l_path;
    let n = 1 + 2 * l + g_ballast * l;
    let mut edges: Vec<(u32, u32, f64)> = Vec::new();
    let x0 = (l + 10) as f64;
    let sp = 2.0;
    edges.push((0, 1, 1.0));
    for i in 1..l {
        edges.push((i as u32, (i + 1) as u32, 1.0));
    }
    for i in 1..=l {
        let d = (l + i) as u32;
        let eps = 1e-7 * i as f64;
        let x_i = x0 + (i as f64 - 1.0) * sp;
        edges.push((0, d, eps));
        edges.push((d, i as u32, x_i - eps));
    }
    let mut next_id = (2 * l + 1) as u32;
    for i in 1..=l {
        let x_i = x0 + (i as f64 - 1.0) * sp;
        for g in 1..=g_ballast {
            let w = x_i + sp * g as f64 / (g_ballast + 1) as f64;
            edges.push((0, next_id, w));
            next_id += 1;
        }
    }
    (n, edges)
}

fn main() {
    let tv: u32 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(26);
    let l = 50usize;
    let (n, edges) = cascade_graph(l, 8);
    let mut adj: Vec<Vec<(u32, f64)>> = vec![Vec::new(); n];
    for &(u, v, w) in &edges {
        adj[u as usize].push((v, w));
    }
    let (t, k) = (2u64, 2u64);
    let log_n = (n.max(2) as f64).log2().max(1.0);
    let l_top = (log_n / t as f64).ceil() as u64;

    let mut solver = Solver::new(&adj, t, k);
    solver.trace_vertex = Some(tv);
    solver.d[0] = Label::Fin(Lab {
        len: 0.0,
        hops: 0,
        curr: 0,
        pred: -1,
    });
    let (_b, settle, _ds) = solver.bmssp(Label::Inf, &[0], l_top);
    let mut count = 0;
    for &v in &settle {
        if v == tv {
            count += 1;
        }
    }
    println!("vertex {tv}: {count} settlement events");

    // Path-vertex settlements in log order, compressed into maximal
    // ascending consecutive runs a..b — each run is one wave segment.
    let mut runs: Vec<(u32, u32)> = Vec::new();
    for &v in &settle {
        if v == 0 || v as usize > l {
            continue;
        }
        match runs.last_mut() {
            Some((_, b)) if *b + 1 == v => *b = v,
            _ => runs.push((v, v)),
        }
    }
    println!("path settlement runs (start-end), in log order:");
    let mut line = String::new();
    for &(a, b) in &runs {
        if a == b {
            line.push_str(&format!("{a} "));
        } else {
            line.push_str(&format!("{a}-{b} "));
        }
        if line.len() > 90 {
            println!("  {line}");
            line.clear();
        }
    }
    if !line.is_empty() {
        println!("  {line}");
    }
    let mut per_vertex = vec![0u32; l + 1];
    for &v in &settle {
        if v != 0 && v as usize <= l {
            per_vertex[v as usize] += 1;
        }
    }
    println!("dups per path vertex (index: events-1):");
    let hist: Vec<String> = (1..=l).map(|j| format!("{}", per_vertex[j].saturating_sub(1))).collect();
    println!("  {}", hist.join(" "));
}
