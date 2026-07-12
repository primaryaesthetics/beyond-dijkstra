# Optimization record — frozen mainline

Discipline (as proven on logtwothirds): the mainline stays semantically
frozen after the differential gate went green. Every change here is
classified BEFORE the code is written:

- **behavior-neutral** — same computation, different representation; the
  claim names the invariants it leans on; gate = the full differential
  test (bit-exact labels + settlement log vs the Python reference).
- anything that changes an observable order or a float operation is NOT
  an optimization and goes to the variants track instead.

One change per commit, gate green before the next, ratio in the commit
message. Baseline: BENCHMARKS.md (468× @1e5, 311× @1e6, 98× NY).

## Pass 1 — ordered D: replace sort-on-pull with a value-ordered tree

**Claim.** Replacing `BlockDs`'s (hash map + full sort on every Pull)
with a pair (value→key ordered tree, key→value hash map) changes no
observable output.

**Proof sketch.** The structure's observables are Pull's return (the
min(M, |D|) keys ordered by ascending stored value, plus the separating
bound = the smallest remaining stored value, else the construction B)
and non-emptiness. These are functions of the map *content* only. The
invariants:

1. Per key, the stored value is the minimum of all values inserted for
   that key (keep-min Insert — identical in both representations).
2. Stored values are unique across keys — `curr` is a component of the
   value, so distinct keys can never carry equal values; the value→key
   tree therefore loses nothing and its ascending iteration is exactly
   the sorted order the reference computes.
3. Content evolution is identical: Insert/Merge/Pull mutate content the
   same way in both representations (Merge is a fold of keep-min
   Inserts, which is order-independent per key, so hash-iteration order
   during Merge remains unobservable).

Bound equivalence: reference bound = (M+1)-th smallest of pre-Pull
content when |D| > M, else B. After popping min(M, |D|) items, the
remainder is non-empty iff |D| > M, and its minimum is exactly that
(M+1)-th smallest.

**Classification.** Behavior-neutral. **Gate:** full differential test.

**Result.** Gate green (300-graph differential, bit-exact). Raw records
`benchmarks/results/bench_fast_pass1_*.txt`:

| input | before | after | speedup |
|---|---|---|---|
| random 1e5 | 468× | 270× | 1.7× |
| random 1e6 | 311× | 258× | 1.2× |
| USA-road-d.NY | 98× | 65× | 1.5× |

The smaller gain at 1e6 says the full sort was not the dominant cost at
scale — per-operation constants elsewhere (recursion bookkeeping,
allocation churn, FindPivots) now carry the profile. Phase attribution
before the next structural pass.

## Pass 2 — mimalloc as the global allocator

**Claim.** Swapping the global allocator changes no observable output.

**Proof sketch.** The allocator decides where memory lives, never what
is computed; no code path reads addresses, capacities beyond reserves,
or allocation order. Proven-safe category from the first project (same
recursion shape: tens of millions of small short-lived Vec/map
allocations per run).

**Classification.** Behavior-neutral. **Gate:** full differential test.

**Result.** Gate green. Raw records
`benchmarks/results/bench_fast_pass2_*.txt`:

| input | before (pass 1) | after | speedup |
|---|---|---|---|
| random 1e5 | 270× | 238× | 1.13× |
| random 1e6 | 258× | 240× | 1.08× |
| USA-road-d.NY | 65× | 42× | 1.56× |

NY gains the most — tiny frontiers mean the run is dominated by
constructing and dropping small per-call structures, exactly the
allocation pattern mimalloc is good at. Next candidates, in order:
phase attribution (port the phase-timer pattern) BEFORE any further
structural work; then scratch-buffer reuse across recursion levels
(proven-safe category) wherever the profile points.

## Phase attribution (after pass 2)

`cargo run --release --features phase-timer --example profile_phases`;
raw record `benchmarks/results/profile_phases_after_pass2.txt`:

