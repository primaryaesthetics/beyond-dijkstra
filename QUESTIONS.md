# QUESTIONS — audit log for arXiv:2602.07868 (v2)

Numbered findings/ambiguities from close reading. Status: OPEN until
resolved by proof, by the authors, or empirically by the reference
implementation. (The first project's QUESTIONS.md produced 14 entries;
this file follows the same discipline.)

## Q1 — prescribed δ is outside its own validity range on all sparse inputs. OPEN

§2.1 requires 3 ≤ δ ≤ m/n. Lemma 3.9 prescribes δ = ¼·min{m/n, log log n}.
For any graph with m/n < 12 the prescription gives δ < 3 at every size n —
e.g. our standard m = 4n input yields δ = 1 always. The paper never says
to clamp. Reference behavior: clamp to δ = max(3, ⌊¼·min{m/n, loglog n}⌋)
and note that with the clamp the analysis's δ ≤ log k assumption (§3,
footnote 1 says the algorithm still works without it) is violated at all
practical sizes: n = 10⁶ gives t ≈ 6, k ≈ 3, log₂k ≈ 1.6 < 3 ≤ δ.
Consequence if confirmed: the stated optimal parameterization is
unreachable for sparse graphs; an implementation is always in the
"analysis-degraded" regime. Candidate letter/note material.

## Q2 — empty P_j after partition. OPEN

Algorithm 2 line 22 defines P_j from subtree F_j, but a Θ(k)-sized subtree
of a merged F̄ far from any root can contain no S-vertices at all (F̄
vertices are mostly locally-discovered non-frontier vertices), or lose all
its S-vertices to the j-minimality rule. Lemma 3.2's "subsets {P_j} of S
each of size O(k)" silently permits |P_j| = 0, but Algorithm 3 line 6 then
takes argmin over an empty set. Reference behavior: drop empty P_j before
returning. Harmless for correctness (frontier condition still holds via
the other P_j and W) but the pseudocode as written crashes.

## Q3 — base-case B′ vs. vertices inserted after the last pull. OPEN

Algorithm 4 returns B′ from the final Pull, but the relaxations performed
after that pull (line 8) can insert labels smaller than B′ when zero- or
tiny-weight edges exist: pull u with B′ = next-smallest, relax (u,v) with
w = 0 giving d[v] = (d[u].length, ...) which is < B′. Then the returned D
contains values below the returned B′, violating Lemma 3.7's dB[D] ≥ B′
and Lemma 3.1's U = eU(B′, S) (v with dis(v) < B′ is complete-able but not
in U). Zero-weight edges are guaranteed to exist — the degree reduction
itself introduces them (§2.1). Suspected fix: exclude such v from the
violation by capping B′ down to min(B′, min inserted value), or pull-check
ordering as in DMM+25's base case. Reference behavior: implement literally
first, let the differential property tests decide, then adopt and document
the minimal fix. HIGH PRIORITY — this is the same bug class as the first
paper's Remark 3.4 tie cascade (found by us, worst bug of that project).

## Q4 — merged F̄ shape. RESOLVED (tree, by first-contact argument)

Algorithm 2 line 11 merges the in-progress K into an existing F̄_j on first
contact. Union of two trees joined at exactly one vertex is a tree; K
cannot share any other vertex with F̄_j because an earlier contact would
have triggered the merge earlier. Multiple merges chain the same argument.
The reference asserts acyclicity of every F̄ in debug mode anyway.

## Q5 — W′ completeness when Q-arborescences overlap F̄ trees. OPEN

Remark 3.3 admits that ∪F_j and the W arborescences are not disjoint.
Lemma 3.7's proof that W′ is complete leans on ⟨W, P_ini⟩ being a frontier;
verify the argument survives a vertex sitting in both an arborescence W_j
and a partitioned subtree F_j' (settled via the F̄ side while W thinks it
owns it). To exercise: graphs where local searches collide heavily —
small-diameter tie-rich inputs.

## Q6 — Relax's `<=` and duplicate DS insertions. RESOLVED (safe by design)

Algorithm 1 keeps `<=` (as DMM+25's Remark 3.4 did — the source of the tie
cascade we found there). Here the exposure is structurally closed: (a) the
4-tuple order is total, and exact tuple equality requires the SAME edge
(u = pred), so an equality-True is an idempotent store; (b) every DS
insertion is keyed with keep-smaller dedup, so duplicate True's cannot
multiply entries — the first project's cascade began exactly when a keyed
dedup map was optimized away. Standing rule for the port and all variants:
**the keyed dedup on D is load-bearing; removing it reintroduces the
cascade bug class. Tie-rich stress test from day one.**

## Q8 — Algorithm 3 crashes when P_j empties after j entered J. CONFIRMED (paper bug)

Found empirically within the first 150 random float graphs: lines 17–20
add j to J when the pivot p_j is settled and P_j is still non-empty, but a
LATER u ∈ U_i in the same pass can remove the remaining members of that
P_j. Line 28 then takes argmin over an empty set. Nothing in the paper
guards this; the pseudocode as written crashes. Reference fix (minimal,
matching line 19's evident intent): at re-selection time, a j ∈ J whose
P_j has emptied is skipped and its pivot dies. Distances stay bit-exact vs
Dijkstra across the full property suite with this fix. Letter/note
material — first confirmed finding of this audit.

## Q3 addendum (2026-07-12)

The literal base case survives 570+ property graphs including all-zero
weights: final distances stay bit-exact vs Dijkstra, so any dB[D] ≥ B′
violation self-heals in the parent loop rather than corrupting labels.
Contract status stays OPEN pending an invariant-check run (verify mode)
that asserts Lemma 3.7's properties per call.

## Q7 — nEdges overflow bound. NOTE

nEdges strictly increases along relaxations and is bounded by the longest
path in the reduced graph (< n₂). i32 suffices below n₂ = 2³¹ (same
assert as the first project's Label packing); the zero-weight cycles make
nEdges of practical labels larger than hop counts of the original graph
(up to +|C_v| detours), still < n₂.
