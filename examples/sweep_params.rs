//! Parameter sweep for the V1 no-transform variant (VARIANTS.md V2):
//! grid over (t, k) with l_top following the formula, median of 3 runs
//! per cell after a warmup, distances cross-checked against the crate's
//! Dijkstra in every cell. The paper's own footnote 3 says the prescribed
//! parameters are "best" in order-analysis terms only.
//!
//! ```text
//! cargo run --release --example sweep_params -- [n] [gr-file]
//! ```

use beyond_dijkstra::dijkstra;
use beyond_dijkstra::variants::no_transform;
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

    // Dijkstra oracle (also the ratio denominator).
    let (indptr, indices, weights) = {
        let mut indptr = vec![0i64; n + 1];
        for &(u, _, _) in &edges {
            indptr[u as usize + 1] += 1;
        }
        for i in 0..n {
            indptr[i + 1] += indptr[i];
        }
        let mut indices = vec![0i32; edges.len()];
        let mut ws = vec![0f64; edges.len()];
        let mut next = indptr.clone();
        for &(u, v, w) in &edges {
            let slot = next[u as usize] as usize;
            indices[slot] = v as i32;
            ws[slot] = w;
            next[u as usize] += 1;
        }
        (indptr, indices, ws)
    };
    let mut dj_dist = vec![0.0f64; n];
    let mut pred = vec![0i32; n];
    let mut heap = dijkstra::Heap::new();
    let mut dj_times = Vec::new();
    for _ in 0..4 {
        let t0 = Instant::now();
        dijkstra::dijkstra(&indptr, &indices, &weights, 0, &mut dj_dist, &mut pred, &mut heap)
            .ok()
            .expect("dijkstra");
        dj_times.push(t0.elapsed().as_secs_f64());
    }
    dj_times.remove(0);
    let dj = median(dj_times);
    println!("dijkstra median {dj:8.4} s");

    let auto = no_transform::sssp(n, &edges, 0, None);
    println!(
        "prescription: t={} k={} (l_top={})",
        auto.t, auto.k, auto.l_top
    );

    println!("{:>4} {:>4} {:>6} {:>10} {:>8}", "t", "k", "l_top", "median s", "x dj");
    for t in [2u64, 3, 4, 5, 6, 8, 10, 12] {
        for k in [2u64, 3, 4, 6, 8, 12] {
            let mut times = Vec::new();
            let mut l_top = 0;
            for _ in 0..4 {
                let t0 = Instant::now();
                let run = no_transform::sssp(n, &edges, 0, Some((3, t, k)));
                times.push(t0.elapsed().as_secs_f64());
                l_top = run.l_top;
                let bad = dj_dist
                    .iter()
                    .zip(run.dist.iter())
                    .filter(|(a, b)| a.total_cmp(b) != std::cmp::Ordering::Equal)
                    .count();
                assert_eq!(bad, 0, "distances diverge at t={t} k={k}");
            }
            times.remove(0); // warmup
            let med = median(times);
            let mark = if t == auto.t && k == auto.k { "  <- prescription" } else { "" };
            println!(
                "{t:>4} {k:>4} {l_top:>6} {med:>10.4} {:>8.2}{mark}",
                med / dj
            );
        }
    }
}
