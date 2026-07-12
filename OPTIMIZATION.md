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

## Rejected / known-noise ideas

(carried from the first project so they are not re-tried blindly:
prefetch hints in cold paths, bit-compare stale checks, PGO — all noise
there; re-test only with a specific hypothesis for why this workload
differs.)
