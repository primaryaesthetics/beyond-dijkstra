//! Property test: the port's readback lengths vs the crate's own Dijkstra,
//! bit-exact, across all three generator regimes (plain, tie-heavy,
//! forced-deep-recursion). This is the fast pure-Rust check; the
//! authoritative mainline gate is `tests/differential.rs`.

mod common;

use beyond_dijkstra::solver::sssp_run;

fn check_seed_range(seeds: std::ops::Range<u64>) {
    for seed in seeds {
        let g = common::gen_diff_graph(seed);
        let expect = common::dijkstra_lengths(g.n, &g.edges, g.source);
        let got = sssp_run(g.n, &g.edges, g.source, g.override_params).dist;
        for v in 0..g.n {
            assert!(
                got[v].to_bits() == expect[v].to_bits(),
                "seed {seed}: vertex {v}: port {} vs dijkstra {}",
                got[v],
                expect[v]
            );
        }
    }
}

#[test]
fn plain_graphs() {
    check_seed_range(0..120);
}

#[test]
fn tie_heavy_graphs() {
    check_seed_range(1000..1120);
}

#[test]
fn tie_heavy_forced_deep_recursion() {
    check_seed_range(2000..2120);
}

/// Tie-rich at scale: small integer weight domain, ~3e5 edges. Exact-tie
/// code paths are invisible on float-random weights (the first project's
/// worst bug was found exactly here), so the mainline keeps one
/// realistic-scale tie-rich member from day one.
#[test]
fn tie_rich_scale_stress() {
    let n: usize = 50_000;
    let m = 6 * n;
    let mut r = common::SplitMix64::new(0x71E_51CE);
    let mut edges = Vec::with_capacity(m);
    for _ in 0..m {
        let u = (r.next_u64() % n as u64) as u32;
        let v = (r.next_u64() % n as u64) as u32;
        let w = (r.next_u64() % 4) as f64; // integer weights 0..3: ties everywhere
        edges.push((u, v, w));
    }
    let expect = common::dijkstra_lengths(n, &edges, 0);
    let got = sssp_run(n, &edges, 0, None).dist;
    for v in 0..n {
        assert!(
            got[v].to_bits() == expect[v].to_bits(),
            "vertex {v}: port {} vs dijkstra {}",
            got[v],
            expect[v]
        );
    }
}
