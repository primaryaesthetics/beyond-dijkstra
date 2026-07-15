# QUESTIONS — audit log for arXiv:2602.07868 (v2)

Numbered findings/ambiguities from close reading. Status: OPEN until
resolved by proof, by the authors, or empirically by the reference
implementation. (The first project's QUESTIONS.md produced 14 entries;
this file follows the same discipline.)

## Q1 — prescribed δ is outside its own validity range on all sparse inputs. RESOLVED 2026-07-15 (sharpened to ALL inputs below n = 2^4096; the clamp is provably free, claimed bounds survive within 2×)

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

Resolution (2026-07-15) — the §3/Lemma 3.9 arithmetic re-run with the
clamp. Four findings; the net verdict is that the complexity CLAIMS
survive verbatim, only the prescription needs the clamp the paper
never states.

1. **The invalidity is universal, not sparse-only.** δ = ¼·min{m/n,
   loglog n} ≥ 3 requires BOTH m ≥ 12n AND loglog n ≥ 12, i.e.
   n ≥ 2^4096. With the paper's own constant ¼, the prescription
   violates §2.1's 3 ≤ δ ≤ m/n on EVERY input of ANY density below
   n = 2^4096 — the dense case is rescued by the loglog n arm only
   past that threshold. (Constant-free reading δ = min{m/n, loglog n}:
   dense graphs become valid at n ≥ 256, but every m < 3n graph stays
   invalid at all n.)

2. **Honest sparse complexity at forced δ = 3.** The clamped reduction
   yields n₂ = Θ(n + m) vertices, O(m) edges, degree ≤ 3, and the §3
   wrap-up formula reads O(m(t + log n · log t/(3t))), minimized at
   t = Θ(√(log n · loglog n)):

     T_clamp = O(m · √(log n · loglog n)).

   For m = Θ(n) this equals the paper's claimed sparse bound
   O(√(mn · log n · loglog n)) up to a constant.

3. **The clamp is free in every regime (factor ≤ 2).** Write
   L = log n · loglog n, so T_clamp = O(m√(L/3)).
   - Sparse arm m ≤ 12n, claim O(√(mnL)) = O(m√(nL/m)): ratio
     √(m/3n) ≤ 2, and ≤ 1 whenever m ≤ 3n.
   - Dense arm m > 12n at n < 2^4096 (there loglog n < 12, so the
     prescribed δ = ¼·loglog n < 3 and the clamp is active), claim
     O(m√(log n)): ratio √(loglog n / 3) < 2 exactly because
     loglog n < 12 in this regime.
   - n ≥ 2^4096 with m ≥ 12n: prescription valid, clamp inactive.
   So δ := max(3, ⌊¼·min{m/n, loglog n}⌋) — the reference behavior
   pinned above — achieves every bound the paper claims, up to a
   factor of 2, at every size, and coincides with the prescription
   asymptotically.

