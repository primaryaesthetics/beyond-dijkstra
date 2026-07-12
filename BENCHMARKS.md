# Benchmarks — mainline baseline (pre-optimization)

> Epoch note: the table below is the FIRST epoch (the untouched faithful
> port). The current mainline state after optimization passes lives in
> OPTIMIZATION.md's per-pass tables (latest: pass 3 — 182× @1e5,
> 170× @1e6, 38× NY). Prose quoting "the" ratio must name its epoch.

Method: same-process ratios only (`examples/bench_fast.rs` — both methods
in one binary invocation, median of 5 runs after a warmup, distances
cross-checked bit-exactly inside the binary). Absolute times are quoted
for context and are not comparable across sessions or machines; the ratio
is the number that travels. Raw outputs live in `benchmarks/results/`.

Machine for this table: MacBook Air (Apple Silicon, macOS), 2026-07-12.

## Faithful mainline vs the crate's Dijkstra

The mainline is the literal port gated against the Python reference —
no optimization passes yet. Its data structure D deliberately mirrors the
reference's observable semantics (keyed map, full sort on every Pull), so
these numbers are the honest floor, not the algorithm's potential.

| input | n | m | dijkstra | mainline | ratio | raw record |
|---|---|---|---|---|---|---|
| random m=4n | 1e5 | 4e5 | 0.009 s | 4.354 s | 468× | bench_fast_baseline_1e5.txt |
| random m=4n | 1e6 | 4e6 | 0.209 s | 65.007 s | 311× | bench_fast_baseline_1e6.txt |
| USA-road-d.NY | 264346 | 733846 | 0.012 s | 1.166 s | 98× | bench_fast_baseline_NY.txt |

Parameters the port chooses at these sizes (printed by the harness): all
three inputs land on δ=3, t=6, k=3, l_top=4 (n2 = 8e5 / 8e6 / 1.47e6).
The prescribed δ = ¼·min{m/n, log log n} clamps to its floor of 3 at
every one of these inputs — the parameter-degeneracy hypothesis of
ASSESSMENT.md holds so far and will be re-examined at the variants stage.

Observations worth carrying forward:

- NY (integer weights, naturally tie-rich, tiny frontiers) runs clean —
  no tie cascade: the keyed dedup in Insert does its structural job.
  Its ratio (98×) is *better* than the random-graph ratios, unlike the
  first project where the road graph was the worst case; plausibly the
  full-sort Pull dominates so strongly at m=4e6 that frontier effects
  are drowned out. Re-measure after the Pull is replaced before drawing
  any input-class conclusion.
- The ratio improves 468× → 311× from 1e5 to 1e6; Dijkstra's own cost
  grows superlinearly with cache pressure while the port's overhead is
  mostly per-operation constants. Still nowhere near a crossover, as
  predicted (the bounds cross near n ≈ 2^1000).

## Notes

- DIMACS road graphs have integer weights — naturally tie-rich; the
  large diameter means tiny frontiers, so recursion overhead is not
  amortized (the input class that reversed random-graph conclusions in
  the first project).
- `benchmarks/data/` is not tracked; fetch USA-road-d.NY.gr from the 9th
  DIMACS Implementation Challenge distribution and gunzip it there.
