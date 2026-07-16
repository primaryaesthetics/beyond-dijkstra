//! Q14 addendum 8 experiment: the base-exit triangle — build the input the
//! standard matrix never built and confirm that the incomplete-restricted
//! counter's measured zero was a family artifact, not an invariant.
//!
//! Mechanics under test: the base case fixes its returned B′ at the last
//! pull, BEFORE the final settlement's post-pull relaxation sweep inserts
//! (the Q3-flagged site). Give that last settlement u two out-edges w₁ → z
//! and w₂ → y plus an edge z → y with w₁ + w_zy < w₂, and the sweep plants
//! y's NON-canonical label d(u) + w₂ below B′ while y's true path runs
//! through z — a fresh entry of an INCOMPLETE vertex below B′, exactly what
//! the `incomplete_*` counters watch. Prediction (QUESTIONS.md Q14
//! addendum 8): the counter fires while distances stay exact and (H) — every
//! settlement settles a complete vertex, counted directly by
//! `settle_incomplete` — holds, because z's strictly smaller complete entry
//! escorts y through every D until a pull takes z first and completes y.
//!
//! Part 1: a crafted 12-vertex star, one base call (t = 2, cutoff 8) — the
//! deterministic live positive, leaked pair printed explicitly — then the
//! full recursion on the same graph showing the escort finishing the job.
//!
//! Part 2: the scaled family for the standard matrix — a unit path with the
//! triangle on every stride-th vertex and a ballast block above. stride = 0
//! is the plain-path control (escape geometry without the wave): the
//! unsettled counter fires there, the incomplete one must not. With
//! triangles, settlement period per stride path vertices is stride + 2;
//! stride ∈ {2, 3} keeps it coprime to the chunk size t³ + 1 = 9 so
//! chunk-end phases rotate through the triangle vertices.
//!
//! Every run is distance-checked against the crate's Dijkstra. The
//! `stale_incomplete_*` counters decide the escort invariant's companion
//! clause (addendum 8's defect ii): zero means fresh-only scope suffices on
//! these families.
//!
//! ```text
//! cargo run --release --features verify-contract,dup-classify --example triangle
//! ```

use beyond_dijkstra::label::{Lab, Label};
use beyond_dijkstra::solver::Solver;

fn dijkstra_lengths(n: usize, edges: &[(u32, u32, f64)], source: usize) -> Vec<f64> {
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
    let mut dist = vec![0f64; n];
    let mut pred = vec![0i32; n];
    let mut heap = beyond_dijkstra::dijkstra::Heap::new();
    let ok = beyond_dijkstra::dijkstra::dijkstra(
        &counts, &indices, &weights, source, &mut dist, &mut pred, &mut heap,
    );
    assert!(ok.is_ok(), "oracle rejected the triangle graph");
    dist
}

fn adj_of(n: usize, edges: &[(u32, u32, f64)]) -> Vec<Vec<(u32, f64)>> {
    let mut adj: Vec<Vec<(u32, f64)>> = vec![Vec::new(); n];
    for &(u, v, w) in edges {
        adj[u as usize].push((v, w));
    }
    adj
}

fn seed_source(solver: &mut Solver) {
    solver.d[0] = Label::Fin(Lab {
        len: 0.0,
        hops: 0,
        curr: 0,
        pred: -1,
    });
}

fn len_of(l: Option<Lab>) -> String {
    match l {
        Some(l) => format!("{}", l.len),
        None => "-".into(),
    }
}

/// Part 1 star: source 0, leaves 1..=8 at weights 1..8 (settlements 2..9 of
/// the cutoff-8 base loop), ballast 9 at weight 50 (the remaining D value
/// that sets B′ at the last pull), and the triangle off leaf 8:
/// z = 10 (w 1.25), y = 11 (w 2.5 direct, true 8 + 1.25 + 0.25 = 9.5).
fn star() -> (usize, Vec<(u32, u32, f64)>) {
    let mut edges: Vec<(u32, u32, f64)> = (1..=8u32).map(|i| (0, i, i as f64)).collect();
    edges.push((0, 9, 50.0));
    edges.push((8, 10, 1.25));
    edges.push((8, 11, 2.5));
    edges.push((10, 11, 0.25));
    (12, edges)
}