4. **The δ ≤ log k violation also costs O(1).** With constant δ = 3
   every use of the assumption in the analysis (Lemma 3.2's simplified
   time term, Lemma 3.9's linear-scan step T3 = C′δ|U| ≤ C′|U| log t)
   is δ·(unit work) with δ = 3 — constant-factor slack even at the
   sizes where log k < 3 (all n with k < 8, i.e. every practical
   size). Footnote 1's waiver ("the algorithm also works when
   δ > log k") is thereby quantified for the clamped regime: constants
   only.

Revised consequence: the stated optimal parameterization is indeed
unreachable at all storable sizes, but the claimed bounds are
achievable — the letter item is now "the prescription needs a clamp
the paper never states, plus the two-line proof that the clamp is
free." Caveat: "claims survive" is at the parameterization level; the
Q11/Q14 re-settlement accounting gap is orthogonal and affects all
parameterizations equally.

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

Upgrade (2026-07-15, Q14 addendum 2): the pinned event-count guard is
not merely a deviation but a safety property — a completed run
certifies T ≤ t³·2^(l_top·t) − n₂ on the total duplicate excess (every
settlement event bubbles into U_top, so the top guard reads all of
them), while the paper's set-count guard imposes NO ceiling on
settlement events anywhere: a re-settlement cascade, if constructible,
finishes silently with super-linear time under set semantics and
aborts loudly under the pinned one.

## Q14 — the missing re-settlement lemma: statement, why naive induction fails, and the one-bit repair. OPEN — NARROWED 2026-07-15 (the D-entry channel is closed by the gate-tiling lemma; only the FindPivots/W′ channel remains)

What the paper needs and never states or proves:

> **Lemma (missing).** The total number of settlement events across all
> calls of a top-level run — Σ_X |U_X| counted with the multiplicity of
> repeated vertices across calls — exceeds the number of distinct
> settled vertices by at most [some bound].

Lemma 3.8's disjoint-union items silently assume the excess is zero;
Q11 measured and traced it nonzero (and the V8 control row shows it is
not even a tie artifact: 664 duplicate events at n = 5e5 float-random).
Everything in Lemma 3.9's time accounting that sums |U_X| over layers
rests on this missing bound.

What is established about the excess (2026-07-13, V7/V8 experiments +
trace):

1. **Channels, exhaustively** (trace-verified): a duplicate settlement
   of v consumes either a stale D entry of v (created before v's
   settlement, pulled after) or a W′ membership in a call whose
   call-local u_set cannot see v's settlement. Post-settlement D
   entries of v arise only from (i) same-edge exact-tie re-offers from
   pred(v)'s own settlement events passing the ≥ B_i / ≥ B′ gates,
   (ii) pivot re-inserts while a re-frontiered v descends (≤ l_top per
   descent), (iii) line-33 premature re-inserts.
2. **Why naive induction fails**: channel (i) charges duplicates of v
   to settlement events of pred(v), giving the recursion
   dup(v) ≤ l_top · (1 + dup(pred(v))) + …, which unrolls
   exponentially along pred chains. The observed excess is tiny
   (0.0003–0.037 events/vertex) because the ≥ B_i / ≥ B′ gates refuse
   most re-offers — any real proof must exploit the gates, not the
   chain structure. We do not have that proof.
3. **The `≤` cannot be removed** (V7, REJECTED with a reachability
   counterexample): rejecting exact ties orphans never-settled
   vertices. So the excess cannot be eliminated by tightening Relax.
4. **One bit eliminates it exactly** (V8, Tier-2 green, measured
   faster): a per-vertex settled marking removes every duplicate while
   preserving distances across the full adversarial matrix, making the
   missing lemma trivially true (excess = 0) for the marked variant.
   V8's own safety assumption — every settlement settles a complete
   vertex — is exactly Lemma 3.7's completeness clause under the Q9
   closure, and the green matrix is empirical support for it.

Status: the bound for the algorithm AS WRITTEN remains open — this is
the strongest candidate for an original mathematical contribution, and
the letter should present it as: here is the gap (Q11), here is why it
is not locally repairable by bound arithmetic (Q12) or by touching
Relax (V7), here is the measured magnitude, and here is the one-bit
repair with proofs-modulo-one-clause (V8). See the 2026-07-15 addendum
below: the gate route asked for in item 2 was executed and closes the
D-entry channel with an additive per-vertex bound; what survives of
the open problem is exactly the FindPivots/W′ channel.

## Q14 addendum (2026-07-15) — the gate argument executed: gate tiling closes the D-entry channel; FindPivots is the one ungated relaxation site

Item 2 above said any real proof must exploit the ≥ B_i / ≥ B′ gates.
Executed. Everything below is for the algorithm AS WRITTEN (paper
pseudocode, event-count or set semantics alike) under one standing
hypothesis:

> **(H)** every settlement event settles a complete vertex — Lemma
> 3.7's completeness clause on reachable states (the Q9 closure);
> empirically supported by V8's green matrix (its contrapositive).

**Lemma A (gate tiling: one insert per settlement event per edge).**
Fix a settlement event ε of vertex u, occurring in call X_q whose call
stack is X_0 ⊃ X_1 ⊃ … ⊃ X_q. The edge sweeps ε triggers offer each
out-edge (u, v) exactly once per stack level, and the
accept-and-insert windows are pairwise disjoint intervals:

- at X_q itself: [0, B_{X_q}) if ε is a base-case pull (Algorithm 4's
  relaxations insert into the base D with no lower gate), or
  [B′_{X_q}, B_{X_q}) if ε is a W′ append (line 36's ≥ B′ gate);
- at each ancestor X_j (j < q): [B_{X_{j+1}}, B_{X_j}) — ε's batch
  rides the returned U_i through the line-30 union up the stack; each
  ancestor sweeps the returned batch exactly once (lines 21–26), the
  line-23 gate is ≥ B_i = B_{X_{j+1}} (the bound the child was called
  with), and Relax's own bound is < B_{X_j}. u cannot also be swept
  via an ancestor's W′ path: line 30 put u into that call's U before
  line 34's W \ U filter runs.

Pull bounds nest down the stack (the separator of Lemma 3.4 is ≤ the
caller's B), so these intervals tile [0, B_{X_0}) disjointly. The
candidate d[u] + w_uv is one fixed 4-tuple across the whole unwind (u
is complete under (H), so its label never moves again), hence lands in
at most ONE window: **each settlement event of u causes at most one
D-insert per out-edge over the entire run.** ∎

This is §3.4 property 6 ("any edge can be validly relaxed at most
once") repaired to per-event form — the form that survives duplicates.

**Lemma B (tie targeting).** A valid relaxation into an
already-complete v is an exact 4-tuple tie whose source is v's
canonical predecessor: cand = ⟨d[u].len + w, d[u].hops + 1, v, u⟩ =
d[v] pins u = d[v].pred and pins d[u]'s length and hops to the
canonical prefix values (cand < d[v] is impossible — labels are walk
lengths and d[v] = dis-label(v) is their minimum). All three
insert-relax sites (lines 23, 36, Algorithm 4's pull loop) run only
inside a settlement event's sweep, so post-settlement gated inserts of
v are triggered only by settlement events of pred(v), along the one
edge (pred(v), v). ∎

**Chain accounting.** Call a "chain" the lifetime of one re-frontier
seed of a settled v: a live D entry (pulls consume it; a child
returning v unsettled re-creates it one level up via the merge, since
D_i ⊇ S_i \ U_i) or a surviving S/P_j membership in a stack call
(which can spawn at most O(1) entries/descents via lines 7/13/29/33
before converting into an entry or dying — the keyed dedup and the
ascending pulls serialize them). Each chain ends in at most one
duplicate settlement event: a base-case pull of v or v's own W′
append, both of which put v into U so lines 18/31/34 stop feeding it.
Counting chain seeds:

- live at v's first settlement: at most one entry per stack D plus
  O(1) memberships per stack call — O(l_top) seeds;
- created after v's first settlement: only through Lemma B's channel,
  at most ONE per settlement event of pred(v) by Lemma A — not one
  per stack level.

The second bullet is where the naive induction's exponential unroll
dies: dup_D(v) ≤ O(l_top) + (1 + dup(pred(v))) is additive along the
canonical predecessor chain, not multiplicative.

**Lemma D (the W′ channel walks tied segments).** The one settlement
channel needing no D entry is a W′ append of a v swept into a failed
FindPivots arborescence of a call with v ∉ S. Every relax acceptance
into a complete vertex pins its predecessor and its length/hops to the
canonical prefix values (Lemma B, noting cand reads only len/hops of
the source label), so the arborescence path from the failed root
q ∈ Q ⊆ S to v is a minimal-(length, nEdges) path: on inputs without
exact length-and-hops ties between distinct walks (float-random,
almost surely) it is exactly the suffix of v's canonical path and q
lies on v's canonical shortest path; on tie-rich inputs it is a
tied-sibling path. Either way: a foreign W′ duplicate of v requires a
tied-path ancestor of v within < k hops to sit in that call's S and
fail its search, and one failed search (< k vertices) emits at most
k − 1 duplicates total. ∎ (The Q11 trace is the non-foreign case:
v5 ∈ S re-frontiered by its own stale entry, own search failed, own
W′ append.)

**Theorem (conditional additive bound).** Under (H), with dup_W(v)
denoting v's W′-channel duplicates from Lemma D,

  dup(v) ≤ Σ_{w on the canonical path s→v} (O(l_top) + dup_W(w)).

In particular on any run where the W′ channel is silent (dup_W ≡ 0),
dup(v) = O(l_top · (hops(v) + 1)) — per-vertex, polynomial, additive.
The exponential blow-up of the naive recursion was an artifact of
ignoring the gates. ∎

**What Lemma A buys the time analysis.** Total gated inserts are now
≤ Σ_u (1 + dup(u)) · outdeg(u) ≤ δ(n₂ + T), T = Σ_v dup(v) the total
excess. Every charging step of Lemma 3.9 that duplicates falsified
degrades additively in T — including the Σ|Q_Y| bullet, whose "at most
δ relaxations from complete predecessors" step was circular under
duplicates (relaxation events from complete predecessors are
themselves multiplied by re-settlements) and is un-circularized by
Lemma A: ≤ δ per settlement EVENT. So the missing lemma reduces to
bounding T alone, and under V8 (T = 0 by construction) Lemma 3.8's
disjoint unions and Lemma 3.9's accounting hold verbatim modulo (H) —
V8 is not just operational closure, it makes the paper's analysis
sound as stated for the marked variant.

**Still open (sharpened).** Bounding dup_W for the algorithm as
written. FindPivots is the single relaxation site with no lower gate
(only cand < B), so Lemma A does not tile it; Lemma D reduces the
question to counting failed-search memberships of re-frontiered tied
ancestors within k − 1 hops, but those S-memberships feed back through
chain bounces (a partial child returning v unsettled migrates the
entry up to be re-pulled), and nothing here bounds bounces. Worst-case
aggregate from the D channel alone is O(l_top · Σ_v hops(v)) —
polynomial but far above the measured 0.0003–0.037 events/vertex; that
gap is the remaining open problem.

**Lemma D measured (same day; raw record
`benchmarks/results/dup_classify.txt`, driver `examples/dup_classify.rs`,
`dup-classify` build feature).** Every duplicate settlement event of
the paper engine classified by channel on the standard adversarial
family (totals match the recorded dup counts exactly; classification
partitions them, asserted per run):

| input | n | dups | base-seed | base-ins | own-root | own-swept | foreign | f-anc-Q |
|---|---|---|---|---|---|---|---|---|
| zero-heavy minslack | 4 094 | 5 | 0 | 0 | 5 | 0 | 0 | — |
| zero-heavy minslack | 16 382 | 608 | 49 | 10 | 549 | 0 | 0 | — |
| tie-rich 0..3 | 170 000 | 5 074 | 734 | 741 | 3 226 | 0 | 373 | 373 |
| zero-heavy large | 500 000 | 149 | 5 | 15 | 110 | 0 | 19 | 19 |
| float-random | 500 000 | 664 | 66 | 299 | 250 | 0 | 49 | 49 |

Three readings:

1. **Lemma D's prediction holds with zero exceptions**: every
   foreign-W′ duplicate (441 across all inputs) has a pred-chain
   ancestor within k − 1 hops that is a failed root (∈ Q) of the
   appending call — including on the tie-rich inputs, where the
   tied-sibling caveat was available but never needed.
2. **The dominant channel is the Q11 trace's own pattern** (own-root:
   a re-frontiered settled vertex whose own search fails), and
   own-swept = 0 everywhere: a settled vertex in S is never re-settled
   through another root's arborescence. The channel inventory
   simplifies in practice to: chains ending at level 0 (base-seed),
   in-base tie re-stores (base-ins — Lemma A/B's bottom-interval
   insert observed directly), own failed roots, and Lemma D foreigners.
3. **On continuous weights the in-base tie re-store carries almost
   half the excess** (299 of 664) — direct confirmation that the
   same-edge re-offer needs no cross-walk ties, only an unchanged
   source label (Q11's corrected reading).

## Q14 addendum 2 (2026-07-15) — the W′ cascade: bounces are polynomial, the reduction is sharp, and the paper's set semantics has no event ceiling at all

Continuing the gate program into the one remaining channel. Everything
below is again for the algorithm as written, under hypothesis (H), with
T = total duplicate settlement events and n₂ the working vertex count.

**Lemma E (bounces are rare, unconditionally).** The total number of
line-33 re-insertions over an entire run — the "bounce" that lets one
chain migrate up and re-descend repeatedly, the last unbounded quantity
in the chain accounting — is less than (n₂ + T)·(l_top + 1)/t.

*Proof.* Lines 31–33 emit at most |S_Y| inserts, and only when Y is a
partial execution: a full execution has B′ = B and the guard interval
[B′, B) is empty. A partial execution at layer l has
|U_Y| > t³·2^(lt) settlement events, and each settlement event appears
in |U_Y| of at most one call per layer (its creator or that creator's
unique ancestor at the layer), so Σ_{Y at layer l} |U_Y| ≤ n₂ + T and
there are fewer than (n₂ + T)/(t³·2^(lt)) partials at layer l. With
|S_Y| ≤ t²·2^(lt) (§3.4 property 2), the layer-l bounce volume is
below (n₂ + T)/t; sum over the l_top + 1 layers. ∎ (Unsettled base-case
seeds recycled through leftover D obey the same count at level 0.)

With l_top = ⌈log n₂/t⌉ this is O((n₂ + T)·log n/t²) — a lower-order
term. Bounces are NOT the missing multiplier.

**Flow corollary.** Per settlement event, Lemma A caps relax-inserts
at outdeg ≤ δ; line-7 pivot inserts are ≤ |U_X|/k per call (Remark
3.3); line 29 inserts once per settled pivot; Lemma E caps bounces;
line-13 expansions follow the paper's own T2 charging at O(|U_X|) per
call. So total D-inserts, S-memberships, and FindPivots searches are
all bounded by poly(δ, t, l_top)·(n₂ + T): the run's entire
bookkeeping volume is self-consistently polynomial in its own
settlement-event count. T itself is the only quantity nothing bounds.

**Why the gates stop here (the branching obstruction).** Lemma D plus
the flow corollary give the per-vertex recursion

  events(v) ≤ O(l_top) + events(pred(v))              [D channel]
             + Σ_{q ∈ tied-anc≤k−1(v)} fails(q)        [W′ channel]

with fails(q) ≤ SM(q) ≤ l_top·chains(q) + bounce share, and
chains(q) ≤ O(l_top) + events(pred(q)). The W′ sum ranges over up to
k − 1 ancestors, so the recursion BRANCHES with reproduction factor
Θ(k·l_top) per k-hop window, and unrolling yields only bounds of the
shape events(v) ≤ 2^(O((h(v)/k)·log(k·l_top))) — exponential in path
depth. This is structural, not slack: FindPivots is the one ungated
relaxation site, nothing stops a re-frontiered ancestor's failed
search from re-walking a settled tied segment of up to k − 1 hops,
each walk can re-settle up to k − 2 vertices, and each re-settlement
seeds chains further downstream. A sub-exponential bound needs a
resource the cascade consumes that the gates do not see; D-inserts,
S-memberships, and bounces are all ruled out (each scales WITH the
cascade at coefficient > 1).

**The missing lemma, final form (a dichotomy).** Either (a) some
family sustains cascade reproduction above 1 and Σ_X |U_X| grows
super-linearly in n — then Lemma 3.9's charging, which sums |U_X|
over calls, is false as stated and the paper's time bound fails on
adversarial inputs; or (b) every input damps the cascade — then the
proof needs a resource beyond the gates, and nothing in the present
accounting supplies one. The construction shape for (a) is explicit:
a long exact-tie path (zero- or equal-weight edges) whose k-hop
windows are re-frontiered by periodic side triggers so every failed
root re-sweeps its settled window; what a construction must deliver
is the B′/partial-execution scheduling cooperating at every scale.
The measurements are far from the cascade regime (≤ 0.037
events/vertex; foreign-W′ ≤ 7% of duplicates) — evidence about the
tried families, not a bound.

**A ceiling the paper doesn't have (upgrades Q13).** Under the pinned
event-count semantics the top call's own guard reads every settlement
event (all of them bubble into U_top), so a completed run certifies
T ≤ t³·2^(l_top·t) − n₂ unconditionally — an enforced polynomial
ceiling on the excess, with the totality assert converting "ceiling
exceeded" into a loud abort. Under the paper's set-count guard no
control-flow ceiling on settlement EVENTS exists anywhere in the
algorithm: a cascade, if constructible, completes with correct
distances and super-linear time, silently. Q13's deviation is thereby
upgraded from pinned convenience to a safety property. (V8 removes
the cascade by construction under either semantics.)

## Q14 addendum 3 (2026-07-15) — the cascade construction attempted: it ignites at Θ(n) with a 100–10⁴× constant, and every super-linear knob plateaus

The dichotomy of addendum 2 put on the bench. Construction (driver
`examples/cascade.rs`, wave diagnostics `examples/cascade_trace.rs`,
raw record `benchmarks/results/cascade.txt`): a unit-weight path
p_1 → … → p_L (dis(p_i) = i), one decoy trigger per path vertex — an
early relaxation s → d_i → p_i planting a stale D entry of p_i at
value x_i above the entire path range, accepted while d[p_i] = ∞ (the
`≤` blocks all post-settlement planting, so pre-settlement decoys are
the only way to seed stale entries at foreign values) — plus G = 8
ballast leaves per trigger gap. Every run distance-checked bit-exact
against the crate's Dijkstra, up to n = 2×10⁶; the Q10/Q13 ceiling
never tripped.

**The cascade ignites.** Wave structure confirmed by trace (path
settlements in log order form ascending runs to the path end): a
pulled stale entry re-settles p_i in a base case, the exact-tie
re-offer of the canonical edge re-inserts p_{i+1} below the local
bound, and the wave rolls the full remaining path, stalling and
resuming across base-case chunks and |U| cutoffs. Excess measured at
0.56–3.8 events/vertex depending on parameters — two to four orders
above every natural family tried (0.0003–0.037) — with ~80–95% of
duplicates in the base-inserted class (the in-base tie re-store,
Lemma A's bottom interval, exactly the wave mechanism).

**Every super-linear knob plateaus.** Three independent scaling
probes, all flat:

1. L at fixed parameters: dup/n ≈ 0.59 constant, L = 50 → 800, at
   forced (3,2,2); ≈ 1.2–2.9 at natural parameters (the variation is
   the l_top sawtooth, not growth).
2. Forced t = 4..7 at fixed l_top = 3, sized at the sawtooth peak
   n = 2^(l_top·t) − 1 (testing #waves ~ n/M_top = Θ(2^t/t)): dup/n
   = 2.96, 3.79, 3.58, 3.22 — flat, hypothesis refuted.
3. l_top = 3, 4, 5 at fixed t = 4, and l_top = 6, 8, 10 at fixed
   t = 2 (testing #waves ~ l_top): dup/n = 2.961, 2.918, 2.919 and
   0.5566, 0.5583, 0.5585 — constant to three decimals over 250× in
   n, hypothesis refuted.

Waves per vertex converge to a size-independent, parameter-dependent
constant (~5.6 at t = 2, ~30–38 at t = 4..7, non-monotone in t).

**The damping force, identified empirically.** Addendum 2's
obstruction analysis asked for "a resource the cascade consumes that
the gates do not see"; the experiment exhibits it: CONCURRENT
WAVEFRONT MERGING. All pre-settlement plants of one vertex collapse
to one entry per D through the keyed dedup (coexisting entries of the
same key keep-min); at every descent the pull re-sorts by current
labels, so simultaneous triggers cluster into the same small-value
batches and their waves fuse; parked fronts absorb later waves that
catch up (a wave reaching an already-inserted front value dedups into
it — visible in the trace as runs ending exactly where an earlier
run's front parked). The number of separated re-frontier episodes a
vertex can experience converges to a constant the run size cannot
inflate. This is the concrete proof target for branch (b) of the
dichotomy: bound concurrent live chains per vertex by the dedup
geometry — none of D-inserts, S-memberships, or bounces (the
resources addendum 2 ruled out), but entry COEXISTENCE.

**Where the dichotomy stands.** No super-linear ignition through this
family; three scaling laws refuted; evidence now leans branch (b) —
but a bound is still not proved, and the constructed Θ(n) excess with
constant ≈ 3.8 already falsifies Lemma 3.8's disjoint-union
accounting by a material constant factor on inputs where the paper's
analysis presumes zero. The letter's framing sharpens once more: the
missing lemma is real (Q11), unrepairable locally (Q12/V7), reducible
to bounding wavefront coexistence (addenda 1–3), removable by one bit
(V8), and empirically Θ(n)-bounded with a two-orders constant under
deliberate attack (this addendum).

## Q7 — nEdges overflow bound. NOTE

nEdges strictly increases along relaxations and is bounded by the longest
path in the reduced graph (< n₂). i32 suffices below n₂ = 2³¹ (same
assert as the first project's Label packing); the zero-weight cycles make
nEdges of practical labels larger than hop counts of the original graph
(up to +|C_v| detours), still < n₂.
