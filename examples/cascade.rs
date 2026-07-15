//! Q14 addendum 2 experiment: attempt to IGNITE the re-settlement cascade —
//! branch (a) of the dichotomy. The construction: a long unit-weight path
//! p_1 → … → p_L (dis(p_i) = i), one decoy trigger per path vertex (an
//! early relaxation s → d_i → p_i that plants a stale D entry of p_i at
//! value x_i far above the whole path range, accepted while d[p_i] = ∞),
//! and G ballast leaves per trigger gap so consecutive triggers land in
//! different pull batches. When progress reaches x_i, the stale entry
//! re-frontiers the settled p_i; its re-settlement re-offers the canonical
//! edge (p_i, p_{i+1}) as an exact tie below the local bound, which
//! re-inserts and re-settles p_{i+1}, and the wave rolls to the end of the
//! path — reproduction 1 per Lemma A, but length Θ(L) per trigger, so the
//! naive prediction is Σ_i (L−i+1) = Θ(L²) excess.
//!
//! MEASURED OUTCOME (recorded in benchmarks/results/cascade.txt): the
//! cascade ignites — waves roll the full path (q11-trace run structure:
//! ascending runs to the path end) and the excess reaches 0.56–3.8 events
//! per vertex, one to four orders above natural inputs — but every growth
//! knob plateaus: dup/n is flat in L at fixed parameters, flat in forced t
//! at fixed l_top, and flat in l_top at fixed t (constant to three decimals
//! over 250× in n). Concurrent wavefronts merge through the keyed dedup and
//! the label-order re-sorting at descent, and the number of separated waves
//! per vertex converges to a parameter-dependent constant. Θ(L²) does NOT
//! materialize through this family.
//!
//! Every completed run is distance-checked bit-exactly against the crate's
//! Dijkstra. Runs whose excess exceeds the pinned event-count ceiling
//! t³·2^(l_top·t) trip the top-level totality assert — reported as
//! CEILING TRIPPED (that outcome is itself the set-semantics silent-slowdown
//! demonstration, made loud by the Q13 deviation).
//!
//! ```text
//! cargo run --release --features dup-classify --example cascade
//! ```

use beyond_dijkstra::variants::no_transform;

/// Path length L, ballast-per-gap G → (n, edges). Vertex ids:
/// 0 = s; 1..=L = path; L+1..=2L = decoys; rest = ballast.
fn cascade_graph(l_path: usize, g_ballast: usize) -> (usize, Vec<(u32, u32, f64)>) {
    let l = l_path;
    let n = 1 + 2 * l + g_ballast * l;
    let mut edges: Vec<(u32, u32, f64)> = Vec::new();

    let x0 = (l + 10) as f64; // trigger values sit above the whole path range
    let sp = 2.0; // gap between consecutive trigger values

    // Path: s → p_1, p_i → p_{i+1}, unit weights. dis(p_i) = i.
    edges.push((0, 1, 1.0));
    for i in 1..l {
        edges.push((i as u32, (i + 1) as u32, 1.0));
    }

    // Decoys: s → d_i (settles at eps_i, long before the path reaches p_i),
    // d_i → p_i offering exactly x_i while d[p_i] is still infinite.
    for i in 1..=l {
        let d = (l + i) as u32;
        let eps = 1e-7 * i as f64;
        let x_i = x0 + (i as f64 - 1.0) * sp;
        edges.push((0, d, eps));
        edges.push((d, i as u32, x_i - eps));
    }

    // Ballast: G leaves per gap, values strictly inside (x_i, x_{i+1}),
    // separating consecutive triggers in pull rank.
    let mut next_id = (2 * l + 1) as u32;
    for i in 1..=l {
        let x_i = x0 + (i as f64 - 1.0) * sp;
        for g in 1..=g_ballast {
            let w = x_i + sp * g as f64 / (g_ballast + 1) as f64;
            edges.push((0, next_id, w));
            next_id += 1;
        }
    }
    assert_eq!(next_id as usize, n);
    (n, edges)
}

/// Dijkstra oracle over the same edge list (inline CSR construction).
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
    assert!(ok.is_ok(), "oracle rejected the cascade graph");
    dist
}

fn main() {
    println!(
        "{:>7} {:>8} {:>9} {:>8} {:>9} {:>9} {:>8} {:>9} {:>8} {:>10}",
        "L", "n", "dups", "dup/n", "base-seed", "base-ins", "own-root", "own-swept", "foreign", "predicted"
    );
    let g_ballast = 8;
    sweep_fixed_params(g_ballast);
    println!();
    sweep_sawtooth_peaks(g_ballast);
}