fn demonstrator() {
    let (n, edges) = star();
    let oracle = dijkstra_lengths(n, &edges, 0);
    let adj = adj_of(n, &edges);
    let b = Label::Fin(Lab {
        len: 100.0,
        hops: 0,
        curr: 0,
        pred: -1,
    });

    println!("Part 1a -- one base call on the star (t=2, cutoff 8, B=len 100):");
    println!(
        "{:>8} {:>7} {:>9} {:>10} {:>10} {:>10} {:>9} {:>4} {:>6} {:>6} {:>10}",
        "engine", "B'", "D[z=10]", "true z", "D[y=11]", "true y", "boundviol", "cap", "unsett", "incomp", "stale-inc"
    );
    for (name, fixed, dedup) in [("paper", false, false), ("V5", true, false), ("V8", false, true)] {
        let mut solver = Solver::new(&adj, 2, 2);
        solver.fixed_base = fixed;
        solver.settled_dedup = dedup;
        solver.oracle_len = Some(oracle.clone());
        seed_source(&mut solver);
        let (bp, _u, ds) = solver.bmssp(b, &[0], 0);
        println!(
            "{:>8} {:>7} {:>9} {:>10} {:>10} {:>10} {:>9} {:>4} {:>6} {:>6} {:>10}  settle-inc={} escort wards/missing={}/{}",
            name,
            bp.fin().len,
            len_of(ds.get(10)),
            oracle[10],
            len_of(ds.get(11)),
            oracle[11],
            solver.viol_bound_base,
            solver.base_cap_hits,
            solver.restr.unsettled_base,
            solver.restr.incomplete_base,
            solver.restr.stale_incomplete_base,
            solver.settle_incomplete,
            solver.escort.wards_base,
            solver.escort.missing_base,
        );
    }

    println!();
    println!("Part 1b -- full recursion on the star (B=INF, l_top=2):");
    for (name, fixed, dedup) in [("paper", false, false), ("V5", true, false), ("V8", false, true)] {
        let mut solver = Solver::new(&adj, 2, 2);
        solver.fixed_base = fixed;
        solver.settled_dedup = dedup;
        solver.oracle_len = Some(oracle.clone());
        seed_source(&mut solver);
        let (b_final, settle, ds_top) = solver.bmssp(Label::Inf, &[0], 2);
        assert!(b_final == Label::Inf && !ds_top.non_empty(), "top partial (star)");
        let dist: Vec<f64> = solver
            .d
            .iter()
            .map(|l| match l {
                Label::Fin(x) => x.len,
                Label::Inf => f64::INFINITY,
            })
            .collect();
        assert_eq!(dist, oracle, "distances diverged (star, {name})");
        let mut seen = vec![false; n];
        let mut dups = 0usize;
        for &v in &settle {
            if seen[v as usize] {
                dups += 1;
            }
            seen[v as usize] = true;
        }
        let rc = &solver.restr;
        let ec = &solver.escort;
        println!(
            "  [{name:>5}] dups={dups} settle-inc={} unsettled base/rec/iter={}/{}/{} incomplete base/rec/iter={}/{}/{} stale-inc base/rec/iter={}/{}/{} escort wards base/rec/iter={}/{}/{} missing={}/{}/{}",
            solver.settle_incomplete,
            rc.unsettled_base,
            rc.unsettled_rec,
            rc.unsettled_iter,
            rc.incomplete_base,
            rc.incomplete_rec,
            rc.incomplete_iter,
            rc.stale_incomplete_base,
            rc.stale_incomplete_rec,
            rc.stale_incomplete_iter,
            ec.wards_base,
            ec.wards_rec,
            ec.wards_iter,
            ec.missing_base,
            ec.missing_rec,
            ec.missing_iter,
        );
    }
}

