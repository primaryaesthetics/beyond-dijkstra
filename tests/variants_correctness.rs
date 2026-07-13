//! Tier-2 gate for the variants track: bit-exact distances vs the crate's
//! own Dijkstra. Settlement order is NOT pinned here — that is the
//! mainline differential gate's job. Coverage: the three generator
//! regimes, a tie-rich integer-weight stress with ≥ 1e6 edges, and the
//! hub/star family the V1 falsification review prescribed (unbounded
//! degree is exactly what the no-transform variant admits).

mod common;

use beyond_dijkstra::variants::no_transform;

fn check(n: usize, edges: &[(u32, u32, f64)], source: usize, tag: &str) {
    let expect = common::dijkstra_lengths(n, edges, source);
    let run = no_transform::sssp(n, edges, source, None);
    assert_dists(n, &run.dist, &expect, tag);
    // The forced-small-parameter regime drives the recursion deepest.
    let forced = no_transform::sssp(n, edges, source, Some((3, 2, 2)));
    assert_dists(n, &forced.dist, &expect, &format!("{tag} (forced t=k=2)"));
}

fn assert_dists(n: usize, got: &[f64], expect: &[f64], tag: &str) {
    for v in 0..n {
        assert!(
            got[v].to_bits() == expect[v].to_bits(),
            "{tag}: vertex {v}: variant {} vs dijkstra {}",
            got[v],
            expect[v]
        );
    }
}

fn check_seed_range(seeds: std::ops::Range<u64>) {
    for seed in seeds {
        let g = common::gen_diff_graph(seed);
        let expect = common::dijkstra_lengths(g.n, &g.edges, g.source);
        let run = no_transform::sssp(g.n, &g.edges, g.source, g.override_params);
        assert_dists(g.n, &run.dist, &expect, &format!("seed {seed}"));
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

/// Tie-rich at scale with >= 1e6 edges (standing rule: exact-tie code
/// paths are invisible on float-random weights).
#[test]
fn tie_rich_million_edge_stress() {
    let n: usize = 170_000;
    let m = 6 * n; // 1.02e6 edges
    let mut r = common::SplitMix64::new(0x5EED_0001);
    let mut edges = Vec::with_capacity(m);
    for _ in 0..m {
        let u = (r.next_u64() % n as u64) as u32;
        let v = (r.next_u64() % n as u64) as u32;
        let w = (r.next_u64() % 4) as f64;
        edges.push((u, v, w));
    }
    let expect = common::dijkstra_lengths(n, &edges, 0);
    let run = no_transform::sssp(n, &edges, 0, None);
    assert_dists(n, &run.dist, &expect, "tie-rich 1e6-edge stress");
}

// --- hub/star family (V1 falsification review) -------------------------
// Sizes straddle t^3 and t^3*2^t for the natural parameters these n's
// produce, so the hub's F-bar spans multiple partition pieces and levels.

fn star_edges(n: usize, hub: u32, zero_weight: bool) -> Vec<(u32, u32, f64)> {
    let mut edges = Vec::with_capacity(2 * (n - 1));
    for v in 0..n as u32 {
        if v == hub {
            continue;
        }
        let (wo, wi) = if zero_weight {
            (0.0, 0.0)
        } else {
            ((v % 3) as f64, (v % 5) as f64)
        };
        edges.push((hub, v, wo));
        edges.push((v, hub, wi));
    }
    edges
}

#[test]
fn star_hub_is_source() {
    for n in [50usize, 1_000, 20_000] {
        let edges = star_edges(n, 0, false);
        check(n, &edges, 0, &format!("star n={n}, hub=source"));
    }
}

#[test]
fn star_hub_not_source() {
    for n in [50usize, 1_000, 20_000] {
        let edges = star_edges(n, 0, false);
        check(n, &edges, 1, &format!("star n={n}, source=spoke"));
    }
}

#[test]
fn zero_weight_star() {
    for n in [50usize, 5_000] {
        let edges = star_edges(n, 0, true);
        check(n, &edges, 0, &format!("zero-weight star n={n}"));
    }
}

/// The review's closure motif at scale: a hub reached by a failed (< k)
/// search from x while a shorter path runs outside S through z, plus a
/// leaf fan so the parent D absorbs Theta(n) inserts at M = t.
#[test]
fn hub_behind_competing_paths() {
    for leaves in [100usize, 10_000] {
        let n = 4 + leaves; // s=0, x=1, z=2, hub=3, leaves 4..
        let mut edges = vec![
            (0u32, 1u32, 10.0f64),
            (0, 2, 10.0),
            (1, 3, 5.0),
            (2, 3, 1.0),
        ];
        for leaf in 4..n as u32 {
            edges.push((3, leaf, (leaf % 3) as f64));
        }
        check(n, &edges, 0, &format!("hub motif, {leaves} leaves"));
    }
}
