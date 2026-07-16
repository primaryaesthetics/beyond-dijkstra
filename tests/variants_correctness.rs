//! Tier-2 gate for the variants track: bit-exact distances vs the crate's
//! own Dijkstra. Settlement order is NOT pinned here — that is the
//! mainline differential gate's job. Coverage: the three generator
//! regimes, a tie-rich integer-weight stress with ≥ 1e6 edges, and the
//! hub/star family the V1 falsification review prescribed (unbounded
//! degree is exactly what the no-transform variant admits).

mod common;

use beyond_dijkstra::variants::{fast, no_transform};

fn check(n: usize, edges: &[(u32, u32, f64)], source: usize, tag: &str) {
    let expect = common::dijkstra_lengths(n, edges, source);
    let run = no_transform::sssp(n, edges, source, None);
    assert_dists(n, &run.dist, &expect, tag);
    // The forced-small-parameter regime drives the recursion deepest.
    let forced = no_transform::sssp(n, edges, source, Some((3, 2, 2)));
    assert_dists(n, &forced.dist, &expect, &format!("{tag} (forced t=k=2)"));
    // V3 hybrid at interior and total levels (99 >= any l_top: one
    // bounded tuple-label Dijkstra).
    for hl in [1u64, 2, 99] {
        let hy = no_transform::sssp_hybrid(n, edges, source, hl, None);
        assert_dists(n, &hy.dist, &expect, &format!("{tag} (hybrid L={hl})"));
        let hyf = no_transform::sssp_hybrid(n, edges, source, hl, Some((3, 2, 2)));
        assert_dists(
            n,
            &hyf.dist,
            &expect,
            &format!("{tag} (hybrid L={hl}, forced t=k=2)"),
        );
    }
    // V4 fast engine across the same regimes.
    for hl in [0u64, 1, 99] {
        let fa = fast::sssp(n, edges, source, hl, None);
        assert_dists(n, &fa.dist, &expect, &format!("{tag} (fast L={hl})"));
        let faf = fast::sssp(n, edges, source, hl, Some((3, 2, 2)));
        assert_dists(
            n,
            &faf.dist,
            &expect,
            &format!("{tag} (fast L={hl}, forced t=k=2)"),
        );
    }
    // V5 fixed base case, both engines, deepest-recursion regimes.
    for hl in [0u64, 1] {
        let v5 = no_transform::sssp_v5(n, edges, source, hl, Some((3, 2, 2)));
        assert_dists(n, &v5.dist, &expect, &format!("{tag} (v5 L={hl})"));
        let f5 = fast::sssp_fixed(n, edges, source, hl, Some((3, 2, 2)));
        assert_dists(n, &f5.dist, &expect, &format!("{tag} (v5-fast L={hl})"));
    }
    // V8 settled bit, natural and deepest-recursion regimes; duplicate
    // settlements must be zero by construction.
    for hl in [0u64, 1] {
        let v8 = no_transform::sssp_v8(n, edges, source, hl, None);
        assert_dists(n, &v8.dist, &expect, &format!("{tag} (v8 L={hl})"));
        assert_eq!(v8.dup_settlements, 0, "{tag} (v8 L={hl}): dups");
        let v8f = no_transform::sssp_v8(n, edges, source, hl, Some((3, 2, 2)));
        assert_dists(
            n,
            &v8f.dist,
            &expect,
            &format!("{tag} (v8 L={hl}, forced t=k=2)"),
        );
        assert_eq!(v8f.dup_settlements, 0, "{tag} (v8f L={hl}): dups");
        // V8 in the fast engine.
        let f8 = fast::sssp_dedup(n, edges, source, hl, Some((3, 2, 2)));
        assert_dists(n, &f8.dist, &expect, &format!("{tag} (v8-fast L={hl})"));
        assert_eq!(f8.dup_settlements, 0, "{tag} (v8-fast L={hl}): dups");
    }
    // V9 flat top runner (VARIANTS.md): must match Dijkstra bit-exactly
    // and the v8-fast degenerate member's distances.
    let f9 = fast::sssp_flat(n, edges, source);
    assert_dists(n, &f9.dist, &expect, &format!("{tag} (v9-flat)"));
    let f8top = fast::sssp_dedup(n, edges, source, 99, None);
    assert_dists(n, &f9.dist, &f8top.dist, &format!("{tag} (v9 vs v8-fast L=top)"));
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
        // V3 hybrid at an interior level exercises the base-case-at-l>0
        // path together with a live parent loop above it.
        let hy = no_transform::sssp_hybrid(g.n, &g.edges, g.source, 1, g.override_params);
        assert_dists(g.n, &hy.dist, &expect, &format!("seed {seed} (hybrid L=1)"));
        // V4 fast engine, full recursion and interior hybrid.
        for hl in [0u64, 1] {
            let fa = fast::sssp(g.n, &g.edges, g.source, hl, g.override_params);
            assert_dists(g.n, &fa.dist, &expect, &format!("seed {seed} (fast L={hl})"));
        }
        // V5 fixed base case (full recursion).
        let v5 = no_transform::sssp_v5(g.n, &g.edges, g.source, 0, g.override_params);
        assert_dists(g.n, &v5.dist, &expect, &format!("seed {seed} (v5)"));
        let f5 = fast::sssp_fixed(g.n, &g.edges, g.source, 0, g.override_params);
        assert_dists(g.n, &f5.dist, &expect, &format!("seed {seed} (v5-fast)"));
        // V8 settled bit (full recursion), zero duplicates by construction.
        let v8 = no_transform::sssp_v8(g.n, &g.edges, g.source, 0, g.override_params);
        assert_dists(g.n, &v8.dist, &expect, &format!("seed {seed} (v8)"));
        assert_eq!(v8.dup_settlements, 0, "seed {seed} (v8): dups");
        let f8 = fast::sssp_dedup(g.n, &g.edges, g.source, 0, g.override_params);
        assert_dists(g.n, &f8.dist, &expect, &format!("seed {seed} (v8-fast)"));
        assert_eq!(f8.dup_settlements, 0, "seed {seed} (v8-fast): dups");
        // V9 flat top runner.
        let f9 = fast::sssp_flat(g.n, &g.edges, g.source);
        assert_dists(g.n, &f9.dist, &expect, &format!("seed {seed} (v9-flat)"));
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
    for hl in [2u64, 99] {
        let hy = no_transform::sssp_hybrid(n, &edges, 0, hl, None);
        assert_dists(
            n,
            &hy.dist,
            &expect,
            &format!("tie-rich 1e6-edge stress (hybrid L={hl})"),
        );
    }
    for hl in [0u64, 2, 99] {
        let fa = fast::sssp(n, &edges, 0, hl, None);
        assert_dists(
            n,
            &fa.dist,
            &expect,
            &format!("tie-rich 1e6-edge stress (fast L={hl})"),
        );
    }
}