/// Part 1: L-sweep at fixed parameter regimes — shows ignition and the
/// batch-merging plateau (waves roll full length, but only ~n/M_top of
/// them stay separated, so dup/n plateaus between l_top jumps).
fn sweep_fixed_params(g_ballast: usize) {
    for &l in &[50usize, 100, 200, 400, 800] {
        let (n, edges) = cascade_graph(l, g_ballast);
        let oracle = dijkstra_lengths(n, &edges, 0);
        let predicted = l * (l + 1) / 2;
        // Forced (3, 2, 2): the deep-recursion regime the minslack
        // experiments pinned; natural params reported separately below.
        for &(tag, ov) in &[("(3,2,2)", Some((3u64, 2u64, 2u64))), ("natural", None)] {
            let result = std::panic::catch_unwind(|| no_transform::sssp_hybrid(n, &edges, 0, 0, ov));
            match result {
                Ok(run) => {
                    assert_eq!(run.dist, oracle, "distances diverged at L={l} {tag}");
                    let c = &run.dup_class;
                    println!(
                        "{:>7} {:>8} {:>9} {:>8.3} {:>9} {:>9} {:>8} {:>9} {:>8} {:>10}  {tag} dist-ok",
                        l,
                        n,
                        run.dup_settlements,
                        run.dup_settlements as f64 / n as f64,
                        c.base_seeded,
                        c.base_inserted,
                        c.wprime_own_root,
                        c.wprime_own_swept,
                        c.wprime_foreign,
                        predicted,
                    );
                }
                Err(_) => {
                    println!(
                        "{:>7} {:>8} {:>9} {:>8} {:>9} {:>9} {:>8} {:>9} {:>8} {:>10}  {tag} CEILING TRIPPED (top guard: excess exceeded t^3*2^(l_top*t) - n)",
                        l, n, "-", "-", "-", "-", "-", "-", "-", predicted,
                    );
                }
            }
        }
    }
}

/// Part 2: the scaling probes. Two candidate laws for the number of
/// separated waves — #waves ~ n/M_top = Θ(2^t/t) at sawtooth peaks
/// (growing-t block) and #waves ~ l_top (growing-l block) — BOTH refuted
/// by the measurements: waves/vertex saturates at a size-independent,
/// parameter-dependent constant (~5.6 at t=2, ~30–38 at t=4..7).
fn sweep_sawtooth_peaks(g_ballast: usize) {
    println!(
        "{:>3} {:>3} {:>9} {:>9} {:>9} {:>8} {:>12} {:>10}",
        "t", "k", "L", "n", "dups", "dup/n", "2^t/(10t)", "waves"
    );
    // (t, l_top): n targeted at 2^(l_top·t) − 1, the sawtooth peak.
    // First block: growing t at fixed l_top (tests #waves ~ 2^t/t).
    // Second block: growing l_top at fixed t (tests #waves ~ l_top).
    for &(t, l) in &[
        (4u64, 3u64),
        (5, 3),
        (6, 3),
        (7, 3),
        (4, 4),
        (4, 5),
        (2, 6),
        (2, 8),
        (2, 10),
    ] {
        let k = ((t as f64) / (t as f64).log2()).ceil() as u64;
        let n_target = (1usize << (t * l) as usize) - 1;
        let l_path = (n_target - 1) / (2 + g_ballast);
        let (n, edges) = cascade_graph(l_path, g_ballast);
        let oracle = dijkstra_lengths(n, &edges, 0);
        let result = std::panic::catch_unwind(|| {
            no_transform::sssp_hybrid(n, &edges, 0, 0, Some((3, t, k)))
        });
        match result {
            Ok(run) => {
                assert_eq!(run.dist, oracle, "distances diverged at t={t}");
                let waves = run.dup_settlements as f64 / l_path as f64;
                println!(
                    "{:>3} {:>3} {:>9} {:>9} {:>9} {:>8.3} {:>12.2} {:>10.1}  dist-ok",
                    t,
                    k,
                    l_path,
                    n,
                    run.dup_settlements,
                    run.dup_settlements as f64 / n as f64,
                    (1u64 << t) as f64 / (10.0 * t as f64),
                    waves,
                );
            }
            Err(_) => {
                println!("{:>3} {:>3} {:>9} {:>9}  CEILING TRIPPED", t, k, l_path, n);
            }
        }
    }
}
