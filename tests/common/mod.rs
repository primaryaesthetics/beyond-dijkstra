//! Shared helpers for the Rust integration tests.
#![allow(dead_code)]

/// SplitMix64 — mirrored bit-for-bit by `tests/diff_driver.py`.
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        SplitMix64 { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

/// A randomly generated directed graph for the differential test.
///
/// IMPORTANT: `gen_diff_graph` is mirrored, draw for draw, by
/// `gen_diff_graph` in `tests/diff_driver.py`; the Python driver
/// re-generates the same graphs from the seeds instead of parsing them from
/// a file. Any change here must be made there as well.
pub struct DiffGraph {
    pub n: usize,
    pub edges: Vec<(u32, u32, f64)>,
    pub source: usize,
    /// Forced (delta, t, k) — seeds >= 2000 use (3, 2, 2), the
    /// deep-recursion regime (l_top grows when t is forced small).
    pub override_params: Option<(u64, u64, u64)>,
}

pub fn gen_diff_graph(seed: u64) -> DiffGraph {
    let mut r = SplitMix64::new(seed ^ 0xB0D1_2602_0786_8AAA);
    // Seeds < 1000: the acceptance-criterion distribution proven on the
    // first project. Seeds >= 1000: tie-heavy stress shapes (weights on a
    // tiny grid, denser graphs) that hammer the 4-tuple tie-breaking and
    // the keyed dedup. Seeds >= 2000: tie-heavy AND forced small (delta, t,
    // k), the regime where the recursion is deepest and the pivot
    // bookkeeping / W-sweep branches dominate.
    let tie_heavy = seed >= 1000;
    let n = if tie_heavy {
        match seed % 4 {
            0 | 1 => 2 + (r.next_u64() % 120) as usize,
            _ => 50 + (r.next_u64() % 1500) as usize,
        }
    } else {
        match seed % 4 {
            0 => 1 + (r.next_u64() % 40) as usize,
            1 => 2 + (r.next_u64() % 459) as usize,
            _ => 500 + (r.next_u64() % 4501) as usize,
        }
    };
    let max_m = if tie_heavy { 6 * n as u64 } else { 3 * n as u64 };
    let m = (r.next_u64() % (max_m + 1)) as usize;
    let mut edges = Vec::with_capacity(m);
    for _ in 0..m {
        let u = (r.next_u64() % n as u64) as u32;
        let v = (r.next_u64() % n as u64) as u32;
        let w = if tie_heavy {
            // 8-value grid including 0.0: ties everywhere.
            (r.next_u64() % 8) as f64 / 8.0
        } else if r.next_u64() % 20 == 0 {
            0.0
        } else {
            ((r.next_u64() % 1_000_000) + 1) as f64 / 1e6
        };
        edges.push((u, v, w));
    }
    let source = (r.next_u64() % n as u64) as usize;
    DiffGraph {
        n,
        edges,
        source,
        override_params: if seed >= 2000 { Some((3, 2, 2)) } else { None },
    }
}

/// CSR form of an edge list for the crate's Dijkstra: counting sort by
/// source vertex, preserving input order within each vertex.
pub fn to_csr(n: usize, edges: &[(u32, u32, f64)]) -> (Vec<i64>, Vec<i32>, Vec<f64>) {
    let mut counts = vec![0i64; n + 1];
    for &(u, _, _) in edges {
        counts[u as usize + 1] += 1;
    }
    for i in 0..n {
        counts[i + 1] += counts[i];
    }
    let mut indices = vec![0i32; edges.len()];
    let mut weights = vec![0f64; edges.len()];
    let mut next = counts.clone();
    for &(u, v, w) in edges {
        let slot = next[u as usize] as usize;
        indices[slot] = v as i32;
        weights[slot] = w;
        next[u as usize] += 1;
    }
    (counts, indices, weights)
}

/// Shortest-path lengths via the crate's Dijkstra, for use as an oracle.
pub fn dijkstra_lengths(n: usize, edges: &[(u32, u32, f64)], source: usize) -> Vec<f64> {
    let (indptr, indices, weights) = to_csr(n, edges);
    let mut dist = vec![0f64; n];
    let mut pred = vec![0i32; n];
    let mut heap = beyond_dijkstra::dijkstra::Heap::new();
    let ok = beyond_dijkstra::dijkstra::dijkstra(
        &indptr, &indices, &weights, source, &mut dist, &mut pred, &mut heap,
    );
    assert!(ok.is_ok(), "dijkstra rejected a generated graph");
    dist
}
