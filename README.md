# beyond-dijkstra

The first implementation of the Duan–Mao–Shu–Yin directed shortest-path
algorithm ([arXiv:2602.07868](https://arxiv.org/abs/2602.07868), February 2026),
in Rust, with the two questions the paper leaves open answered by measurement
and by proof: does it run faster than Dijkstra, and does its correctness
argument hold up.

Short answers, both documented below. **It is not faster** — not at any size
you can store. And **the algorithm looks correct while one of its proofs is
not**: every engine here returns exact distances on every input we have run,
but two lines of the printed pseudocode crash on ordinary graphs, and the
completeness clause of Lemma 3.7 is false as stated — we exhibit a 70-vertex
graph where a settled vertex is incomplete. The paper's own engine still
finishes with the right answers there; nothing in the paper guarantees that it
will. The distinction matters because the clause is load-bearing: trust it and
optimize on it, as any implementer would, and you get a wrong distance. Ours
did.

This is the second study in a series. The first,
[logtwothirds](https://github.com/primaryaesthetics/logtwothirds), implemented
the 2025 paper that broke the sorting barrier
([arXiv:2504.17033](https://arxiv.org/abs/2504.17033)); this repository does the
same for the faster algorithm four of those five authors published ten months
later.

## The algorithm

Dijkstra's algorithm settles vertices in order of increasing distance. That
ordering is the trick and also the tax: producing `n` values in sorted order
costs `Ω(n log n)` comparisons, so any algorithm that hands you vertices in
distance order inherits a `log n` factor. On sparse graphs that term dominates
the clock. In 2025 Duan, Mao, Mao, Shu and Yin got below it —
`O(m log^(2/3) n)` — by never materializing the full sorted sequence: a
`FindPivots` step picks a small set of vertices whose settlement unlocks
everything behind them, and a divide-and-conquer recursion settles whole blocks
at once.

The 2026 paper sharpens the same recursion to
`O(m √(log n) + √(m n log n log log n))`, which is `O(m √(log n · log log n))`
on sparse graphs. Three things change. Labels become the 4-tuple
⟨length, hop count, current vertex, predecessor⟩ compared lexicographically, so
labels of distinct vertices never tie exactly — the authors' answer to the
tie-ambiguity class the previous paper left open. `FindPivots` runs bounded
local Dijkstra forests instead of `k` Bellman–Ford rounds, partitioned
Frederickson-style into Θ(k)-sized subtrees, which is where the improvement
comes from: `O(log k)` per vertex instead of `O(k)`. And the frontier data
structure becomes a BST of Θ(M) blocks, unsorted inside a block, with keyed
deduplication built into `Insert`.

`ASSESSMENT.md` has the full component inventory and the classification of
every lemma into correctness-carrying and time-analysis-only — the license list
the variants track works from.

## Is it faster

No. The gap is smaller than the first paper's, and it does not close.

The faithful port — the one gated bit-for-bit against the Python reference —
runs at these ratios against this crate's own Dijkstra, same process, median of
five, distances cross-checked inside the binary:

| input | n | m | ratio vs Dijkstra |
|---|---|---|---|
| random, m = 4n | 10⁵ | 4·10⁵ | 182× |
| random, m = 4n | 10⁶ | 4·10⁶ | 170× |
| USA-road-d.NY | 264 346 | 733 846 | 38× |

Those are after three behavior-neutral optimization passes (`OPTIMIZATION.md`;
the untouched first-epoch port was 468× / 311× / 98×, recorded in
`BENCHMARKS.md`). The mainline is frozen — it exists to be provably the paper's
algorithm, not to be fast.

The variants track (`VARIANTS.md`) then asks how much of that gap is the
algorithm and how much is faithfulness to the paper's bookkeeping. Nine
engines later (V1–V10; V6 stayed a hypothesis, see `VARIANTS.md`), the best
member runs at **1.8–2.2× of Dijkstra** across
recorded runs (V9, `benchmarks/results/bench_fast_v9_*.txt` and
`bench_fast_v5v8_*.txt`). It gets there by removing, one provable step at a
time, everything the recursion turns out not to need at practical parameters —
and what remains at the end is lazy Dijkstra carrying the paper's 16-byte
labels. That is the honest endpoint: the residual factor measures the label
machinery, nothing else.

The reason is in the parameters, and it is the first audit finding. The
prescribed `δ = ¼·min{m/n, log log n}` needs both `m ≥ 12n` and
`log log n ≥ 12` to satisfy the paper's own `3 ≤ δ ≤ m/n`, so on **every input
of any density below n = 2^4096** the prescription is outside its validity
range and an implementation must clamp. At every size we can run, all inputs
land on δ=3, t=6, k=3 — the recursion is four levels of overhead over a graph
it just made eight times bigger. The two bounds themselves cross near
`n ≈ 2^1000`; the observable universe holds roughly `2^266` atoms.

## Is it correct, and is it proved

This is where the second study diverges from the first, and the two halves of
the question have different answers. The distances are
right: every engine here is gated against Dijkstra bit-exactly over a property
matrix, a tie-rich 1.02·10⁶-edge stress, star/hub families, and zero-weight
pressure graphs. The *proofs* are another matter. `QUESTIONS.md` is the audit
log — fourteen numbered entries, each resolved by argument, by a driver that
exhibits the behavior, or left open and marked open. Three groups are worth a
reader's time.

**Two pseudocode-level crashes.** In Algorithm 3, a later `u ∈ U_i` in the same
pass can empty a `P_j` after `j` has already entered `J`, so the re-selection at
line 28 takes an argmin over an empty set. It shows up within the first 150
random float-weighted test graphs. The same happens at line 6 when the Θ(k)
partition produces a subtree containing no `S`-vertices; on hub-shaped graphs
most `P_j` are empty, so there it is the common case rather than a corner. Both
have one-line guards (Q2, Q8).

**A settlement that is not complete.** Lemma 3.7's completeness clause says
every vertex the algorithm settles carries its true distance at that moment.
The separator cap in the base case and the `W′` filter compose in a way that
lets an incomplete vertex through: `examples/rowh.rs` builds a 70-vertex graph
where the three engines separate, and one of our own variants — the settled-bit
variant, which is exact everywhere else — finishes with a wrong final distance
(23.59375 against the true 23.546875). The paper's engine heals through a
duplicate settlement; nothing in the paper says it must.

That is the whole reason this finding matters. The clause is not decorative —
it is what licenses the obvious optimization of never reopening a finished
vertex, and an implementer who takes the lemma at face value earns a wrong
answer for it. We know because we did. Recorded with drivers and
predictions-written-before-runs in Q14 addenda 13–15.

**Our own completeness proof.** Capping `B′` in the base case closes that hole
exactly, and `PROOF.md` is the linear write-up of what we can then prove:
on that engine every settlement settles a vertex carrying the lexicographic
minimum (length, hop count) over all source-to-vertex paths, and the label it
settles with is bit-final. It is a careful hand proof against a frozen
implementation — not machine-checked, and explicitly not a proof about the
paper's algorithm as written, which the counterexample above rules out. It took
thirty-two recorded addenda under one discipline: every step derived in one
session and re-derived from the code in a separate later session before
anything was built on it — eight derivation/audit pairs, written down as they
happened, falsifications included.

## Layout

| file | what it holds |
|---|---|
| `ASSESSMENT.md` | paper reading: component inventory, lemma classification, stage plan |
| `SPEC.md` | frozen reference semantics — tie tuple, pull order, dedup, determinism |
| `QUESTIONS.md` | the audit log, Q1–Q14 with addenda; every finding traceable to a driver |
| `PROOF.md` | the completeness theorem for the V5 engine |
| `VARIANTS.md` | V1–V10: claim, proof sketch, classification, gate, measurement |
| `OPTIMIZATION.md` | behavior-neutral passes on the frozen mainline |
| `BENCHMARKS.md` | first-epoch measurements and method |
| `reference/reference.py` | the Python reference the port is gated against |
| `src/solver.rs` | the faithful port |
| `src/variants/` | the research engines |
| `examples/` | benchmark harness, phase profiler, and one driver per finding |
| `benchmarks/results/` | raw run records; every number in these documents traces here |

## Running it

```bash
cargo test                       # differential gate + variant gates, 14 tests
cargo run --release --example bench_fast
```

`bench_fast` runs Dijkstra and the engines in one process on the same graph and
cross-checks distances before reporting medians. Road-graph rows need
`benchmarks/data/USA-road-d.NY.gr` from the
[9th DIMACS Implementation Challenge](http://www.diag.uniroma1.it/challenge9/download.shtml).

The library API is two entry points — `dijkstra::dijkstra` over CSR arrays, and
`solver::sssp_run` over an edge list — plus the variant engines under
`variants::`. Feature flags turn on the instrumentation the research record was
built with (`verify-contract`, `dup-classify`, `q11-trace`, `phase-timer`); all
of them are observation-only and zero-cost when off.

If you need shortest paths in production, use the Dijkstra. That is not modesty
about the port — it is the measured result, and it is the point of the
repository.

## License

MIT. The paper is the authors' work; this is an independent implementation and
audit of it.
