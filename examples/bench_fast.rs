//! Head-to-head timing of the production Dijkstra and the mainline port of
//! arXiv:2602.07868 on one graph (random m = 4n, weights in [0.01, 1),
//! source 0, or a DIMACS `.gr` file). Median of five runs each after a
//! warmup, plus a distance cross-check so a timing win can never hide a
//! wrong answer. Same-process ratios only — absolute times are not
//! comparable across sessions (thermal drift).
//!
//! ```text
//! cargo run --release --example bench_fast -- [n] [gr-file]
//! ```

use beyond_dijkstra::dijkstra;
use beyond_dijkstra::solver::sssp_run;
use std::time::Instant;

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        SplitMix64 { state: seed }
    }
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

fn dimacs_graph(path: &str) -> (usize, Vec<(u32, u32, f64)>) {
    let text = std::fs::read_to_string(path).expect("read .gr file");
    let mut n = 0usize;
    let mut edges = Vec::new();
    for line in text.lines() {
        let mut it = line.split_ascii_whitespace();
        match it.next() {
            Some("p") => {
                assert_eq!(it.next(), Some("sp"));
                n = it.next().unwrap().parse().unwrap();
            }
            Some("a") => {
                let u: u32 = it.next().unwrap().parse::<u32>().unwrap() - 1;
                let v: u32 = it.next().unwrap().parse::<u32>().unwrap() - 1;
                let w: f64 = it.next().unwrap().parse().unwrap();
                edges.push((u, v, w));
            }
            _ => {}
        }
    }
    (n, edges)
}

fn random_graph(n: usize, seed: u64) -> Vec<(u32, u32, f64)> {
    let m = 4 * n;
    let mut rng = SplitMix64::new(seed);
    let mut edges = Vec::with_capacity(m);
    for _ in 0..m {
        let u = (rng.next_u64() % n as u64) as u32;
        let v = (rng.next_u64() % n as u64) as u32;
        let w = 0.01 + (rng.next_u64() % 990_000) as f64 / 1_000_000.0;
        edges.push((u, v, w));
    }
    edges
}

fn to_csr(n: usize, edges: &[(u32, u32, f64)]) -> (Vec<i64>, Vec<i32>, Vec<f64>) {
    let mut indptr = vec![0i64; n + 1];
    for &(u, _, _) in edges {
        indptr[u as usize + 1] += 1;
    }
    for i in 0..n {
        indptr[i + 1] += indptr[i];
    }
    let mut indices = vec![0i32; edges.len()];
    let mut weights = vec![0f64; edges.len()];
    let mut next = indptr.clone();
    for &(u, v, w) in edges {
        let slot = next[u as usize] as usize;
        indices[slot] = v as i32;
        weights[slot] = w;
        next[u as usize] += 1;
    }
    (indptr, indices, weights)
}

fn median(mut xs: Vec<f64>) -> f64 {
    xs.sort_by(|a, b| a.total_cmp(b));
    xs[xs.len() / 2]
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n: usize = args.get(1).map(|s| s.parse().unwrap()).unwrap_or(1_000_000);
    let (n, edges) = match args.get(2) {
        Some(path) => {
            eprintln!("graph: {path}");
            dimacs_graph(path)
        }
        None => {
            eprintln!("graph: random m=4n, n={n}");
            (n, random_graph(n, 0xC0FFEE))
        }
    };
    eprintln!("n={n}, m={}", edges.len());

    let (indptr, indices, weights) = to_csr(n, &edges);
    let mut dist = vec![0.0f64; n];
    let mut pred = vec![0i32; n];
    let mut heap = dijkstra::Heap::new();
    let mut dj_times = Vec::new();
    for _ in 0..6 {
        let t0 = Instant::now();
        let r = dijkstra::dijkstra(&indptr, &indices, &weights, 0, &mut dist, &mut pred, &mut heap);
        let dt = t0.elapsed().as_secs_f64();
        assert!(r.is_ok());
        dj_times.push(dt);
    }
    dj_times.remove(0); // warmup
    let dj = median(dj_times);

    let mut port_times = Vec::new();
    let mut port_dist = Vec::new();
    for i in 0..6 {
        let t0 = Instant::now();
        let run = sssp_run(n, &edges, 0, None);
        let dt = t0.elapsed().as_secs_f64();
        port_times.push(dt);
        if i == 0 {
            eprintln!(
                "params: delta={} t={} k={} l_top={} n2={}",
                run.delta, run.t, run.k, run.l_top, run.n2
            );
        }
        port_dist = run.dist;
    }
    port_times.remove(0); // warmup
    let bf = median(port_times);

    let mismatches = dist
        .iter()
        .zip(port_dist.iter())
        .filter(|(a, b)| a.total_cmp(b) != std::cmp::Ordering::Equal)
        .count();
    assert_eq!(mismatches, 0, "port distances diverge from dijkstra");

    println!("dijkstra median {dj:8.3} s");
    println!("mainline median {bf:8.3} s   ({:.2}x dijkstra)", bf / dj);
}