/// Part 2 family: source 0 → gate 1 (w 10) → unit path p_i = 2 + i,
/// dis(p_i) = 11 + i, i = 0..L−1; the triangle on every stride-th path
/// vertex (z at +1.25, y at +2.5 direct / +1.5 true, z → y at 0.25);
/// E_A ballast leaves off the source at values above everything.
fn triangle_graph(l_path: usize, stride: usize, e_a: usize) -> (usize, Vec<(u32, u32, f64)>) {
    let mut edges: Vec<(u32, u32, f64)> = Vec::new();
    let path0 = 2u32;
    edges.push((0, 1, 10.0));
    edges.push((1, path0, 1.0));
    for i in 0..(l_path as u32 - 1) {
        edges.push((path0 + i, path0 + i + 1, 1.0));
    }
    let mut id = path0 + l_path as u32;
    if stride > 0 {
        for i in (0..l_path).step_by(stride) {
            let p = path0 + i as u32;
            edges.push((p, id, 1.25)); // z
            edges.push((p, id + 1, 2.5)); // y, true = dis(p) + 1.5
            edges.push((id, id + 1, 0.25));
            id += 2;
        }
    }
    let v0 = (l_path + 20) as f64;
    for g in 0..e_a {
        edges.push((0, id, v0 + g as f64));
        id += 1;
    }
    (id as usize, edges)
}

fn family() {
    println!("Part 2 -- the scaled family (full recursion, t=2, k=2):");
    println!(
        "{:>7} {:>7} {:>6} {:>8} {:>7}  per-engine counters below",
        "L", "stride", "E_A", "n", "tris"
    );
    for &(l_path, stride, e_a) in &[
        (200usize, 0usize, 500usize), // plain-path control: no triangles
        (200, 2, 500),
        (5000, 2, 3000),
        (5000, 3, 3000),
        (20000, 2, 3000),
    ] {
        let (n, edges) = triangle_graph(l_path, stride, e_a);
        let oracle = dijkstra_lengths(n, &edges, 0);
        let adj = adj_of(n, &edges);
        let (t, k) = (2u64, 2u64);
        let log_n = (n.max(2) as f64).log2().max(1.0);
        let l_top = (log_n / t as f64).ceil() as u64;
        let tris = if stride > 0 { l_path.div_ceil(stride) } else { 0 };
        println!("{:>7} {:>7} {:>6} {:>8} {:>7}", l_path, stride, e_a, n, tris);

        let mut v8_dups = 0usize;
        for (name, fixed, dedup) in
            [("paper", false, false), ("V5", true, false), ("V8", false, true)]
        {
            let mut solver = Solver::new(&adj, t, k);
            solver.fixed_base = fixed;
            solver.settled_dedup = dedup;
            solver.oracle_len = Some(oracle.clone());
            seed_source(&mut solver);
            let (b_final, settle, ds_top) = solver.bmssp(Label::Inf, &[0], l_top);
            assert!(
                b_final == Label::Inf && !ds_top.non_empty(),
                "top partial at L={l_path} stride={stride}"
            );
            let dist: Vec<f64> = solver
                .d
                .iter()
                .map(|l| match l {
                    Label::Fin(x) => x.len,
                    Label::Inf => f64::INFINITY,
                })
                .collect();
            assert_eq!(dist, oracle, "distances diverged at L={l_path} stride={stride}");
            let mut seen = vec![false; n];
            let mut dups = 0usize;
            for &v in &settle {
                if seen[v as usize] {
                    dups += 1;
                }
                seen[v as usize] = true;
            }
            if dedup {
                v8_dups = dups;
            }
            let rc = &solver.restr;
            let ec = &solver.escort;
            println!(
                "  [{name:>5}] dups={dups} settle-inc={} viol base/rec={}/{} cap={} unsettled base/rec/iter={}/{}/{} incomplete base/rec/iter={}/{}/{} stale-inc base/rec/iter={}/{}/{} escort wards base/rec/iter={}/{}/{} missing={}/{}/{}",
                solver.settle_incomplete,
                solver.viol_bound_base,
                solver.viol_bound_rec,
                solver.base_cap_hits,
                rc.unsettled_base,
                rc.unsettled_rec,
                rc.unsettled_iter,
                rc.incomplete_base,
                rc.incomplete_rec,
                rc.incomplete_iter,
                rc.stale_incomplete_base,
                rc.stale_incomplete_rec,
                rc.stale_incomplete_iter,
                ec.wards_base,
                ec.wards_rec,
                ec.wards_iter,
                ec.missing_base,
                ec.missing_rec,
                ec.missing_iter,
            );
        }
        assert_eq!(v8_dups, 0, "V8 must have zero dups");
    }
}

fn main() {
    demonstrator();
    println!();
    family();
}