| input | find_pivots | of it: partition | base_case | relax sweep | rest |
|---|---|---|---|---|---|
| random 1e5 | 79.9% | 1.7% | 5.3% | 8.0% | ~7% |
| random 1e6 | 87.7% | ~2% | 2.6% | 4.8% | ~5% |
| USA-road-d.NY | 27.8% | — | 20.6% | 24.5% | ~27% |

On random graphs the local Dijkstra searches inside FindPivots are the
whole story (the tree partition is noise); NY is spread evenly. So the
next structural pass targets the search inner loop, and NY conclusions
should not be drawn from random-graph passes.

## Pass 3 — FindPivots: epoch-stamped membership + scratch reuse

**Claim.** Replacing FindPivots' per-call/per-search hash maps and sets
(`tree_of`, `k_set`, `k_parent_idx`, `q_set`, `w_set`, `prior`, `gset`,
per-tree `set`) with solver-owned flat arrays stamped by a monotone epoch
counter, and reusing the search heap/vectors across searches, changes no
observable output.

**Proof sketch.**

1. Every replaced structure is observed ONLY through membership queries
   (`contains`/`insert`-dedup/`get`) — never iterated. The orders that
   reach the output all come from Vecs that stay untouched (`k_vert`,
   `k_parent_order`, `q_list`, `w_list`, `s_list`, tree/group order).
   An epoch-stamped array answers every membership query identically
   (stamp[v] == current epoch ⟺ v inserted since this set instance began);
   distinct set instances take distinct epochs, so stale stamps can never
   leak across searches, calls, or groups.
2. `|K|` loop threshold: `k_vert.len()` equals the old `k_set.len()` —
   both grow by exactly one on every first-time insert (x seeds both).
3. Per-tree `vert`/`set` elimination: the reference's per-tree vertex
   list is write-only (only `edges` and `root` are ever read), and its
   set existed solely to dedup that list; the membership that actually
   matters flows through `tree_of`, which the stamps implement. Dropping
   write-only state is observation-free.
4. Scratch reuse (heap, `k_vert`, `k_parent_order`): cleared before every
   search; a cleared container is observationally a fresh one (proven-safe
   category). The edge buffer of a newly formed tree is cloned out — a
   Vec clone is elementwise-identical.
5. Epoch space (u32): at every call start, if fewer than 2³¹ epochs
   remain, all stamp arrays are zeroed and the counter restarts (no
   stamps are live between calls); a single call exhausting 2³¹ epochs
   is impossible without ≥ 2³¹ heap operations first, and `next_epoch`
   panics loudly rather than alias. Correctness never rests on wrap
   being unreachable. Scratch cost: eight u32 words per reduced vertex.

**Classification.** Behavior-neutral. **Gate:** full differential test.

**Result.** Gate green. Raw records
`benchmarks/results/bench_fast_pass3_*.txt`:

| input | before (pass 2) | after | speedup |
|---|---|---|---|
| random 1e5 | 238× | 182× | 1.31× |
| random 1e6 | 240× | 170× | 1.41× |
| USA-road-d.NY | 42× | 38× | 1.11× |

Post-pass profile (`profile_phases_after_pass3.txt`, 1e6): find_pivots
is still 82.8% — but its representation overhead is gone; what remains
is the recursion's intrinsic volume (every call runs a bounded local
search from every frontier vertex; Σ|S| across all calls × per-search
work). Constant-shaving the frozen mainline further has visibly
diminishing returns; the next big lever is algorithm-level and belongs
to the variants track — first candidate no-degree-reduction, which
would shrink the working graph 8× (n2 = 8e6 → n = 1e6 on the random
family) before any search starts.

## Rejected / known-noise ideas

(carried from the first project so they are not re-tried blindly:
prefetch hints in cold paths, bit-compare stale checks, PGO — all noise
there; re-test only with a specific hypothesis for why this workload
differs.)