/// Q10 stress: zero-heavy tie-rich graphs sized just under the l_top
/// sawtooth minimum at forced t = 2, where the |U| cutoff's slack over n
/// is smallest (limit = 8n). Re-settlements (Q3 escapes) count against
/// that cutoff; the entry points' totality assert turns any overflow
/// into a loud failure, and this family maximizes the pressure.
#[test]
fn q10_resettlement_pressure() {
    for &(n, mult) in &[(4094usize, 6), (16382, 4)] {
        let m = mult * n;
        let mut r = common::SplitMix64::new(0x0010_C0DE + n as u64);
        let mut edges = Vec::with_capacity(m);
        for _ in 0..m {
            let u = (r.next_u64() % n as u64) as u32;
            let v = (r.next_u64() % n as u64) as u32;
            let w = if r.next_u64() % 3 == 0 { 1.0 } else { 0.0 }; // zero-heavy
            edges.push((u, v, w));
        }
        let expect = common::dijkstra_lengths(n, &edges, 0);
        for hl in [0u64, 1, 2] {
            let run = no_transform::sssp_hybrid(n, &edges, 0, hl, Some((3, 2, 2)));
            assert_dists(n, &run.dist, &expect, &format!("q10 n={n} L={hl}"));
            let fa = fast::sssp(n, &edges, 0, hl, Some((3, 2, 2)));
            assert_dists(n, &fa.dist, &expect, &format!("q10 n={n} L={hl} (fast)"));
            let v5 = no_transform::sssp_v5(n, &edges, 0, hl, Some((3, 2, 2)));
            assert_dists(n, &v5.dist, &expect, &format!("q10 n={n} L={hl} (v5)"));
            let v8 = no_transform::sssp_v8(n, &edges, 0, hl, Some((3, 2, 2)));
            assert_dists(n, &v8.dist, &expect, &format!("q10 n={n} L={hl} (v8)"));
            assert_eq!(v8.dup_settlements, 0, "q10 n={n} L={hl} (v8): dups");
            let f8 = fast::sssp_dedup(n, &edges, 0, hl, Some((3, 2, 2)));
            assert_dists(n, &f8.dist, &expect, &format!("q10 n={n} L={hl} (v8-fast)"));
            assert_eq!(f8.dup_settlements, 0, "q10 n={n} L={hl} (v8-fast): dups");
        }
    }
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
