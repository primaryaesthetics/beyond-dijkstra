//! L-sweep for the V3 hybrid oracle (VARIANTS.md): for each hybrid level
//! L ∈ [0, l_top], levels ≤ L run the base case at the level's cutoff.
//! L = 0 is V1; L = l_top is one bounded tuple-label Dijkstra. Two
//! parameter settings per graph: the prescription and the V2 sweep's best
//! shallow setting. Median of 3 after warmup; every cell distance-checked
//! against the crate's Dijkstra.
//!
//! ```text
//! cargo run --release --example sweep_hybrid -- [n] [gr-file]
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
    let settings: [(Option<(u64, u64, u64)>, &str, u64); 2] = [
        (None, "prescription", auto.l_top),
        (Some((3, 12, 6)), "sweep-best (t=12, k=6)", {
            let log_n = (n.max(2) as f64).log2().max(1.0);
            (log_n / 12.0).ceil() as u64
        }),
    ];

    for (ov, name, l_top) in settings {
        println!("--- params: {name} (l_top={l_top}) ---");
        println!("{:>4} {:>10} {:>8}", "L", "median s", "x dj");
        for hl in 0..=l_top {
            let mut times = Vec::new();
            for _ in 0..4 {
                let t0 = Instant::now();
                let run = no_transform::sssp_hybrid(n, &edges, 0, hl, ov);
                times.push(t0.elapsed().as_secs_f64());
                let bad = dj_dist
                    .iter()
                    .zip(run.dist.iter())
                    .filter(|(a, b)| a.total_cmp(b) != std::cmp::Ordering::Equal)
                    .count();
                assert_eq!(bad, 0, "distances diverge at L={hl}");
            }
            times.remove(0);
            let med = median(times);
            println!("{hl:>4} {med:>10.4} {:>8.2}", med / dj);
        }
    }
}
