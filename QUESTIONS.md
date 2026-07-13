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

## Q9 — Lemmas 3.1/3.2/3.7 are false as standalone contracts. OPEN

Found during the V1 (no-degree-reduction) falsification review, and
independent of that variant — it concerns the paper as written. Concrete
instance: vertices {s, x, z, v}; edges s→x (10), s→z (10), x→v (5),
z→v (1); labels d[s] = 0, d[x] = 10 (complete), d[z] = d[v] = ∞. Call
BMSSP(B = 20, S = {x}, l ≥ 1) with k ≥ 3. The stated precondition holds:
⟨∅, {x}⟩ is a frontier for Ũ(20, {x}) = {x} — v's true shortest path
(length 11, via z) avoids S entirely. But FindPivots relaxes x→v to
d[v] = 15, the search fails (< k vertices), W = {x, v}, and the call
returns U = {x, v} with d[v] = 15 ≠ dis(v) = 11: Lemma 3.2's "W ⊆ Ũ"
and Lemma 3.7's "U = Ũ(B′, S) is complete" are both violated.

The lemmas are true in every state reachable from a top-level run: the
§2.5 loop keeps every incomplete-but-relevant vertex reachable from a
complete vertex present in D ∪ P̂, and the Pull + pivot-expansion step
forces that vertex into the child's S. This closure invariant is what
the induction actually rides on; the lemma statements omit it. Not a
bug in the algorithm — a gap between the stated contracts and the
proved property. Letter/note material.

Practical consequence for this repo: correctness testing must drive the
top-level entry only; property tests that call bmssp mid-recursion with
synthetic frontiers can fail (or pass) without indicting anything.

## Q10 — top-level totality is unproved: re-settlements count against the |U| cutoff. OPEN

Found during the V3 falsification review; applies to the paper as
written, at every hybrid level including the unmodified algorithm. The
top call is a full execution only if the |U| ≤ t³·2^(l_top·t) cutoff
never fires, and t³·2^(l_top·t) ≥ t³·n. But U counts settlement EVENTS,
not vertices: a Q3 escape (a label below the returned B′ that the child
left in D) lets a vertex settle with a stale label, improve later, and
settle again. The paper bounds neither the count of such re-settlements
nor argues the cutoff has slack for them. Empirically (248 adversarial
cells over zero-heavy/tie-rich graphs, all hybrid levels, forced deep
recursion): zero mismatches and zero top-level partials, but up to
0.3–0.5 duplicate settlements per vertex, roughly linear in n — against
a worst-case slack of (t³−1)·n ≈ 7n at the l_top sawtooth minimum
(n just below 2^(l_top·t), t = 2). A ~5× empirical margin with no
proof. If the cutoff ever fired, the run would return silently wrong
distances.

Mitigation in this repo: both entry points now assert the top call
returned B′ = ∞ and D = ∅ — the only silent failure mode is loud at
zero cost. Sub-finding: a base-case pull that empties D returns
bound = B, and later relaxations can refill D, so a child can return
B′ = B with D ≠ ∅ — contradicting Lemma 3.1's "full execution ⇒ D = ∅"
(Q3 sub-case; the parent handles it). Letter/note material: the missing
lemma is a bound on re-settlements per vertex.

## Q11 — measured duplicate settlements contradict Lemma 3.8's disjoint-union accounting; the base case is NOT the channel. CONFIRMED (mechanism traced)

Follow-up to Q10 with new measurements (raw record
benchmarks/results/dup_settlements_v5.txt). Two facts:

1. The V5 fixed base case (B′ capped at dB[D], restoring the Lemma 3.1
   contract — VARIANTS.md V5, proof included) fires **zero times** on
   every adversarial input tried (zero-heavy tie-rich at the Q10
   minimum-slack sizes, forced deep recursion, and large random): the
   base-case B′-undercut that Q3/Q10 presumed to be the re-settlement
   channel never actually materializes at return time.
2. Duplicate settlements in the top-level log exist anyway — up to
   608 events at n = 16 382 (0.037/n) and 5 074 at n = 170 000 — and
   are bit-identical with and without the V5 fix.

So the re-settlement channel is inside Algorithm 3 itself, not the base
case. The initial hypothesis blamed Relax's `≤` accepting an exact
4-tuple tie as True (Q6's "idempotent store"), re-inserting a settled
vertex into D to be pulled and settled again.

**Mechanism addendum (traced end-to-end; raw record
`benchmarks/results/q11_trace.txt`, driver `examples/q11_trace.rs`,
`q11-trace` build feature).** The first duplicated vertex of the
zero-heavy min-slack run (v5 at n = 4094, log positions 2571 and 3595
of 4087) shows a chain the hypothesis only partially anticipated:

1. **Seed — a stale D entry, no tie involved.** v5 is first reached at
   value (1, h7) and legitimately inserted into the level-5 call's D.
   Its label then improves to (0, h7) inside a sibling subtree's
   FindPivots, and v5 settles THERE (base case, settlement #1). The
   level-5 D still holds (1, h7): keyed keep-min dedup is per-D-
   instance, and nothing in the paper deletes or updates entries of
   vertices completed in other branches — there is no cross-level
   decrease-key.
2. **Re-frontier.** The level-5 final pull surfaces the stale entry and
   hands the already-complete v5 to a child's frontier S. Pull has no
   completeness check — the disjointness items of Lemma 3.8 implicitly
   assume D never yields a completed vertex, and this pull falsifies
   exactly that.
3. **The complete vertex stays structurally alive.** Exact-tie
   `≤`-re-acceptances (the Q6 channel — same edge, identical tuple)
   make relax() return True from and into completed vertices, so v5's
   local searches still grow K-trees: it roots trees, becomes an
   initial pivot at levels 4 and 3 (two more D inserts of a complete
   vertex), and keeps descending.
4. **Second U event via W′, not the base case.** At level 2 its search
   finally fails (|K| = 1 < k), v5 joins W; the W′ filter excludes only
   THIS call's u_set — settlement #1 happened in a different branch and
   is invisible — and d[v5] < B′ holds, so the line-34 boundary append
   emits v5 into U again (settlement #2).

So the confirmed picture: the duplicate's seed is a stale outer-D entry
pulled after an out-of-band improvement + settlement; exact-tie
re-acceptance is the enabling channel that (a) carried the vertex into
the sibling subtree's trees for settlement #1 and (b) keeps completed
vertices indistinguishable from live ones afterwards; and the extra U
event is emitted by Algorithm 3's own W′ path, whose completeness
filter is call-local. Consequence stands as measured: Lemma 3.8's items
"the U_X's at the same layer are disjoint" and "U_X is a disjoint union
of W′_X and the children's U's" are false as stated on tie-rich inputs,
and the |U| accounting that Lemma 3.9 and top-level totality rest on
counts these extra events with no bound in the paper. The missing lemma
must bound re-frontierings of completed vertices (stale-entry pulls and
the pivot re-inserts they enable), not base-case B′ escapes. Still
open: evaluate a guarded strict-< (the first project's counterexample
warns this is exactly where cross-call equivalence intuitions die —
full class-2 treatment required). Letter/note material, pairs with Q10.

## Q7 — nEdges overflow bound. NOTE

nEdges strictly increases along relaxations and is bounded by the longest
path in the reduced graph (< n₂). i32 suffices below n₂ = 2³¹ (same
assert as the first project's Label packing); the zero-weight cycles make
nEdges of practical labels larger than hop counts of the original graph
(up to +|C_v| detours), still < n₂.
