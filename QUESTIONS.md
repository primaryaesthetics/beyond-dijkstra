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

## Q3 — base-case B′ vs. vertices inserted after the last pull. CONFIRMED (contract violation demonstrated; V5 is the proved fix)

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

## Q5 — W′ completeness when Q-arborescences overlap F̄ trees. RESOLVED (overlap harmless by argument; fragility lives elsewhere)

Remark 3.3 admits that ∪F_j and the W arborescences are not disjoint.
Lemma 3.7's proof that W′ is complete leans on ⟨W, P_ini⟩ being a frontier;
verify the argument survives a vertex sitting in both an arborescence W_j
and a partitioned subtree F_j' (settled via the F̄ side while W thinks it
owns it). To exercise: graphs where local searches collide heavily —
small-diameter tie-rich inputs.

Resolution (2026-07-13, by argument against the proof text). Neither
proof uses disjointness anywhere:

1. Lemma 3.2's frontier conclusion needs only that every complete
   u ∈ S lands in some P_j or in W — membership, not exclusivity. The
   overlap corners check out: a u skipped at line 3 (already inside an
   F̄) is partitioned into some F_j and, being outside Q, survives into
   P_j; a u whose own search failed is the root of its arborescence,
   so u ∈ Q ⊆ W; a u in BOTH (failed own search, member of another
   root's F̄) is excluded from P_j by the S \ Q filter but covered by
   W. A vertex sitting in W_j and F_j' simultaneously satisfies the
   frontier disjunction through either side.
2. Lemma 3.7's W′-completeness step consumes ⟨W, P_ini⟩ only through
   "incomplete v ∈ W′ visits a complete vertex in P_ini" and then
   rides the loop invariant dB[D ∪ P] ≥ B′ to a contradiction with
   d[v] < B′. No tree structure enters at all.

So the overlap admitted by Remark 3.3 is harmless as such. The step's
real weak joint is the invariant it rides: dB[D ∪ P] ≥ B′ read over
CURRENT labels, which stale D entries falsify (a completed vertex
sitting in D at an outdated stored value with current label < B′ —
exactly the Q11 state; see also Q12 for the stored-value leak). Worth
recording: if an incomplete vertex ever did enter W′, the line-36 gate
(candidate ≥ B′) would REFUSE the repairing relaxation — the plausible
empirical rescue is a later Q11 duplicate re-settling the upstream
vertex. No such incomplete-W′ state has been constructed; distances
are bit-exact vs Dijkstra across every suite, including the heavy-
collision families (star/hub, zero-heavy min-slack, tie-rich stress)
that exercise merged trees hardest.

## Q6 — Relax's `<=` and duplicate DS insertions. RESOLVED (safe by design) — UPGRADED: the `<=` is itself load-bearing

Algorithm 1 keeps `<=` (as DMM+25's Remark 3.4 did — the source of the tie
cascade we found there). Here the exposure is structurally closed: (a) the
4-tuple order is total, and exact tuple equality requires the SAME edge
(u = pred), so an equality-True is an idempotent store; (b) every DS
insertion is keyed with keep-smaller dedup, so duplicate True's cannot
multiply entries — the first project's cascade began exactly when a keyed
dedup map was optimized away. Standing rule for the port and all variants:
**the keyed dedup on D is load-bearing; removing it reintroduces the
cascade bug class. Tie-rich stress test from day one.**

Upgrade (2026-07-13, V7 verdict in VARIANTS.md): "idempotent store" was
the right safety argument but undersold the role. The `<=` is the
algorithm's ADOPTION mechanism: FindPivots labels vertices without
taking responsibility for their out-edges, and the only way an
early-labeled vertex later enters a D — hence ever settles, hence ever
relaxes its own out-edges — is the exact-tie re-acceptance when its
predecessor settles. Tightening Relax to strict-< orphans such vertices
and loses reachability (Tier-2 seed 0, a plain float graph: vertex left
at ∞). So the `<=` cannot be removed, and its cost — re-adopting
already-settled vertices, i.e. Q11's duplicate settlements — is a
built-in of the design as written, removable only by adding an explicit
completeness marking (VARIANTS.md V8).

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

## Q3 addendum (2026-07-13) — verify mode ran; CONFIRMED

The `verify-contract` build feature counts per-return violations of the
bound clause (dB[D] ≥ B′ over stored values, base-case and recursive
returns separately) and of the full-execution dichotomy. Recorded run:
`benchmarks/results/verify_contract.txt` (`examples/verify_contract.rs`).
Two results:

1. **Demonstrator (no ties needed).** A crafted star — nine leaves at
   weights 1..9, a zero-weight edge out of leaf 8, t = 2 so the cutoff
   is t³ = 8, B = len 100 — makes Algorithm 4 as written return
   dB[D] = 8 < B′ = 9: one bound violation. The V5 cap returns the
   repaired pair (zero violations, one cap hit). This is the concrete
   Lemma 3.1/3.7 counterexample for the letter.
2. **The adversarial family is contract-clean.** On the full
   dup-settlement family (zero-heavy min-slack 4094/16382, tie-rich
   170k, zero-heavy 500k; both engines) every counter is ZERO — no
   bound violation at any base-case or recursive return, no dichotomy
   violation. So the thousands of duplicate settlements measured there
   (Q11) arise with all return contracts intact: the re-settlement
   channel operates entirely below the bound clauses, and neither the
   Q12 leak nor Q10's B′ = B-with-D ≠ ∅ sub-case is exercised by this
   family. Q12 stays open on the argument, unobserved empirically.

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

## Q10 — top-level totality is unproved: re-settlements count against the |U| cutoff. REVISED (scope narrowed by Q13)

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

Addendum (2026-07-13): Q13 narrows this finding's scope. The paper's
line 30 unions U as a SET, under which the top-level cutoff can never
fire (|U| ≤ n₂ < t³·2^(l_top·t)) — so the totality exposure as stated
above applies to the pinned event-count semantics of this repo's
reference, where the entry asserts guard it. The B′ = B with D ≠ ∅
sub-finding stands as written (it is semantics-independent). What
re-settlements break in the paper itself is the disjoint-union
accounting and the time analysis — see Q11 (confirmed mechanism and
concrete Lemma 3.8 witness) and Q12 (the related bound-validity gap).

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
filter is call-local.

Correction (2026-07-13, V8 measurement): the earlier "consistent with
dup ≈ 0 on float-random weights" line was wrong — 664 duplicates
measured at n = 5e5 float-random (0.0013/n, raw record
`benchmarks/results/dup_settlements_v8.txt`). Exact tuple equality
needs only a same-edge re-offer at an unchanged source label, which
continuous weights do not prevent; tie-rich inputs amplify the channel
(~25x), they do not create it. Operational closure exists: VARIANTS.md
V8 (per-vertex settled bit) removes every duplicate while keeping
distances bit-exact across the full matrix — and is measurably faster.
The `≤` itself cannot be touched (V7, REJECTED — it is the adoption
mechanism; see Q6 upgrade). Consequence stands as measured: Lemma 3.8's items
"the U_X's at the same layer are disjoint" and "U_X is a disjoint union
of W′_X and the children's U's" are false as stated on tie-rich inputs,
and the |U| accounting that Lemma 3.9 and top-level totality rest on
counts these extra events with no bound in the paper. The missing lemma
must bound re-frontierings of completed vertices (stale-entry pulls and
the pivot re-inserts they enable), not base-case B′ escapes. Still
open: evaluate a guarded strict-< (the first project's counterexample
warns this is exactly where cross-call equivalence intuitions die —
full class-2 treatment required). Letter/note material, pairs with Q10.

## Q12 — Line 29's unguarded insert can break Lemma 3.7's dB[D] ≥ B′ in partial executions; the invariant proof skips it. OPEN

Found while writing the transitive contract-restoration argument (V5
follow-up, VARIANTS.md). Algorithm 3 has exactly one unguarded D-insert:
line 24 is gated by `d[u] + wuv ≥ Bi`, lines 31–37 by `≥ B′`, but the
re-selection insert (line 29, `D.Insert(⟨pj, d[pj]⟩)`) has no bound
check. Two channels can put d[pj] below B′_i at that point:

1. **Line 23 pseudocode vs prose.** The pseudocode reads
   `if Relax(u, v, B) and d[u] + wuv ≥ Bi` — the relaxation runs with
   bound B and mutates d[v] even when the candidate is below Bi; only
   the insert is gated. The prose restatement says "relax every edge
   (u, v) such that d[u] + wuv ∈ [Bi, B)" — no sub-Bi mutations. Under
   the pseudocode reading, a surviving member of a P_j with j ∈ J is
   skipped by line 25's pivot update but still takes the label
   decrease, and line 28's argmin then selects it.
2. **Child-internal decreases (either reading).** The i-th sub-call
   relaxes with bound Bi > B′_i and can lower a P_j member below B′_i
   without settling it (the member was ≥ Bi at pull time, so line 13
   left it out of S_i). The paper's own P̂ device acknowledges these
   vertices ("d[v] may decrease but v is not in Di or Ui") and removes
   them from P̂_j — but line 28 selects from P_j, not P̂_j.

The proof of Lemma 3.7's loop invariant 2 (dB[D ∪ P] ≥ B′_i) covers
inserts with the bare assertion "For any v added into D, d[v] ≥ B′_i" —
no argument is offered for line 29, and the P-side bullets protect only
vertices NOT inserted into D. If the |U| cutoff fires in the same
iteration (partial execution), the sub-B′ entry survives to the return
(earlier-iteration leaks are consumed by later pulls; only the final
iteration's leak can escape), violating clause 2 of Lemma 3.7
(B′ ≤ dB[D]) — the clause the parent's own bookkeeping consumes.
Reference behavior: SPEC pins the pseudocode reading of line 23.
Empirical check ran (2026-07-13, Q3 addendum): the verify-mode counter
for recursive returns stayed at zero across the full adversarial
family, both engines — the leak is reachable on the argument above but
was not exercised by these inputs; a dedicated construction would need
a partial execution whose final iteration re-selects a pivot lowered
below B′_f.

Sharpening (same day, from the attempted fix): the Q12 state admits NO
local repair at the return boundary — unlike Q3, which V5's cap fixes
with every clause intact. If X < B′_f was stored by the final
iteration's re-selection while some already-settled label L satisfies
X < L < B′_f, then Lemma 3.7's clauses are jointly unsatisfiable for
any reported B′: B′ > X breaks B′ ≤ dB[D], B′ ≤ X < L breaks
max dis(U) < B′; and gating the insert instead leaves an unsettled
vertex with dis < B′ that the completeness clause U = Ũ(B′, S) may
claim. Coherent repairs are statement-level only: adopt the prose
reading of line 23 (closes the sweep channel; the child-interior
channel survives) or weaken clause 2 to carve out final-iteration
premature entries whose keys' labels dropped during that iteration.
Full argument in VARIANTS.md ("Algorithm 3's contract, transitively").
Letter/note material — arguably the strongest single item: a
correctness contract that cannot be locally restored.

## Q13 — the loop guard's |U|: the paper counts distinct vertices, the pinned reference counts settlement events. OPEN (pinned deviation)

Line 30 is `U ← U ∪ Ui` — a set union — so line 9's `|U| ≤ t³·2^lt`
compares distinct-vertex cardinality; W′ (line 34) is defined via
`W \ U` on the same set. The reference pinned U as a list (the
settlement log; SPEC "BMSSP" pins) and the guard as its length. The two
agree exactly until a duplicate settlement (Q11) occurs; with
duplicates, event counts cross a level's cutoff earlier than distinct
counts. Consequences:

1. **Q10's top-level exposure belongs to the pinned semantics, not the
   paper's.** Under set counting the top-level guard can never fire:
   |U| ≤ n₂ while the cutoff is t³·2^(l_top·t) ≥ t³·n₂ (from
   l_top = ⌈log n₂ / t⌉), which is the paper's own remark
   "|U| ≤ |V| = o(t³n)". The top call is a full execution
   unconditionally — no re-settlement pressure on correctness there.
2. **What duplicates break in the paper** is therefore not top-level
   totality but the accounting: Lemma 3.8's disjoint unions (falsified
   concretely — Q11) and the Σ|U_X| time analysis of Lemma 3.9, plus
   per-level guards firing at different times than the paper's analysis
   assumes.

The deviation stays pinned (the differential gate depends on it); the
mainline totality asserts cover its only dangerous consequence.
Candidate micro-variant for completeness: guard on distinct count —
expected distance-neutral, changes recursion shape on tie-rich inputs.

## Q7 — nEdges overflow bound. NOTE

nEdges strictly increases along relaxations and is bounded by the longest
path in the reduced graph (< n₂). i32 suffices below n₂ = 2³¹ (same
assert as the first project's Label packing); the zero-weight cycles make
nEdges of practical labels larger than hop counts of the original graph
(up to +|C_v| detours), still < n₂.
