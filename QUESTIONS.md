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

Superseded in part (2026-07-16, Q15): item 2's cleanliness was a
property of those inputs. The wave-escape family violates the base
clause thousands of times and the recursive-return clause hundreds of
times on plain positive unit weights, and V5 zeroes all of it — see
Q15 for the mechanism and the recorded run.

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

## Q14 — the missing re-settlement lemma: statement, why naive induction fails, and the one-bit repair. OPEN — NARROWED 2026-07-15 (the D-entry channel is closed by the gate-tiling lemma; only the FindPivots/W′ channel remains); (H) proof opened 2026-07-16 (addendum 7: restricted invariant measured clean everywhere, base-case half proved, two sub-lemmas remain); (H) FALSIFIED LIVE 2026-07-17 for the paper engine and V8 (addendum 13, row G; cold-audited addendum 14) — the proof target is now (H | V5); V8 wrong FINAL distance exhibited in full recursion same day (addendum 15, row H) — V8's exactness falsified, VARIANTS.md amended; (H | V5) ASSEMBLED AT SKETCH GRADE 2026-07-18 (addendum 16: the window/off-path-anchor recast — window proved modulo Q12, anchor closed over the channel inventory, E-c drops out of the base half; same-session math, cold audit owed); cold audit PASSED 2026-07-18 (addendum 17: all five targets hold, two wording repairs, Q12 carve-out transitive, custody persistence named as the write-up's spine — Q12 is the last statement-level gap)

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

## Q14 addendum 3 (2026-07-15) — the cascade construction attempted: it ignites at Θ(n), one to four orders above natural inputs, and every super-linear knob plateaus

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
0.56–3.8 events/vertex depending on parameters — one to four orders
above every natural family tried (0.0003–0.037; the closest same-regime
comparison is 0.56 vs 0.037, a factor of 15) — with ~80–95% of
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

## Q14 addendum 4 (2026-07-15) — the B′-regression hypothesis refuted by measurement; per-call monotonicity survives, the once-per-call lemma follows, and containment is the new structural picture

Attempting to prove a blocking theorem ("after the first settlement
event of v inside call X's subtree, B′_X has passed dis-label(v) and
every gate refuses further dis-valued entries") surfaced a candidate
counter-mechanism: the base-case interior is ungated below, so a
re-settlement wave could return B′ = its front's small value and drag
the parent's progress measure BACKWARDS — B′_i < B′_{i−1} — vacating
every "≥ B′_{i−1}" step in the Lemma 3.7/3.8 proofs. A cheap counter
(`bprime_regressions` in the dup-classify build; re-recorded runs in
`benchmarks/results/{dup_classify,cascade}.txt`) tested it:

**Zero regressions on every run.** The full adversarial family
(thousands of duplicates) and every cascade configuration (up to
n = 2×10⁶, dup/n up to 3.8): B′ never decreased across iterations of
any call; the V8 control is likewise zero. The hypothesis is refuted,
and the reason is instructive twice over:

1. **Self-calibration at call creation.** A call's B′_0 is the minimum
   LABEL among its pivots, so a re-frontiered settled vertex (stored
   value high, label low) lowers the fresh call's starting point
   instead of regressing an old call mid-loop. The stale pull's label
   drop is absorbed where the paper's invariants don't look.
2. **Containment.** A wave escaping its containing child (partial exit
   with the front in leftover D, merged into an ancestor whose B′ has
   already progressed past the front's value) would force a regression
   or a bound-clause violation at that ancestor — and neither counter
   has ever fired. The structural reason: a call whose value-window is
   wide enough to span both the settled range and the trigger range is
   necessarily a HIGH-level call, and its |U|-budget t³·2^(lt) grows
   2^t-fold per level while the window widens — in every configuration
   tested the wave burns out inside the child (needed: wave length
   ≤ t³·2^((l−1)t); violated only around t ≥ 17 at our densities,
   far beyond practical parameters).

**Lemma F (once per call, settled-value channel; conditional).**
Assume per-call B′ monotonicity (now empirically universal, still
unproved under duplicates) and the bound clause. Then each call X
sources at most ONE settlement event of v through entries stored at
value dis-label(v): the first such event in X's subtree forces
B′_X > dis-label(v) by max dis(U_i) < B′_i (which holds structurally
even under duplicates), after which line-24 inserts are gated above
it, merged child structures carry values ≥ B′_child ≥ B′_i, line 33
is gated ≥ B′, and v's P_j membership was removed at line 18. Every
further event of v sourced by X consumes a pre-settlement entry of v
(stored value > dis-label(v), bounded by one per D at a time via the
keyed dedup) or a W-membership. ∎ (This is Lemma 3.8's per-call
disjointness salvaged in the only form that survives Q11: the Q11
trace's double settlement consumed exactly one stale pre-settlement
entry at stored value (1, h7) > dis-label (0, h7) — consistent.)

Consequences: all events of v beyond once-per-call ride pre-settlement
entries and W-memberships, both call-consuming resources; combined
with wavefront merging (addendum 3) and Lemma A, the count of
separated episodes per vertex is pinned to the number of DISTINCT
CALLS whose window straddles dis-label(v) and that actually receive a
pred(v)-batch — fresh-call creation via straddling pull batches is
the true limiting resource, replacing "entry coexistence" as the
branch-(b) proof target. What remains unproved, in order: (i) per-call
B′ monotonicity under duplicates (attack at the wave-escape regime,
wave length > t³·2^((l−1)t)); (ii) a bound on straddling-call creation.
Both are now concrete, instrumented, and empirically constrained.

Follow-up (2026-07-16, Q15): item (i) attacked with the escape family —
B′ monotonicity SURVIVED (zero regressions even under forced escapes);
the escape leaks through the bound clause instead, which fails at
scale for the algorithm as written. So Lemma F's monotonicity
hypothesis strengthens empirically while its bound-clause hypothesis
narrows Lemma F's scope to the V5-repaired algorithm; item (ii) gains
the batch-coarsening feedback as its concrete mechanism. Details and
recorded runs in Q15.

## Q14 addendum 5 (2026-07-16) — the coarsening law quantified: a conditional Θ(n) theorem, its merge-onset prediction confirmed, and the honest loopholes

The batch-coarsening feedback (Q15 item 5) turned into arithmetic and
tested. Throughout K := t²·2^t (the live pull capacity is
M ≥ C/K at cumulative event count C, because the climbing leftover D
resides at the level l with t³·2^((l−1)t) < C ≤ t³·2^(lt) and
M = t·2^((l−1)t)).

**Conditional theorem (coarsening ⟹ linear excess).** Assume every
pair of separately-firing wave triggers over the same path must be
separated in pull rank by Ω(M) = Ω(C/K) other entries at firing
progress C. Then R sequential re-waves of a length-L path force
ballast Σ_r ≥ Σ_r (n₀ + rL)/K ≥ R²L/2K ≤ n, so the total excess
T = R·L satisfies

  T² = (R²L)·L ≤ 2K·n·L,  i.e.  T ≤ √(2K·n·L) ≤ t·2^((t+1)/2)·n.

The per-vertex ceiling is dup/n ≤ √(2K·L/n) ≤ t·2^((t+1)/2) ≈ 5.7 at
t = 2 and ≈ 22.6 at t = 4. This matches the addendum-3 plateau from
the other side: the strongest measured excess (3.8) sits below the
t-appropriate ceiling everywhere. Excess beyond Θ(n) requires
re-waving the SAME vertices (disjoint fresh paths pay for themselves
in n), so the theorem, if its hypothesis is closed, is branch (b) with
T = O(t·2^(t/2)·n) — n^(1+o(1)) at prescription parameters, which
still exceeds the paper's time budget asymptotically but pins the
excess to near-linear.

**The staggered experiment** (driver `examples/escape.rs` part 2, raw
record `benchmarks/results/escape.txt`): R rounds of (phase block of
E leaf values, trigger planted at p_r), waves firing sequentially so
batch-merging can only come from coarsening. The law's merge-onset
prediction r* ≈ K·E/(E+L) — beyond r* the block no longer outgrows
the live M — is confirmed: at t = 2, E = L = 20 000 (r* = 8) the
effective wave counts are ~6 of 8 and ~10 of 16 (119 984 and 199 938
duplicates ≈ 6L and 10L); growing E to 60 000 (r* = 12) recovers ~7
full waves at R = 8. Measured maxima stay far below the ceilings:
dup/n ≤ 0.75 at t = 2, ≤ 1.58 at t = 4 (R = 64, E = 5 000; 535 996
duplicates at n = 340 075) — the strongest staggered attack loses to
the cascade's 2.9 and everything sits under t·2^((t+1)/2).

**Loopholes that keep the theorem conditional**, named honestly:
(i) sibling-D separation — triggers in different sibling structures
are separated by the first wave's own events rather than rank ballast
(the optimization then still binds through consumed-events C, but the
accounting differs); (ii) late plants — a plant for a far path vertex
can be made just before that vertex's first settlement, entering a
different level structure than the early-plant analysis assumes;
(iii) the plant-erasure effect — plants sharing a base chunk with
their target's label insert are erased by the keyed dedup (observed
directly: the t = 4, R = 16 staggered row has ZERO duplicates because
26 early vertices fit inside one t³ = 64 chunk and every plant was
deduped away — the same mechanism that Q6 documented as load-bearing
protection, here acting as an accidental trigger suppressor).

## Q14 addendum 6 (2026-07-16) — the three loopholes closed: firing altitude, gap capacity, and the reuse attack's collapse; the linear-excess theorem reaches sketch grade with no named holes

Addendum 5 left the coarsening theorem conditional on three loopholes.
All three are now closed — two by argument, the third by argument plus
a designed adversarial experiment that failed exactly as the argument
predicts (and failed harder, exposing one more suppression mechanism).
Throughout: (H), the pinned Merge semantics (children's leftover Ds
consolidate into the parent's — the paper's own Lemma 3.4 Merge), and
K = t²·2^t.

**Lemma G (firing altitude).** Any entry pulled at global settlement
count C is pulled by a call on the active stack whose subtree contains
every event since that call's creation; since the call has not
exhausted, its budget t³·2^(lt) exceeds those events, so a call
created when only n₀ events existed pulls with capacity
M = t·2^((l−1)t) ≥ (C − n₀)/K. Custody chain: two stale entries that
fire separately were batch-separated at the FIRST (highest) call
whose D held both — a call at least as old as the younger plant — so
every separation of early-planted triggers is priced at coarsening
altitude. ∎ (sketch)

This closes loophole (ii), late plants, by a squeeze: a full wave
needs its trigger value above the whole settled range (x > dis(p_L)),
yet the plant must precede its target's first labeling (relax-≤
refuses inserts above the current label), so plants are early, ride
the consolidated climb, and fire at altitude.

**Lemma H (gap capacity).** Rank separation of consecutive triggers
must be paid by entries whose STORED values lie in the value gap
between them. Wave re-inserts cannot pay: they are stored at settled
labels, far below trigger range — this closes loophole (i)'s
self-payment variant, and its cross-call variant is priced at the
common ancestor by Lemma G's custody chain. What can pay: (a) gated
relax inserts, whose stored value equals the accepting candidate
dis(u) + w_uv from a settlement event — at most ONE value per in-edge
under (H); (b) pivot and line-33 inserts, which store current labels
at progress scale and cannot reach high gaps; (c) pre-settlement
plants. Plants are capped hard: per vertex at most one live entry per
stack tile (the gate tiling of Lemma A puts coexisting entries of one
key in distinct stack Ds, ≤ l_top + 1), consolidation merges of the
climb keep-min them together, and after first settlement no re-plant
above dis is possible (relax-≤). So gap capacity over a whole run is
≤ (l_top + 1)·n, and edge-paid reuse (one vertex filling many gaps
through many in-edges) is structurally impossible. ∎ (sketch)

**The reuse attack (loophole (iii) inverted), run and collapsed.**
Driver `examples/escape.rs` part 3, record `benchmarks/results/
escape.txt`: shared ballast vertices with q in-edges from chunk-spaced
early sources offering values in q different trigger gaps in
descending order — the design that would gain a δ-factor if entries
survived independently. Measured at q ∈ {8, 4, 1} (R = 8 triggers,
L = E = 20 000, n ≈ 40k): the run collapses to EXACTLY 1.00 effective
waves (20 000 duplicates) in every configuration, for both the paper
engine and V5. Post-mortem identified a suppressor beyond Lemma H's
dedup-at-birth: **wave-crossing plant erasure** — a wave that crosses
vertex v re-inserts v at its settled label through line-33/pivot
sites, and wherever such an insert meets the D holding v's pending
higher plant, keep-min erases the plant. The first wave sterilizes
later triggers over the same path. Every suppression mechanism found
in three days of adversarial construction — batch merging, coarsening,
plant erasure at birth chunks, wave-crossing erasure — operates in
favor of branch (b).

**Theorem (linear excess; sketch grade, no named holes).** Under (H),
with the pinned Merge semantics, for the V5-repaired engine (and the
paper engine, whose Q3 leak only damps further):

  T ≤ √(2K·(l_top + 1)·n·L_max) + O((l_top + k)·n)
    ≤ t·2^((t+1)/2)·√(l_top + 1)·n + lower order,

where L_max ≤ n is the longest settled tie-path. The proof skeleton is
Lemmas A, B, D, E, F, G, H plus the chain accounting; what remains to
make it a paper-grade proof is the custody-chain formalization and the
wave-exhaustiveness bookkeeping (every duplicate rides a wave started
by a plant or an initial-seed stale entry). Empirical margins: the
strongest attack achieved dup/n = 1.58 (t = 4) against a ceiling of
≈ 22.6·√l_top; every other family sits 1–4 orders below. The missing
re-settlement lemma of Q14, for the honest engine, is answered at
sketch grade: **the excess is linear in n with an n^o(1) parameter
constant — Lemma 3.8's disjointness fails, but only by a bounded
per-vertex factor, and the paper's totality is safe while its time
analysis degrades by at most the same factor.**

## Q14 addendum 7 (2026-07-16) — the (H) proof opened: the incomplete-restricted invariant passes its pre-test with zero violations everywhere, the freshness algebra, base-case completeness unconditional, and the two sub-lemmas that remain

The route to (H) — every settlement event settles a complete vertex —
is to re-run Lemma 3.7's induction with its falsified clause 2
restricted to incomplete vertices. Before attempting the proof, the
candidate invariant was instrumented and measured.

**The pre-test (driver instrumentation in `src/solver.rs`, features
`verify-contract` + `dup-classify`; raw record
`benchmarks/results/restricted_invariant.txt`).** Counters for the
restricted clause — "min stored value among keys of INCOMPLETE
vertices ≥ B′" — at base-case returns, recursive returns, and the end
of every BMSSP iteration, under two completeness proxies: *unsettled*
(vertex not yet first-settled; a superset of incomplete, so zero here
implies the true clause) and *incomplete* (exact: current label length
≠ true distance, oracle-checked). Results over the full adversarial
matrix (escape/staggered/reuse, the dup_classify five, the
verify_contract families; paper, V5, and V8 engines):

1. **The exact restricted clause holds with ZERO violations
   everywhere** — including the escape rows where the unrestricted
   clause fails 988/2 973 times per run. Every falsifying entry ever
   measured belongs to a complete vertex. Strong go signal.
2. **The unsettled proxy FAILS on escape geometry** (paper at
   L = 20 000: 2 223/742/2 965 base/rec/iter; V8 within a few counts
   of paper; V5 zero — its cap lowers B′ below the offending entry).
   The counts reproduce Q15's chunk arithmetic ⌈wave/(t³+1)⌉: the
   per-chunk Q3 undercut is the NEXT path vertex freshly relaxed at
   its exact distance — complete but never settled. The Q3 star pins
   the semantics in miniature (vertex 10 at len 8 < bound 9:
   unsettled fires, exact does not). Consequence: **the invariant must
   restrict by completeness, not settledness** — no algorithm-visible
   event marks the boundary, which is fine for a proof (completeness
   is the same semantic property the paper's own clauses quantify
   over) but rules out any "settled-bit" shortcut in the induction.

**The freshness algebra (proved, small).** Call an entry of key v
*fresh* if its stored value equals the current label d[v].

- *Stored dominates label*: every stored value of v in every live D is
  ≥ d[v] — inserts store then-current labels, labels never increase,
  keep-min and Merge only keep such values. ∎
- *Fresh entries are minima and persist*: a fresh entry is the global
  minimum entry of its key, survives every keep-min and every Merge,
  and stops being fresh only when a later relaxation drops the label. ∎
- *Label-drop inventory*: a label drop always re-inserts fresh at the
  accepting site except at (i) FindPivots' k Bellman-Ford rounds (the
  one ungated relaxation site — Lemma A's note), whose damage is
  confined to W-territory of that call and repaired at its return
  (W′ settles, lines 31–33 re-insert S-remainders at current labels,
  Lemma 3.2's ⟨W, P⟩ covers the rest), and (ii) line-36 sweeps
  accepting below B′ (gate refuses the insert). Line-23 re-offers
  below B_i are exact ties — the child's clause 3 already placed the
  fresh entry in D_i — so no drop occurs there. ∎ (inventory)

**Base-case completeness is unconditional (proved).** During a base
call no other call runs, and every in-base label drop re-inserts, so
stored = label for every key throughout; pulls ascend by label. If a
pulled u were incomplete, the first incomplete vertex y on u's
shortest path has a complete predecessor z that is either already
pulled (its sweep then fresh-inserted y at dis(y) < d[u], and
dis(y) < B since stored_u < B — in-window) or still stored fresh at
dis(z) ≤ dis(u) < d[u]; either way an entry strictly below u's
contradicts u being the minimum pull, with the tuple order settling
exact ties. So every base-case pull settles a complete vertex,
independent of Q3 — which breaks only the bound clause. ∎
[Cold-audit correction 2026-07-16: NOT unconditional — the "or still
stored fresh at dis(z)" step silently assumes z's fresh entry sits in
THIS base D, i.e. covered seeds at call entry; that premise is free
only for the top-level call, whose D is the only D. "Independent of
Q3" survives. The entry condition is exactly L1's handoff, so the base
case moves inside the induction rather than anchoring it — see the
audit note at the end of addendum 8.]

**The repaired Frontier Split.** Lemma 3.6's hypothesis
{x ∈ Y : d[x] < B_s} ⊆ Y_s is stated over labels while Pull selects by
stored values — the stale-entry gap. But its proof consumes membership
only for the complete covering vertex y′ with d[y′] = dis(y′) < B_s:
if coverage is fresh, y′'s entry sits at its label and Pull captures
it (and line 13 captures the P̂ side by labels via clause 3). So the
sub-call precondition survives verbatim under fresh coverage. ∎
(conditional on L1)

**What remains — the proof's spine, two sub-lemmas.**

- **(L1) Fresh-coverage maintenance**: at every iteration boundary,
  every incomplete v ∈ Ũ_P has a COMPLETE coverer on its shortest
  path holding a fresh entry in D∪P̂. Initialization is Lemma 3.2 plus
  pivots inserted at current labels; the iteration step hands coverage
  to the child (pull + repaired split), receives it back through the
  child's clause 3 — which is precisely a freshness statement
  (R(B, U) entries at d[v] = d[u] + w_uv, current labels) — plus
  lines 31–33 at current labels, and Merge preserves fresh minima.
  The named hole: coverers whose labels drop mid-iteration through
  the two inventory sites of deeper active calls; both are confined
  to those calls' W/S territory and repaired at their returns, but
  the custody bookkeeping across the active stack is unwritten.
- **(L2) The endgame above B′** (W′ completeness and
  Ũ(B′, S) ⊆ U): the paper contradicts an incomplete v with
  d[v] < B′ via dis(coverer) = d[coverer] ≥ dB[D∪P] ≥ B′ — invalid
  under stale entries, and this consumption is FOR COMPLETE COVERERS,
  so the restricted clause 2 alone cannot feed it. The repair must
  show the fresh coverer u has dis(u) ≥ B′ directly: if dis(u) < B′
  then u is complete yet unsettled (a settled-and-unwound u would
  have completed v's path successor — Lemma A's windows tile
  [0, B_top), so the full unwind offers the next edge validly once),
  and an unsettled fresh entry at stored = dis(u) < B′ should have
  been pulled and settled by a child before B′ climbed past it —
  U_i = Ũ(B′_i, S_i) by induction. That last step consumes per-call
  B′ monotonicity (measured universal, including every escape row —
  Q15 finding 1 — but still unproved) and a capture argument ("B′
  never passes an unpulled fresh entry"), which is exactly the
  once-per-call machinery of Lemma F. For V5 the endgame must route
  differently — V5 regresses B′ (Q15) — consistent with conservation
  of trouble: each engine breaks exactly one hypothesis.

**Status.** (H) now rests on: base-case completeness (proved above,
unconditional) for every base pull, plus (L1) + (L2) for W′ appends
and the induction's handoff. The pre-test's confinement result — every
measured falsification of the paper's clause 2, on every family ever
constructed, is complete-keyed — is the empirical statement that no
third hole exists. Payoff unchanged: (H) proved discharges the
standing hypothesis of Lemmas B, D, F, G, H and the linear-excess
theorem (addendum 6), and upgrades V8 from "empirically safe" to
"provably safe modulo (L1)/(L2)".

## Q14 addendum 8 (2026-07-16, same day, unreviewed — every claim here is flagged for independent cold re-derivation before anything is built on it) — permanent freshness, canonical-prefix completeness, the incompleteness floor, a thought-experiment that probably falsifies the restricted clause as an invariant, and the escort invariant that replaces it. COLD-AUDITED same day (fresh session; note at end): M and O confirmed, triangle confirmed against paper + port, N restated for len/hops-completeness, escort statement needs two fixes before instrumenting

Written immediately after addendum 7 while attacking L1/L2. Three
lemmas hold at proof or sketch grade; the fourth item is a designed
counterexample to the restricted clause itself, which sharpens the
proof target rather than killing it.

**Lemma M (permanent freshness).** A fresh entry of a COMPLETE vertex
stays fresh forever, wherever keep-min and Merge carry it: its stored
value dis(u) is the key's floor (stored ≥ label ≥ dis-label), so no
later insert undercuts it and no keep-min evicts it, and a complete
label never moves. Fresh coverage by complete vertices never decays;
it is only ever CONSUMED by a pull. So L1's custody question is about
creation and pull-handoff exclusively. ∎

**Lemma N (canonical-prefix completeness).** Every vertex on the
canonical shortest path of a complete vertex is complete. At the
moment v became complete, the accepting candidate pinned its canonical
predecessor's label to the canonical prefix value (Lemma B read
backward), so the predecessor was complete at that moment, and
completeness is permanent; induct along the prefix. ∎

**Lemma O (incompleteness floor, base-settled predecessors).** At any
iteration boundary with active stack X_0 ⊃ … ⊃ X_q: if y is
incomplete, dis(y) < B_{X_0}, and y's canonical predecessor z was
settled by a base-case pull, then dis(y) ≥ B_{X_q}. Sketch: the base
settlement swept [0, B_base) at once and the line-30 unwind tiles
[B_base, B_{X_0}) with ancestor windows (Lemma A); the canonical offer
(z, y) carries the fixed value dis(y) (Lemma N) and lands in exactly
one window; sweeps run in the same iteration as each return, so the
pending windows at any boundary belong to strict ancestors of the
batch's current custody call — all on the active stack, with lower
ends ≥ B_{X_q} by bound nesting. y incomplete means its canonical
offer is pending (a swept canonical offer completes y), so dis(y)
sits in a pending window. ∎ (sketch) — The W′-settled-predecessor
case is NOT covered: a W′ settlement sweeps only [B′_Y, B_Y), and the
range [0, B′_Y) is vouched by U_Y = Ũ(B′_Y, S_Y), which reaches y
only through S_Y. The one channel that survived every previous
closure survives this one too.

**The base-exit triangle (thought experiment — the restricted clause
is probably NOT an invariant).** Take a base call whose cutoff-ending
settlement u_last has two out-edges: w₁ to z and w₂ to y, plus an edge
z → y, with d[u_last] + w₂ > d[u_last] + w₁ + w_zy, and the rest of
D's remaining values (which set the returned B′) far above. The final
post-pull sweep relaxes both edges in adjacency order: z becomes
complete at dis(z) = d[u_last] + w₁, y receives the NON-canonical
value d[u_last] + w₂ < B′ while its true path runs through z — an
INCOMPLETE key with a fresh entry below B′, exactly what the pre-test
counter watches. The measured zero across the matrix is then a family
artifact: no tested input builds this triangle. To be confirmed by a
dedicated driver (predicted: the incomplete-restricted counter fires
while distances and (H) hold — which would also re-validate the
counter's semantics on a live positive).

**The escort invariant (the corrected proof target).** In the
triangle, (H) still survives, and the reason generalizes: z's fresh
COMPLETE entry at dis(z) < d[y] was created by the same sweep into the
same D and travels with y's entry through every merge (custody is
shared until a pull separates them). Value-ordered pulls take z first
— in an earlier batch, whose settlement sweep immediately relaxes the
canonical (z, y) and completes y before y's entry can surface; or in
the SAME batch, where the child receives both, settles z first by the
same argument one level down, and completes y before pulling it. If a
chain z → y₁ → … → y_r hangs below B′ with only z complete, each
settlement completes the next link and fresh-inserts it below the
following one — the escort regenerates down the chain. Proposed
invariant, replacing the restricted clause:

> Every fresh entry of an incomplete key below B′ coexists in its D
> with a strictly smaller fresh entry of a complete key from which a
> chain of pending canonical offers reaches it.

(H) then follows at pull sites: a pulled minimum is either complete,
or its escort — strictly smaller — was pulled before it and completed
it first, contradiction. The measured zero-violation state is the
special case where no incomplete entry sits below B′ at all; the
escort form is what the algorithm actually guarantees. What this
does to the remaining spine: L1's decay half is closed by Lemma M;
L2's capture argument becomes "escorts are pulled before wards",
which within one D is the data structure's value order and across a
batch boundary is the settlement sweep running before any later pull
— both mechanical, neither consuming B′ monotonicity. What still
consumes B′ monotonicity is the W′ endgame (Lemma O's named gap plus
addendum 7's L2), unchanged.

Next steps, in order: (i) independent re-derivation of Lemmas M/N/O
and the escort argument — the window bookkeeping in O and the
batch-boundary case of the escort were written fast and unreviewed;
(ii) the triangle driver, added to the standard matrix; (iii) the
escort invariant instrumented (for each sub-B′ incomplete entry,
verify a smaller complete co-resident chain-connected entry exists);
(iv) the W′ endgame with the floor lemma in hand.

**Cold audit (2026-07-16, fresh session) — the attack plan's item 0
executed over addenda 5–8 and Q15.** (a) Every quoted number
re-traced to the recorded files (escape.txt, restricted_invariant.txt,
dup_classify.txt, cascade.txt): all exact. (b) Every paper citation
re-checked against paper_text.txt — Algorithm 3 lines 7/13/23/24/29/
30/31–33/34/36, Lemma 3.6's hypothesis and what its proof consumes,
Lemma 3.7's three clauses, Observation 2.1's dB[Y], §3.4 property 6's
per-level windows, Algorithm 4's pull-then-sweep order: all faithful.
(d) escape, dup_classify and verify_contract re-run under
verify-contract + dup-classify: outputs reproduce the recorded files
exactly. Two in-place corrections above came out of (a)/(c): the Q15
chunk-formula floor/ceiling and the addendum-7 base-case
"unconditional". (c) Adversarial re-derivation, item by item:

- **Lemma M confirmed.** One step made explicit: an entry leaves a D
  only through pull or keep-min, and keep-min (strict `<`, ties keep
  the incumbent) can never evict a key's floor. Merge is per-key
  keep-min, so custody reduces to creation and pull-handoff as
  claimed.
- **Lemma N corrected: completeness must be read as len/hops-
  completeness.** The acceptance that completes v pins its
  predecessor's len and hops to the canonical prefix values — NOT the
  predecessor's own pred field, so the full-4-tuple induction breaks
  at the second step (a predecessor can carry a non-canonical pred at
  exact len/hops). Since relax candidates read only len/hops of the
  source (Lemma D's own note) and the instrumentation defines
  incomplete by length, len/hops-completeness is the right notion
  everywhere in addenda 7–8, and Lemma N's "canonical path" is the
  accepted-pred tight path — a minimal-(len, hops) path, not
  necessarily the pred-id-tie-broken one. No downstream consumer
  needs more.
- **Lemma O confirmed at sketch grade.** Wording fix: an ACCEPTED
  offer completes y (the unwind sweep merely triggers acceptance;
  refusal is only ever by gate, since a label at cand is already
  complete). The window bookkeeping reconstructs cleanly: pending
  windows at a boundary belong to strict ancestors of the batch's
  custody call, all on the active stack, lower ends ≥ B of the
  deepest active call by bound nesting. The W′ gap stands as named.
- **The triangle confirmed as real mechanics against BOTH engines of
  truth**: the paper's Algorithm 4 fixes B′ at the last Pull (line 4)
  before the final sweep's inserts (lines 6–8), and the port does the
  same (base_case: bound set at pull, the Q3-flagged insert after).
  The measured zero is a family artifact as suspected; the driver
  prediction stands.
- **The escort invariant: mechanism survives adversarial probing —
  same-batch and split-batch handoffs (pull is a value-prefix, so a
  pull can never take the ward without its strictly smaller escort),
  chain regeneration, zero-weight links (strictness survives in tuple
  order through hops) — but the STATEMENT has two defects to fix
  before instrumenting.** (i) "Chain of pending canonical offers" is
  wrong for chains of length ≥ 2: intermediate offers need not exist
  yet — plant u_last → y₂ directly and neither (z, y₁) nor (y₁, y₂)
  is pending until z settles. Re-draft: a canonical tight-edge chain
  from the escort to the ward whose intermediates are each either
  unlabeled or co-resident below the next link; regeneration (each
  settlement completes the next link and fresh-inserts it below the
  following one) is the induction step. (ii) The fresh-only
  restriction does not give (H) at pull sites by itself: a STALE
  entry of an incomplete key below B′ — creatable by the two
  non-re-inserting label-drop sites (FindPivots rounds, line-36
  accepts below B′), since the re-insert of a drop lands at the
  accepting site's D, not necessarily where the old entry lives — is
  outside the invariant's scope, and a pull consuming it settles the
  vertex at its current incomplete label. The escort form needs a
  companion clause: either addendum 7's confinement (stale incomplete
  entries exist only for W/S territory of active deeper calls,
  repaired at their returns) promoted to invariant grade, or a proof
  that a stale incomplete entry below B′ always coexists with a
  smaller fresh entry of the same key in the SAME D. This, alongside
  the W′ endgame, is the open drafting question — the instrumentation
  (next-steps item iii) should count stale-incomplete sub-B′ entries
  separately to decide which clause is true.

Net effect on the spine: L1's decay half stays closed (M holds);
the base case now consumes the co-residence premise instead of
anchoring it, which makes the escort invariant — not the restricted
clause — the single load-bearing target, as this addendum proposed;
L2's plan is unchanged. Order of work after the audit: triangle
driver first (validates the counter semantics on a live positive),
then the escort re-draft with the stale-entry clause decided by
instrumentation, then the W′ endgame.

## Q14 addendum 9 (2026-07-16, fresh session) — the triangle driver: the restricted clause falsified live as predicted, (H) measured directly for the first time (zero violations everywhere), and the stale-incomplete channel empty across the whole matrix. CONFIRMED (recorded runs)

Addendum 8's next-steps items (ii) and (iii)'s counting half, executed.
Driver `examples/triangle.rs` (record `benchmarks/results/triangle.txt`);
two new observation-only counters (features verify-contract +
dup-classify, results unchanged, both suites green with the features on
and off):

- `stale_incomplete_{base,rec,iter}` (RestrCounts): check-site events
  where the D holds an entry strictly below the bound whose vertex is
  incomplete AND whose stored value differs from the vertex's current
  label — the entry a fresh-only co-residence invariant cannot see
  (addendum 8's defect ii).
- `settle_incomplete` (Solver): the direct (H) counter — settlement
  events (base-case pulls and W′ boundary appends) whose label length
  differs from the oracle's true distance at settlement time. Until now
  (H) was only ever inferred through dup/distance proxies.

**Part 1a — the deterministic live positive.** The star: leaves 1..8
at weights 1..8 fill the t=2 base cutoff, ballast at 50 is the one
remaining D value at the last pull (so B′ = 50 by the fixed-at-pull
rule), and the triangle hangs off leaf 8: z at w 1.25, y at w 2.5
direct while the true path is 8 → z → y = 9.5. Paper and V8 return
B′ = 50 with the leaked pair in D — z stored 9.25 = true (the complete
escort), y stored 10.5 ≠ true 9.5 — and `incomplete_base` fires: 1.
The restricted clause is NOT an invariant, exactly as the addendum-8
thought experiment predicted; the measured zero of the pre-test was a
family artifact, and the counter's semantics are now validated on a
live positive. V5 caps the bound to 9.25 (cap_hits=1) and shows zero —
consistent with its all-zero record. Stale-inc 0 (the leaked entry is
fresh); settle-inc 0.

**Part 1b — the escort finishes the job.** Full recursion on the same
star: distances exact, dups 0, settle-inc 0, with the fire visible at
base and iter sites (1/0/1). y is completed by z's settlement sweep
before anything can pull it at the stale value.

**Part 2 — the scaled family** (gate + unit path, the triangle on
every stride-th path vertex, ballast block above; stride ∈ {2, 3}
keeps the settlement period t³+1-coprime so chunk-end phases rotate).
The stride 0 control reproduces escape's contained-L=200 numbers
exactly (unsettled 22/6/28, incomplete 0) — cross-driver confirmation
of the chunk arithmetic. With triangles, every run is distance-exact,
V8 dup-free, settle-inc 0, and `incomplete_base`/`incomplete_iter`
fire at scale: paper 12 (L=200, stride 2), 312 (L=5000, stride 2),
75 (L=5000, stride 3), 1250 (L=20000, stride 2); V8 fires with
different counts (6/147/208/588) — the count is a chunk-phase
artifact of each engine, the existence is the finding. Two
observations for the record: `incomplete_rec` stayed 0 everywhere
(on these families the leak is a base-return phenomenon; the stride-3
run's 101 recursive-return bound violations were all complete-keyed),
and on the stride-2 rows the paper engine's duplicate settlements
coincide with the fire counts (12/312/1250) — the leaked entry is
consumed as a benign duplicate of an already-complete vertex, i.e.
the dup channel and the leak are the same event chain seen at two
sites (stride 3 breaks the coincidence: 303 dups vs 75 fires).

**The matrix sweep** (record `benchmarks/results/stale_incomplete.txt`):
escape (all three parts), dup_classify, and verify_contract re-run
with both new counters. Every previously recorded number
(restricted_invariant.txt) reproduces unchanged, and both counters
are ZERO on every family, engine, and check site. Consequences:

1. **(H) now has a direct measurement, and it is zero everywhere** —
   including the escape waves, whose thousands of paper-engine
   duplicate settlements therefore all re-settle already-complete
   vertices at the true length. Every measured settlement event in
   the corpus settles a complete vertex.
2. **The escort's companion clause: the stale-incomplete channel is
   empty at every check site.** This is the instrumentation decision
   addendum 8 asked for: the fresh-only escort scope suffices
   empirically, and the invariant-grade candidate for the companion
   clause is addendum 7's confinement (stale incomplete entries live
   only in W/S territory of active deeper calls, repaired at their
   returns) rather than a same-D fresh-companion claim. Caveat kept
   explicit: the counters observe return/iteration boundaries, not
   every mid-iteration state; the proof still owes the confinement
   argument itself.
3. The proof target is now cleanly the escort invariant (addendum 8's
   statement with defect i's chain re-wording and defect ii resolved
   toward confinement). Next per the plan: the escort re-draft, its
   own instrumentation (a strictly smaller complete co-resident
   chain-connected entry per sub-B′ incomplete fresh entry), then the
   W′ endgame.

## Q14 addendum 10 (2026-07-16, fresh session) — the escort invariant re-drafted in last-complete-vertex form (both addendum-8 defects discharged), the regeneration induction with its single leaking case identified as the W′ channel, and the escort instrumented live: zero missing escorts across the matrix and the triangle family

Addendum 9's consequence 3 executed: the re-draft, then the escort's
own counters over everything.

**The re-drafted statement.** Addendum 8's defect i (the "chain of
pending canonical offers" wording) is repaired by quantifying over
tight paths and anchoring the escort at the last complete vertex.
Definitions pinned first: a *tight path* to y is a source-to-y path
every edge of which satisfies dis(u) + w = dis(v) exactly (float
equality — the arithmetic the labels themselves live in; at proof
grade, (len, hops)-tight per the len/hops-completeness of the
addendum-8 audit). *Complete* means the label's len equals dis
(len/hops at proof grade); *fresh* means the stored value equals the
key's current label. On any tight path to an incomplete y a last
complete vertex exists (the source is complete, y is not).

> **(E)** At every observable boundary (base and recursive returns,
> against the returned B′; end of every BMSSP iteration, against the
> current B′), for every fresh entry of an incomplete key y stored
> strictly below the bound in the checked D: some tight path to y has
> its last complete vertex z holding a fresh entry in the SAME D.

Two clauses of the addendum-8 statement are no longer assumed — they
are consequences of the anchor choice:

- *Strictness*: z's fresh entry sits at len dis(z) ≤ dis(y) <
  len(y's stored entry) — the first inequality because z lies on a
  shortest path to y under non-negative weights, the second because
  labels are lengths of real paths and y is incomplete. So the escort
  is strictly below the ward already in the len slot of the value
  order, hence contained in every pull prefix that contains the ward.
- *Intermediates*: every vertex strictly between z and y on the
  chosen path is non-complete (unlabeled or incomplete) by the
  maximality of z. The old "unlabeled or co-resident below the next
  link" clause dissolves into the choice of anchor.

**Companion clause (defect ii, resolved per addendum 9).** (E)
quantifies over fresh entries only. The stale channel — entries of
incomplete keys below the bound at a non-label value, creatable by
the two non-re-inserting label-drop sites — is covered by addendum
7's confinement, promoted to companion:

> **(E-c)** Stale entries of incomplete keys strictly below the
> governing bound do not exist at observable boundaries; mid-
> iteration they are confined to the W/S territory of active deeper
> calls and are repaired at those calls' returns.

Measured status: the boundary half is the `stale_incomplete_*` zero
across the whole corpus (addendum 9); the mid-iteration half is the
part the counters cannot see and the proof still owes.

**(E) ⇒ (H) at base pulls, mechanically (modulo E-c).** Induct on
settlement events in time; let the first (H) violation be a base pull
settling an incomplete y. The pulled entry is fresh (a stale entry
below the bound is excluded by E-c at the pull moment — the mid-
iteration half of E-c is consumed exactly here). By (E) at the seeding
of this base call plus the regeneration step below, y's current escort
z holds a fresh co-resident entry strictly below y's — so z, not y,
would be the minimum: z must have been pulled in an earlier singleton
of the same base loop. That pull settled z complete (minimality), and
the base case's post-pull sweep relaxed the tight edge (z, x₁) — x₁
non-complete means its label tuple strictly exceeds the canonical
candidate, so the offer is accepted, x₁ completes, and the insert
lands in this same base D (the candidate value dis(x₁) ≤ dis(y) <
y's stored len < the base bound, so the sweep's bound gate passes).
The chain from the last complete vertex to y is now strictly shorter;
induct on its length — y completes before any pull can take it. ∎
(modulo E-c's mid-iteration half)

**The regeneration induction (why (E) survives events), case
inventory.** Events that touch the invariant:

1. *Escort consumption by keep-min or merge*: impossible — Lemma M
   (a fresh entry of a complete key is the key's floor; keep-min
   cannot evict it, and a complete label never moves).
2. *Escort pulled without the ward* (the prefix ends between them):
   the escort travels to the child and is settled there or deeper;
   every settlement of it triggers the accepted canonical offer to
   the next chain link, whose insert lands in the settling call's D —
   which merges back into the parent's ds at the return inside the
   SAME parent iteration. Co-residence is restored by the boundary,
   and the window in between is pull-free at the parent (one pull per
   iteration, and the child cannot pull the ward — the ward is not in
   the child's D). This is why a boundary-grade invariant suffices
   for pull sites.
3. *Escort pulled with the ward*: both land in the child (pulls are
   value-prefixes and the escort is strictly smaller); (E) holds one
   level down and the argument recurses.
4. *Chain advance — the leaking case.* If the escort z is settled by
   a W′ boundary append rather than a base pull, z's out-edges are
   relaxed not by z's own settlement but by the parent's relax sweep
   (lines 21–26 of the port; the paper's 31–33) in the iteration of
   the child's return — and that sweep's insert is gated by
   d[v] ≥ B′_i (the child's returned bound). An accepted canonical
   offer BELOW B′_i completes x₁ but creates NO entry anywhere: the
   ward's last complete vertex is then entry-less and (E) as stated
   is violated. This is Lemma O's named W′-settled-predecessor
   channel and addendum 7's L2, reached from a third independent
   direction. Note the knot: the child-return contract
   U_i = Ũ(B′_i, S_i) would say x₁'s predecessor situation cannot
   arise — but that contract is what (H) is being proved FOR. The
   escort invariant's single leaking case IS the endgame's mutual
   induction, to be untied in the W′ session, not here.
5. *Ward creation sites* (a fresh incomplete entry appearing below
   the bound): base-case final-sweep non-canonical plants (the
   triangle), iteration relax-sweep plants, pivot re-selection
   re-inserts at current labels, line-33 premature returns to the
   frontier, FindPivots-seeded initial inserts, and merge carries.
   The two sweep cases carry the escort with them (the same sweep
   source is settled-complete and its tight out-edge went into the
   same D — the triangle's mechanics generalized); the re-insert
   sites are NOT argued here and are exactly what the instrumentation
   quantifies over.

**The instrumentation.** Counters `escort_wards_{base,rec,iter}` and
`escort_missing_{base,rec,iter}` at the same three sites as the
restricted-invariant counters: wards = fresh entries of incomplete
keys strictly below the site's bound in the checked D; for each ward
a backward search over exactly-tight edges (reverse adjacency, built
lazily) that traverses only non-complete vertices and succeeds when
it reaches a complete vertex holding a fresh entry in the same D
strictly below the ward's entry in the value order. Zero missing is
the invariant surviving at measurement grade; any fire falsifies (E)
as stated (the search accepts a superset of canonical chains, so a
miss means no tight chain at all). Observation-only, features
verify-contract + dup-classify, results unchanged.

**Predictions written before the runs**: wards fire exactly where
`incomplete_*` fired (the star part 1, the stride rows of the
triangle family; zero on the whole standard matrix), and
escort_missing = 0 everywhere — no tested family builds case 4's
sub-B′_i W′ configuration.

**Results (recorded run: `benchmarks/results/escort.txt`).** Both
predictions confirmed, and the search itself validated on live
positives:

- *The star (part 1a, paper and V8)*: the one ward — y's leaked entry
  at stored 10.5 — finds its escort: z, complete, fresh at 9.25 in the
  same base D, one tight edge away. wards=1, missing=0; V5 zero wards
  (the cap removes the leak). Full recursion: wards base/rec/iter =
  1/0/1, missing all zero.
- *The scaled family*: wards coincide event-for-event with the
  `incomplete_*` fires on every row (paper 12/312/75/1250 on the
  stride rows, V8 6/147/208/588; wards_rec = 0 everywhere, matching
  incomplete_rec = 0) and **missing = 0 on every row, engine, and
  site** — every leaked incomplete entry these families can produce
  carries a strictly smaller complete fresh co-resident entry reached
  by a tight chain through non-complete vertices.
- *The standard matrix* (escape all three parts, dup_classify,
  verify_contract): wards = 0 everywhere — the matrix never presents
  a sub-bound incomplete fresh entry at a boundary at all, consistent
  with its all-zero `incomplete_*` record — and every previously
  recorded number (triangle.txt, stale_incomplete.txt,
  restricted_invariant.txt) reproduces bit-for-bit with the escort
  columns stripped.

Consequences: (E) survives at measurement grade on every family ever
constructed, including the purpose-built falsifier of the restricted
clause — the corrected proof target now has direct empirical support,
not just the triangle thought-experiment. The base-pull half of (H)
rests on (E) plus E-c by the mechanical argument above. Standing
caveats: the counters observe boundaries, not every mid-iteration
state (E-c's mid-iteration half is unmeasurable by construction), and
no tested family exercises regeneration case 4 — wards_rec = 0
everywhere means the recursive-boundary configuration, where the
W′-channel leak would surface, has never been built. A driver that
forces a W′-settled escort with a sub-B′_i canonical successor is the
natural companion to the W′ endgame session, where the leaking case
and Lemma O's named gap are the same knot.

## Q14 addendum 11 (2026-07-17, fresh session; the confinement argument is same-session math, flagged for cold re-derivation like addendum 8 was) — the W′ endgame: case 4 cannot be assembled on any engine; four named repair mechanisms, each exhibited live by a dedicated probe; the first recursive-boundary ward fires with its escort found; the custody object of L1 materialized and its repair channel demonstrated

The attack plan's spine item 1 executed: instead of one driver that
fires case 4, the session produced a proof-shaped inventory of WHY
every assembly attempt fails, with one probe per blocking mechanism
(driver `examples/wprime.rs`, record `benchmarks/results/wprime.txt`;
all predictions below were written into the driver header before the
first run and confirmed on it, every row, every engine).

**The construction problem.** Case 4 needs a W′-settled escort z
(z ∈ W of its call, complete, d[z] < B′ at the W′ computation, never
pulled) whose canonical successor x₁ has dis(x₁) below the governing
bound while x₁'s completion creates no entry. Silence is the crux:
z must never hold an entry in any live D, or a pull settles it and
the complementary gates regenerate x₁'s entry. Working through the
port's mechanics (all sites verified against the code, not memory),
every channel that could complete z silently is blocked:

1. **Pocket closure** (row A). A FAILED FindPivots search (heap
   emptied with |K| < k) pops everything it labels, so it enumerates
   the entire region reachable from its root through improvable labels
   under the call's B. If the pocket's labels are exact — every true
   path enters via the root x ∈ S, whose own label is exact — the
   local search is a Dijkstra of the pocket: every successor with
   candidate < B joins it (relax accepts ties, so equal labels join
   too; cand > d[v] with cand = dis(v) is impossible below the true
   value; a smaller tuple at equal len means v is already
   len-complete). Sub-B′ successors of W′-settled pocket vertices are
   therefore themselves pocket members and W′-settle complete in the
   same append. Row A: the pocket {x, z} W′-settles complete, all
   counters zero, all engines.

2. **FindPivots-time healing** (row B). A stale pocket whose true
   path enters via another FAILED root is re-explored exactly at
   FindPivots time: failed searches pop everything, so the improving
   search re-relaxes the successor chain in place — z AND x₁ are
   complete before the main loop starts, and both W′-settle. Row B:
   z@16, x₁@18 healed and settled, counters zero.

3. **Tie-restore plus complementary gates** (row D). Line 23 accepts
   exact ties (cand ≤ d[v]), so every pull-settled vertex re-offers
   its canonical successor even when the successor's label is already
   final; base-case inserts are ungated (the Q3 site), and outside
   the base the two gates are complementary by construction: the
   child's relax refuses exactly when cand ≥ b_i, which is exactly
   when the parent sweep's insert gate d[v] ≥ b_i accepts. A
   pull-settled vertex's sub-B successor ALWAYS regains an entry in
   some live D — so a stale W-vertex improved by a pulled chain is
   itself re-entried and pull-settled, never W′-settled. Row D: r@14
   pulled, the base child re-walks r → c₁ → z by tie-restores and
   settles z by pull; z → x₁ (990) is refused inside the child
   (990 ≥ 16) and accepted at the parent sweep, whose gate INSERTS
   x₁@990 into the parent D. x₁ ends complete WITH an entry.

4. **The separator cap** (row C — the one genuinely new lemma-shaped
   fact). The only silent-completion channel left is a CAPPED tree
   search: the improving search hits |K| = k exactly on z, so z is
   improved at FindPivots time with no insert and is never popped —
   z's stale pocket had already refused z → x₁ at the stale candidate
   (stale 30 + 975 ≥ B = 1000) while the true value slid under the
   bound (22.75 + 975 = 997.75 < B): the stale-label inflation of
   Observation 2.1's gap, materialized. But z's true path passes
   through the tree root r ∈ S, and r holds a fresh entry (pivot
   insert) at d[r] < dis(z). Pull separators are minima of the
   remaining D, so B′ ≤ every value that stays unpulled at exit:
   either r is pulled — mechanism 3 walks the chain and z is
   pull-settled — or r's entry survives and B′ ≤ d[r] < dis(z), so
   z ∉ W′ and the case-4 premise dies. Row C, measured: B′ = 21 ≤
   d[r] = 21.75 < dis(z) = 22.75; z returned complete, entry-less,
   unsettled, NOT in U; x₁ still unlabeled at d = ∞; r's entry at
   21.75 sits in the returned D. All counters zero.

**The custody object, materialized.** Row C's z is exactly the object
L1's bookkeeping owes: complete, entry-less, unsettled, invisible to
the parent (not in U, not in S, no entry anywhere). Its custody is
carried NOT by z but by the fresh entry of its improver root r — via
the unpulled pivot entry, or line 33's premature return (b_prime ≤
d[r] < B re-inserts r at the boundary). The full-recursion runs of the
same graphs (part 2 of the record) show the repair end-to-end: the
parent re-pulls r, the child's FindPivots re-explores the subtree, and
every engine finishes distance-exact with zero wards. This names L1's
invariant in frontier form: every un-walked completion has, on its
true path, a fresh entry held by the active stack at value ≤ its dis —
(E)'s escort at the frontier, anchored at S/pivot/premature-return
entries. The mutual induction with the child contract is coherent, not
circular: the anchor exists at the completion event, persists by
Lemma M, and caps every deeper bound through the separators before any
boundary where case 4 could fire.

**Case 4's status.** On these mechanics the case-4 configuration
cannot be assembled in-call: every W′-settled vertex is either
exactly labeled by its own pocket search (closure ⟹ its sub-B′
successors W′-settle with it) or was improved and then either
pull-settled (mechanisms 2–3, not W′) or capped out of W′ (mechanism
4). What remains OPEN is the cross-call channel: a pocket search
refusing into a vertex completed by an EARLIER call (labels persist
globally; the entry may live in an ancestor's D or have been
consumed). The settled sub-case discharges (the earlier settlement
explored the successors under its own gates, entries merged up); the
stranded sub-case reduces to the row-C anchor one level up (the
anchor's value < dis of everything it hides, and bound nesting
carries the separator cap down the stack: b_child ≤ parent separator
≤ anchor). Writing that stack-nesting bookkeeping down formally is
Lemma O's window argument — the remaining formal debt. The channel
inventory is now complete and each channel has a named cap; the knot
is no longer "unknown leak" but "one bookkeeping lemma."

**The first recursive-boundary ward** (row E). A base-exit triangle
planted at the cutoff exit of the LAST child before the parent's |U|
limit exit survives the merge into the parent's returned D: wards_rec
fires for the first time on any input ever constructed (addendum 10's
"case untested" caveat discharged on the co-residence side), and the
escort search finds z_t — complete, fresh, co-resident, one tight
edge below — so missing_rec = 0. incomplete_rec fires event-for-event
with wards_rec (1/1), stale-inc stays zero, settle-inc stays zero,
and the full-recursion embedding stays distance-exact with
missing = 0 while wards fire at base/rec/iter = 1/1/2 (paper and V8;
V5's base cap removes the leak, B′ capped to the pair's floor at
20.84375 — same pattern as the triangle star).

**Consequences for the spine.**

- (E)'s regeneration case 4 is CLOSED at in-call grade by mechanisms
  1–4 (sketch; cold re-derivation owed), and the cross-call residue
  is the same statement as L1's custody and Lemma O's windows — the
  three open items of addenda 7–10 are now literally one lemma: the
  active-stack anchor/window bookkeeping.
- L2's capture argument ("B′ never passes an unpulled fresh entry")
  is no longer a Lemma-F import: it is the separator cap, measured
  live in row C, and it is PER-EXIT and local — B′ monotonicity
  across iterations is NOT consumed by this route. The B′-monotonicity
  multi-front attack (queued item 2) may accordingly shrink to the V5
  endgame only (V5 regresses B′ by design, Q15, and routes the
  endgame differently).
- E-c's mid-iteration half gains evidence but not a proof: crafted
  stale pockets (rows C, D) never produce a stale-incomplete entry
  anywhere (stale-inc 0/0/0 across the driver), consistent with the
  two label-drop sites never firing below the bound on these shapes.
- The watch-list's mid-unwind case in L2 remains to be handled inside
  the window lemma, not separately.

Next session: the anchor/window lemma proper (Lemma O's sketch
upgraded to cover W′-settled predecessors using the separator cap and
Lemma M), which — if it closes — discharges (H) modulo the E-c
mid-iteration half, i.e. the last two debts merge into one.

## Q14 addendum 12 (2026-07-17, fresh session) — cold audit of addendum 11: numbers and mechanisms confirmed, but the silent-completion inventory missed a fifth channel (merge-abort), probed live as row F; the anchor/window lemma statement corrected to proxy anchors with boundary collapse

Item 0 of the cycle, per the standing rule that same-session math gets
a cold re-derivation before anything builds on it. Verdict: addendum
11's numbers are exact against `benchmarks/results/wprime.txt`, the
re-run reproduces every row bit-for-bit on every engine, and
mechanisms 1–4 re-derive cleanly against the port's code — but the
channel inventory has a real gap, and closing it corrects the
anchor/window lemma's statement before it is attempted.

**Re-derivation notes (all against `src/solver.rs`, not memory).**

- Mechanism 1 (pocket closure): a FAILED search's heap empties, every
  accepted improvement re-pushes its vertex, and stale heap entries
  are skipped at pop — so the last pop of each pocket vertex relaxes
  all its edges at its final label. The refusal case at equal length
  (candidate above the stored tuple because of hops/curr/pred order)
  leaves the target len-complete, which is the completeness grade
  (H) is stated at (addendum 8's Lemma N standing). Confirmed. One
  sharpening: the W′ relax sweep re-offers each pocket vertex's
  CURRENT pred edge tuple-exactly, so the above-B′ residue of a
  closed pocket regains entries through the d ≥ B′ gate — mechanism
  3's complementarity holds at the W′ sweep too, not just at pulls.
- Mechanism 2 (FindPivots-time healing): same exhaustiveness
  argument; the healing conclusion needs the improving search to
  FAIL, which is exactly what row B builds. Confirmed as stated.
- Mechanism 3 (complementary gates): the child's relax bound is b_i
  and the parent sweep's insert gate is d[v] ≥ b_i, complementary by
  the same comparison; the "v already better via another path" case
  leaves v len-complete with custody on its own path. Confirmed.
- Mechanism 4 (separator cap): B′ = the last child's returned bound
  ≤ its b_i ≤ every value still stored at that pull, so an unpulled
  anchor entry caps B′. Zero-weight paths do not break it: if
  dis(z) = dis(r), the W′ filter's STRICT d[z] < B′ ≤ d[r] = dis(z)
  still excludes z, and in tuple order the anchor's entry (fewer
  hops) sits below z's label at equal length. Confirmed, slightly
  stronger than stated.
- Lemma M's use (keep-min cannot evict): `BlockDs::insert` keeps the
  smaller value, `merge` is inserts, `pull` removes only pulled keys.
  Confirmed.
- The relabel-site inventory behind the whole argument is exhaustive:
  `relax` has exactly four call sites — FindPivots search (no
  insert), base case (ungated insert, the Q3 site), parent relax
  sweep (gate d ≥ b_i), W′ sweep (gate d ≥ B′). The sub-B′ successor
  of a W′-settled vertex, which the W′ sweep's gate refuses to
  insert, is covered by closure: it is a member of the same failed
  pocket and W′-settles in the same append (or was already settled).

**The correction — a fifth silent-completion channel.** Addendum 11
claimed "the only silent-completion channel left is a CAPPED tree
search." False: the search loop has THREE exits — heap empty
(failed), |K| ≥ k (capped), and tree contact (merged, the
`break 'search` after an accepted relax into a tree-stamped vertex) —
and the MERGED exit aborts an improving search exactly like a cap. An
improver that relaxes into the stale z (z joins K, is pushed) and
then contacts an existing tree on a later edge of the same pop
completes z silently: z is never popped, z's successor is never
re-relaxed. Unlike the capped case, the improver root r gets NO tree
of its own and NO pivot entry of its own — r is swept into the
CONTACTED tree's partition, and its P_j group may elect a cheaper
merged root as pivot. Custody of z then rides a PROXY: an entry whose
vertex is not on z's true path at all, at value ≤ d[r].

Row F (driver updated, record re-run; predictions in the driver
header written before the first run and confirmed exactly, every
figure, every engine): x@20 plants the stale pocket {x, z} (z stale
30, z → x1 refused at 1005 ≥ B, true 997.125 hidden); g@21.75 caps a
3-leaf tree; r@21.5 improves z to its true 22.125 and merges into
g's tree before popping z; s4@21 merges the same way; the partition
groups [h2, r, z, s4] so P_j = [r, s4] with pivot s4@21 — r
entry-less, z's custody on the proxy. Ballast forces the |U| limit
exit at separator 20.5. Measured: B′ = 20.5 ≤ d[s4] = 21 < d[r] =
21.5 < dis(z) = 22.125; z complete, entry-less, unsettled, not in W′
(its W membership from x's failed search is filtered out by 22.125 ≥
B′); x1 unlabeled; x W′-settles complete; all counters zero, all
three engines identical; full recursion (n = 90) distance-exact with
all counters zero.

**The boundary-collapse fact (lemma-shaped, new).** The proxy is
strictly mid-call state — at every call boundary custody
re-materializes as entries: (i) line 33 hands every S-member with
B′ ≤ d < B a direct entry (row F's r@21.5, live); (ii) an S-member
with d < B′ is in W (failed root) and W′-settles, or was pulled; and
(iii) B′ cannot pass an unpulled proxy entry (the separator cap),
while consuming it pulls the pivot, whose expansion/re-selection
re-frontiers or re-entries the represented root (pivot bookkeeping,
lines 27–29). So "every un-walked completion has a fresh entry on
its true path held by the active stack" is wrong as stated but right
at boundaries: mid-call the entry may be a P_j proxy at value ≤
dis of the represented true-path vertex; at any return boundary the
active stack again holds direct entries whose values cap B′ below
the dis of everything hidden.

**Consequences.** Case 4 stays closed — the fifth channel is blocked
by the same separator cap, now via the proxy — and the in-call
inventory is complete on the code's control flow (three search
exits + the pulled case + the base case's ungated inserts). The
anchor/window lemma to be attempted next must be stated with proxy
anchors and the boundary collapse, not with on-path entries: every
un-walked completion z has a true-path vertex x′ (the healing front,
an S-member of some active call) that is entry-held or
P_j-represented at value ≤ dis(x′) ≤ dis(z), bound nesting carries
the cap down the stack, and boundaries collapse proxies back to
direct entries. The merge-channel analysis and the boundary-collapse
fact are same-session math — flagged for cold re-derivation before
the lemma builds on them.

## Q14 addendum 13 (2026-07-17, fresh session) — cold audit of addendum 12, and its consequence: the separator cap has a timing hole, the W′ filter never consults D, and the two compose into a live falsification of (H) and of Lemma 3.7's completeness clause; V5 closes the hole exactly

Item 0 of the cycle: cold re-derivation of addendum 12's same-session
math (the merge-channel analysis and the boundary-collapse fact)
against `src/solver.rs` and the paper text, then whatever the audit
forces. It forced a falsification probe.

**Cold audit of the merge channel — CONFIRMED, with one unification.**
Row F re-derived by hand from the driver's construction: x's failed
search plants {x, z} with z stale at 30 and the z → x1 offer refused
at 1005 ≥ B; g's search caps at |K| = 4; r's search improves z to its
true 22.125, pushes it, and merges on the same pop's second edge
(r → h2, 21.84375 < 21.875) before z is popped; s4 contacts the same
tree at 21.828125; the partition of the merged tree from root g walks
h2's subtree into the group [h2, r, z, s4] (DFS trace against
`partition_tree` confirmed), so P_0 = [r, s4] with argmin pivot s4@21,
r entry-less, custody of z on the proxy. Separators: five 8-settle
children exit the |U| limit at 40 > 32, B′ = 20.5 from the last base
child's empty-remainder bound. Every recorded figure reproduces from
first principles. One unification the addendum missed: mechanisms 4
and 5 are the same mechanism — in BOTH the capped and merged cases the
custody anchor is the pivot of the partition group owning the improver
root (value ≤ d[root] < dis(z) by argmin over a P_j containing the
root), and row C's "root's own fresh entry" is the special case where
the root wins its group's argmin. Q2's empty-P_j drop cannot break the
capture: the anchor lives in the root's group, and a capped/merged
root is never in Q (only failed roots join `q_list`), so its group's
P_j is non-empty.

**Cold audit of boundary collapse — conclusion survives, two
corrections.** (i) confirmed as coded (line 33 inserts every S-member
with B′ ≤ d < B, live in row F's r). (ii) as stated ("an S-member with
d < B′ is in W and W′-settles, or was pulled") under-enumerates: an
S-member can also reach the boundary below B′ through pivot expansion
(line 13 seeds P_j members into the child's S without consuming any
entry), through the child's returned D (an unpulled child seed merges
back up), or through a post-last-pull re-selection (lines 27–29 run
before B′ is updated from the child's return). The honest dichotomy is
"settled, or entry-held in a live D at its current value" — every
branch of the case analysis lands in one of the two, so the collapse
conclusion is strengthened, not weakened. (iii) the separator cap is
correct but carries a TIMING QUALIFIER that addenda 11 and 12 both
stated too broadly: B′ ≤ b_last ≤ (stored values) holds only for
entries PRESENT IN D AT THE LAST PULL. Entries born after the last
pull — above all the last child's returned D, whose sub-B′ content is
exactly the Q3 leak, plus last-iteration re-selections — are not
capped, and row E's ward pair (z_t@20.84375, y_t@20.90625 under
B′ = 21) is the measured population of that hole.

**The composition — row G, the W′ leak.** The W′ filter (line 34 of
the paper, the port's boundary sweep) tests only W-membership,
non-settlement, and d[x] < B′; it never consults D. So the cap's
timing hole composes with pocket walking: take row E's graph and add
ONE seed f@995 with the single edge f →4→ y_t. f's FindPivots search
accepts the stale offer (999 < B = 1000), walks the sink y_t, and
fails at |K| = 2 < 3 — y_t enters W at 999. Everything else replays
row E bit-for-bit: the sixth child cutoff-exits right after planting
z_t@20.84375 (complete) and y_t@20.90625 (true 20.859375 via the
never-walked z_t) below its returned bound, B′ = 21. At the boundary
y_t ∈ W, y_t ∉ U, d[y_t] = 20.90625 < 21 — y_t passes the filter and
is SETTLED INCOMPLETE. Recorded (driver row G in
`examples/wprime.rs`, record `benchmarks/results/wprime.txt`, all
predictions written into the driver header before the first run and
confirmed exactly on it): paper and V8 identically B′ = 21, |U| = 40,
settle_incomplete = 1 — the direct (H) counter firing for the first
time on any input in the project's history — with viol 1/1, incomp
and wards 1/1/1, missing 0; V5: cap = 1, B′ = 20.84375, |U| = 39,
settle-inc = 0, everything else zero. Rows A–F reproduced
byte-identically in the same run; the full-recursion part stays
distance-exact on every engine (the falsification does not embed on
THIS graph: by the time f enters any call's S, y_t is settled complete
and the 999 offer is refused).

**What is falsified.** The crafted call state satisfies Lemma 3.7's
preconditions verbatim — every seed is complete, so ⟨∅, S⟩ is a
frontier for eU(B, S) by the paper's own definition (Section 2.5) —
hence this is a counterexample to the lemma AS STATED, not to an
unreachable configuration; no reachability argument is needed. Of
conclusion 2 ("max dis(U) < B′ ≤ dB[D], U = eU(B′, S) and is
complete"): the completeness clause fails (y_t ∈ U via W′,
incomplete), the bound clause fails (dB[D] = 20.84375 < 21, known
since Q3/Q15), and the set-equality clause fails (z_t ∈ eU(21, S) \ U
— which row E had already exhibited, unremarked, without the
completeness breach). The paper's own proof of "W′ is complete" (the
first bullet of Lemma 3.7's concluding argument) uses induction
proposition 2, dB[D ∪ P] ≥ B′ — the clause falsified live since Q3 —
so the proof breaks exactly where the counters said it would; row G
materializes the consequence in the conclusion. (H) — "every
settlement event settles a complete vertex" — is therefore FALSE for
the paper's algorithm as written, at direct-call grade.

**Consequences.**

- The H-proof program's target changes from (H) to (H | V5). V5's
  one-line base cap (B′ ≤ dB[D] at base returns, restoring Lemma 3.1's
  contract, transitively at every level per Q15 finding 3) removes the
  cap's timing hole: every merged entry is ≥ the returned bound, so
  the separator-cap argument needs no present-at-last-pull qualifier,
  and row G measures the repair live (the cap reads the escort
  z_t@20.84375 — the escort invariant and the V5 cap are the same
  object at the boundary). The anchor/window lemma is DEAD for the
  paper engine (counterexample) and becomes provable-shaped for V5.
- V8's distance-correctness is now suspect by derivation: in row G,
  V8 sets y_t's settled bit at the wrong label, and V8's relax
  rejects every later repair offer at a settled target — a
  full-recursion embedding (a value-adjacent f inside the leaking
  call's own S-batch, alignment to be engineered) is predicted to end
  with a WRONG FINAL DISTANCE on V8 while the paper engine heals
  through the duplicate channel. V8's green suites never contained
  the shape: it needs a stale pocket walk into the ward of a
  cutoff-exit leak. Queued as the next probe (row H) — until it runs,
  V8's exactness claim in VARIANTS.md must be treated as conditional
  on (H | V8-reachable states), which row G has made non-vacuous.
- The paper audit gains its sharpest finding: not a contract clause
  but the correctness lemma's completeness conclusion, falsified by a
  crafted frontier state satisfying the lemma's hypotheses, with a
  recorded reproducible run. Q3 → Q15 → row G is now one causal
  chain: the base-exit undercut, industrialized at scale, composed
  with the W′ filter's blindness to D.

Next session: cold re-derivation of THIS addendum's chain (the timing
hole, the frontier-precondition check, and the falsification logic are
same-session math — the probe's numbers are recorded but the
consequence analysis needs a fresh eye), then row H (the V8
wrong-distance embedding), then the anchor/window lemma restated for
V5.

## Q14 addendum 14 (2026-07-17, fresh session) — cold audit of addendum 13: the falsification chain holds end to end, the root sharpened to one unproven sentence in the paper, and two scope corrections

Item 0 of the cycle: cold re-derivation of addendum 13's same-session
math — the timing-hole derivation, the frontier-precondition check,
and the falsification logic — against the paper text and
`src/solver.rs`, with the recorded run re-executed. Verdict: the
falsification STANDS on every link; nothing in addendum 13's headline
needs retraction. The audit adds one sharpening and two scope notes.

**Reproduction.** `cargo run --release --features
verify-contract,dup-classify --example wprime` re-run cold:
byte-identical to `benchmarks/results/wprime.txt`, all rows, all
engines, direct calls and full recursion. Row G's numbers re-derived
by hand from the construction: six children (pairs of ballast pivots,
M = 2), separator b₆ = 21 fixed by the pull that extracts
{g10@20, g11@20.5} while g12@21 heads the remainder; the sixth base
child's ninth pull (u_last@20.8125, over its cutoff 8) empties the
base D so the pull's bound is the call bound 21, and u_last's
post-pull sweep then inserts z_t@20.84375 and y_t@20.90625 below it;
|U| = 5·6 + 9 = 39 plus y_t via W′ = 40; V5 caps to dB[D] = 20.84375
and keeps |U| = 39. Every figure matches the record.

**Timing hole — CONFIRMED against both texts.** The port's iteration
order (pull, child call, child-D merge, relax sweep, re-selection,
then B′ ← B′_i) matches Algorithm 3's line order, so on a limit exit
the returned D contains the last child's merged entries with no
separator above them. The cap's coverage re-derived: entries
surviving the last pull are ≥ b_f ≥ B′_f (the pull separator);
relax-sweep inserts carry the d ≥ b_i gate with b_f ≥ B′_f, so the
last iteration's line-24 inserts are capped too; earlier iterations'
inserts either got pulled (settled) or sit above the last separator.
The uncapped births are exactly addendum 13's channels — the last
child's returned D (the Q3 leak) and last-iteration re-selections —
and row G rides the first.

**Frontier precondition — CONFIRMED verbatim, one implicit
sub-hypothesis identified and checked.** Lemma 3.7 as printed assumes
only "⟨∅, S⟩ is a frontier for eU(B, S)". The frontier definition
(Section 2.4) additionally requires X, Y ⊆ eU — so every seed needs
dis(s) < B and a shortest path visiting S (itself). The tightest seed
is f: dis(f) = 995 < B = 1000 — holds. With every seed complete,
frontier condition 2 is automatic for all of eU. No size hypothesis
appears in 3.7; row G's |S| = 21 satisfies even Lemma 3.1's
t·2^{2lt} = 32. No hidden hypothesis: the counterexample meets the
lemma's stated preconditions exactly as claimed.

**Falsification logic — CONFIRMED, with the window clause noted.**
W is the union of FAILED searches' arborescences in the paper's own
Algorithm 2 (lines 17–19), matching the port's `w_list`; y_t enters
W_f under both. The W′ formula is the paper's (proof of Lemma 3.7,
first bullet: W′ = {x ∈ W \ U : d[x] < B′}) — the D-blindness is in
the paper, not a port artifact. The three failed clauses of
conclusion 2 re-instantiated by hand: completeness (y_t settled at
20.90625 ≠ dis 20.859375), bound (dB[D] = 20.84375 < 21), set
equality (z_t ∈ eU(21, S) \ U). One clause SURVIVES and is worth
recording: max dis(U) < B′ holds (dis(y_t) = 20.859375 < 21) — the
falsification settles a wrong LABEL inside the window, it does not
settle a vertex outside the window. Any repaired lemma keeps the
window clause as is.

**The sharpening — one root sentence.** Addendum 13 located the proof
break at induction proposition 2. The audit pushes one level deeper:
proposition 2's inductive step ("any v added into D has d[v] ≥ B′_i")
is sound GIVEN the children honor Lemma 3.1's dB[D] ≥ B′ clause, and
the recursion bottoms out at the single unproven sentence "Clearly
base case satisfies Lemma 3.7." That sentence is false: Algorithm 4
fixes B′ at the pull (line 4) and inserts relaxed neighbors into D
afterwards (lines 6–8), so a cutoff exit returns sub-B′ entries — Q3,
confirmed 2026-07-13. Everything else in Lemma 3.7's proof audited
locally valid. The paper therefore has ONE root defect with two
symptom families: the bound clause violated at scale on natural
weights (Q15), and the completeness clause falsified by a crafted
frontier (row G). This is the sharpest possible framing for the
eventual write-up.

**Scope note 1 — the boundary-collapse dichotomy is (H | V5)-scoped.**
Addendum 13's corrected dichotomy ("an S-member at the boundary below
B′ is settled, or entry-held in a live D") leans on W′ completeness
in its W-channel branch — the very clause row G falsifies — so on the
paper engine it inherits the counterexample and is dead with (H). Its
proper home is the V5 program, where the cap restores W′ completeness
and every branch of the case analysis closes. The anchor/window lemma
should cite it only under V5.

**Scope note 2 — the relay channel, flagged for the ward inventory.**
An incomplete W′-settled vertex relays its wrong label onward: the
final sweep (paper line 37, the port's W′ relax) relaxes its
out-edges at the wrong value, and a successor landing below B′ is
refused a D entry by the d ≥ B′ insert gate — wrong-labeled,
unsettled, entry-less, invisible to W and to the escort wards (wards
key on entries). Row G does not fire it only because y_t is a sink.
On the paper engine the duplicate channel still heals it (z_t's entry
re-settles y_t, whose strict improvement re-relaxes the successor);
under V5 the channel closes with W′ completeness itself. Recorded so
the dichotomy is never quoted engine-unconditionally, and as a
candidate ingredient for row H's embedding: it is a second way to
hide a wrong value from repair, orthogonal to V8's settled bit.

Audit verdict: addendum 13 confirmed in full — (H) is false for the
paper engine and V8 at direct-call grade, the target stays (H | V5),
and the causal chain Q3 → Q15 → row G now bottoms out in Algorithm
4's pull-then-insert order and the one "Clearly" that papered over
it. Next: row H (the V8 wrong-distance embedding — f value-adjacent
inside the leaking call's S-batch, chain re-alignment so the cutoff
still fires at u_last), then the anchor/window lemma under V5.

## Q14 addendum 15 (2026-07-17, fresh session) — row H: the W′ leak embedded in a full recursion; V8 finishes with a wrong final distance (the first distance-incorrect V8 run on any input), the paper engine heals through one duplicate settlement, V5 closes the hole — the three engines separate on a single 70-vertex input

Attack-plan spine item: row H, the V8 wrong-distance embedding
predicted by addendum 13 ("V8's distance-correctness is now suspect by
derivation") and queued by addendum 14.

**Why row G does not embed, and the fix.** The falsification needs
y_t ∈ W of the very call whose exit strands the leaked entries, and W
membership is decided at FindPivots time — so the stale-walking seed f
must be a member of that call's S, walking y_t while it is still
unlabeled. In row G's full run f sits at 995: by the time any call
pulls it, y_t is settled complete and the walk is refused. Worse, the
first level-1 call of any full recursion has S = {0}, and a single
search from the source caps immediately on the seed fan-out (W = ∅
always), so the leak can never fire in the call that does the early
work. The first call with a real multi-seed S is the level-2 call's
SECOND pull — row H makes f value-adjacent and packs the entire
leaking configuration into that one batch.

**Construction** (`examples/rowh.rs`, one graph, source 0, n = 70,
t = 2, k = 3, l_top = 4; record `benchmarks/results/rowh.txt`):

- Phase A: 32 sink seeds at 1.0..16.5. The first level-1 call
  (S = {0}, limit 32) consumes exactly these — base child 1 settles
  0 + a1..a8 (cutoff exit at separator 5), twelve sink pairs take |U|
  to 33 > 32 — and limit-exits at B′ = 20 with every later key
  unsettled in the returned D.
- The batch: the level-2 call's second pull (M = 8) extracts exactly
  {p1..p6, g11, f} — six 3-leaf ballast stars at 20..22.5, the chain
  root g11@23, f@23.75. The 9th-smallest key, a sink at 30, becomes
  the pull separator: B_C* = 30.
- Inside C* (level 1, B = 30): f's search accepts the stale offer
  f →4→ y_t at 27.75 (y_t unlabeled — the chain is never walked
  before this call) and fails at |K| = 2 < 3: W = {f, y_t}. Three
  star-pair children settle 24; then g11 is pulled ALONE, the chain
  child walks g11 + h1..h7 + u_last = 9 pulls, the 9th empties the
  base D fixing the bound, and u_last's post-pull sweep plants
  z_t@23.53125 and y_t@23.59375 (true 23.546875 via the never-walked
  z_t) below it. |U| = 33 > 32: limit exit, B′ = 30, and y_t passes
  the W′ filter — settled INCOMPLETE inside a full recursion.
- Aftermath: C*'s returned D carries z_t and y_t up; the level-2
  call's next pull hands them to a cleanup call whose FindPivots
  search from z_t repairs y_t — unless something blocks the relax.

Three alignment facts the construction turned up (they constrain any
future embedding of this shape):

1. A failed root holds no D entry, so an 8-key batch yields at most 7
   pivots and the chain root is pulled alone — and a solo pull's
   empty remainder makes the base child's bound the CALL bound B_C*
   itself. The recursion hands the widest possible leak window (B′ =
   B_C* = 30) for free; the direct rows' careful separator
   engineering (b₆ = 21) is not needed at all.
2. The plan's chain re-alignment: 8 chain steps so that g11 + h1..h7
   + u_last = 9 pulls with u_last exactly ninth (cutoff 8) — the base
   D must be EMPTY at the ninth pull so the bound is fixed before the
   sweep inserts the leak below it (Q3's pull-then-insert order, the
   root defect of addendum 14, doing the work).
3. The stale value threads dis(y_t) < d_direct(y_t) < σ < B_C*
   (23.546875 < 23.59375 < 27.75 < 30): below B_C* so the FindPivots
   walk is accepted, above the direct value so the chain child's
   insert is the operative wrong label. Hand alignment sufficed; the
   grid-search fallback the plan reserved was not needed.

**Recorded run** (`benchmarks/results/rowh.txt`, all predictions
written into the driver header before the first run and confirmed
exactly on it, every figure):

- paper: settle-inc = 1 (y_t at C*'s boundary), viol base/rec = 1/1,
  incomp b/r/i = 1/1/2, wards 1/1/2, missing 0, dups = 1 — y_t is
  settled TWICE (incomplete at 23.59375 in C*, complete at 23.546875
  in the cleanup call's W′) and the final distances are EXACT. The
  duplicate channel's healing is now measured in vivo: exactly one
  duplicate settlement (wprime_own_root) is the entire repair — on
  this input Q11's duplicate is not overhead, it IS the correctness.
- V8: identical counters (the engines coincide bit-for-bit up to the
  W′ settlement, and every counter site at or after it sees identical
  D states), dups = 0, y_t settled ONCE — the settled bit rejects
  z_t's repair relax and the level-2 pull drops y_t's stale entry:
  final d[y_t] = 23.59375 against true 23.546875 (err 0.046875), the
  only wrong vertex of 70. The first distance-incorrect V8 run on any
  input, exactly as derived.
- V5: cap = 1 (the chain child's bound capped to dB[D] = 23.53125,
  the escort z_t again), settle-inc = 0, all other counters zero,
  distances exact — y_t never passes the W′ filter, f is re-inserted
  by line 33, the cleanup call settles everything complete.

**What this settles.**

- (H | V8-reachable states) — the conditional addendum 13 made
  non-vacuous — is FALSE: the completeness hypothesis fails on a
  V8-reachable state and V8 freezes the wrong label its contrapositive
  paragraph predicted. VARIANTS.md's V8 exactness claim amended in
  this commit: exact on the tested matrix, incorrect in general; V8
  not promotable to mainline; V5+V8 is the candidate composition and
  its exactness is precisely the (H | V5) program.
- The Tier-2 matrix's green V8 gate is explained, not contradicted:
  the shape needs a stale pocket walk into the ward of a base-case
  cutoff-exit leak, and no generator family contains it. A crafted
  counterexample is the only way this red gate could ever have been
  produced — which is why addendum 13's derivation, not the matrix,
  found it.
- The three engines now separate on one 70-vertex input: paper =
  incomplete settlement + duplicate heal (exact), V8 = incomplete
  settlement + frozen wrong distance, V5 = no incomplete settlement
  at all. Keeper regression probe for any future engine work.
- The relay channel (addendum 14 scope note 2) stays dormant by
  design (y_t is a sink): row H needed only the settled bit as the
  hiding mechanism, so the relay remains an unexercised second
  channel, still available if a paper-engine wrong-distance embedding
  is ever wanted (it would need the healing duplicate itself blocked,
  which the relay's entry-less successor provides).

Next: the anchor/window lemma restated under V5 — (H | V5) is now the
only living exactness target above tested-matrix grade, for the paper
engine and V8 alike.

## Q14 addendum 16 (2026-07-18, fresh session; the ENTIRE recast is same-session math, flagged for cold re-derivation before anything builds on it) — the anchor/window lemma under V5: the window half proved from the pull invariant plus the V5 contract (modulo Q12, the single carve-out), the anchor weakened to off-path custody, (H | V5) assembled at sketch grade over the audited channel inventory — and E-c's mid-iteration half drops out of the base argument

The attack-plan item queued by addenda 13–15. One infrastructure note
first: development moved to an arm64 machine this session, and the
recorded corpus survives the platform change — `wprime` and `rowh`
re-run cold reproduce `benchmarks/results/wprime.txt` and
`benchmarks/results/rowh.txt` byte-identically (modulo line endings),
every row, every engine, direct calls and full recursion. The port's
determinism is architecture-independent as designed (IEEE-754 ops
only; BlockDs reads order only through the value-sorted pull;
keep-min is insertion-order-free), and every number quoted below
traces to those records unchanged.

Scope: the V5 engine (`fixed_base`), len-completeness grade (Lemma N
standing; the tuple-grade upgrade is a named debt). Target: (H | V5)
— every settlement event (base pull; W′ boundary append) settles a
len-complete vertex. Definitions pinned from addenda 10–12: *complete*
= label len equals dis; *live D* = the ds of each call on the active
stack plus the current base D (returned Ds merge synchronously at the
child-return line, so no other D ever exists); *governing bound* β of
an event = the pulled entry's stored value at a base pull, B′ at a W′
append. Stored values never sit below their key's current label
(labels only improve; keep-min keeps the minimum of historical
inserts, each of which was the label at its own insert time), so
"stale" always means stored-above-label, never below.

**Lemma W (the window).** At a settlement event with governing bound
β, no live D holds a stored value < β — modulo exactly one carve-out,
Q12, named in (W2).

- (W1) *Ancestors.* Every stored value of a call's ds is < its block
  bound B: each insert site is fed by a relax gated at cand < B (base
  post-pull inserts, both sweeps), pivot and line-33 inserts store
  labels < B by their own guards, and merges carry child values
  < B_child ≤ B. Hence every pull separator is ≤ B, hence call bounds
  nest downward. While a child subtree is active, the parent's ds is
  untouched between the pull and the child call (no insert site
  executes in that span), so dB[parent ds] equals the pull remainder's
  minimum, which is ≥ b_i = B_child by the pull invariant (the
  returned separator IS the remainder's minimum). Composing up the
  active stack: every strict ancestor A of the event's call C holds
  dB[A.ds] ≥ B_C ≥ β.
- (W2) *Own D at a W′ append.* At the filter, ds holds the loop-exit
  content plus the line-33 re-inserts. Line-33 inserts are ≥ B′ by
  their guard. The loop-exit content is ≥ B′ by the transitive
  proposition (VARIANTS.md "Algorithm 3's contract, transitively")
  with V5 supplying the base clause at every leaf — modulo its one
  hole, Q12's final-iteration line-29 re-selection. Under V5 that
  channel has never fired (viol_bound_rec = 0 on every family
  including escape, `benchmarks/results/escape.txt`), but it is open
  as statement-level math, so it is carried as the proof's one
  carve-out: if a Q12 state occurred, W2 would fail and the proof
  would go silent there (no counterexample is implied — the
  contradiction below simply would not fire).
- (W3) *Own D at a base pull.* The pull pops the value-order global
  minimum, so nothing co-resident is below β by definition.

**Lemma A′ (the anchor, weakened to off-path custody).** At every
settlement event, for every vertex y with an incomplete label and
every tight path to y, with z the path's last complete vertex (the
source is complete, so z exists): (i) some live D holds a stored
value whose len is ≤ dis(z), or (ii) z has been settled, or (iii) z
was popped at its final label by a FindPivots search of the event's
own call. The weakening against addendum 12's statement is the point:
custody need not sit on y's path, need not name z, and need not be
fresh — ANY live stored value at or below dis(z) suffices, because
A′'s only consumer is the contradiction against Lemma W. The proxy
complication (row F) and the deep-pocket residue (channel 4 below)
are absorbed by the statement instead of case-splitting it, and
freshness — which the escort form (E) had to carry — drops out
entirely.

**Reduction: (A′) + (W) ⟹ (H | V5).** Induct on settlement events in
program order; suppose the first violation settles y incomplete under
β. The strictness chain: dis(z) ≤ dis(y) < len(d[y]) ≤ len(β), strict
at the incompleteness step. Now each A′ branch contradicts:

- (i) gives a live stored value at len ≤ dis(z) < len(β) — strictly
  below β, impossible by W (W3/W1 at base pulls, W2/W1 at appends).
- (iii): the pop relaxed the tight edge (z, x₁) at
  cand = the canonical tuple of x₁, with len(cand) = dis(x₁) ≤
  dis(y) < len(β) ≤ len(B_C), so the offer cleared the relax bound;
  x₁ incomplete at that moment means its label exceeded cand, so the
  offer was accepted and x₁ completed during FindPivots — before the
  event — contradicting z's last-complete maximality at the event.
- (ii): z settled complete (an incomplete settlement of z would be an
  earlier violation). z rides the concatenated returned U past every
  boundary above its settling call, and each level's relax sweep
  re-offers (z, x₁) at that level's call bound. Walk up to the
  deepest ACTIVE call A that received z in a finished iteration
  (z's settling call itself if that is C in an earlier iteration or
  an earlier singleton of the same base loop): the sweep there ran
  strictly before the current event at bound B_A ≥ B_C > len(cand),
  the offer was accepted against x₁'s incomplete label, and x₁
  completed — same contradiction. The one same-time corner, z
  settling in the current W′ batch itself, never reaches (ii): a
  batch member is in W, W-membership at a final label is exactly
  case (iii) (a failed search's heap empties, so everything walked
  was popped), and W-membership at a non-final label followed by
  later completion lands in the channel inventory with custody (i).

Contradiction in every branch; no first violation exists. ∎ (modulo
Q12 via W2)

**The channel inventory behind A′** — every way a vertex completes,
and where its custody lands (the silent-completion inventory of
addenda 11–13, exhaustive on the port's four relax call sites and
three search exits, re-run under V5):

1. *Base post-pull sweep from a settled pred*: the insert is ungated
   (Q3's channel) — custody (i) lands in the base D and merges up.
   The proof consumes the ungated insert POSITIVELY: V5 keeps it and
   caps only the returned bound, which is exactly why the leak that
   killed (H) on the paper engine becomes custody under V5.
2. *Iteration relax sweep from a settled pred*, gate d ≥ b_i: pass →
   custody (i). Refusal means the same offer already cleared the
   child's own bound (complementary gates, mechanism 3) — descend
   into the child that returned the pred; the descent terminates at a
   base insert (1), a FindPivots walk (4/5), or a W′ sweep (3).
3. *W′ final sweep from a settled pred*, gate d ≥ B′: pass → custody
   (i). Refusal (cand < B′) is the row-G site, closed under V5: the
   sweeping source is in W′, so either it was popped at its final
   label — then this edge was already relaxed during the search and
   the target is in W, settling complete in the same batch — or its
   completion came from a capped/merged improver whose proxy custody
   (5) is a live entry below B′, impossible at this boundary by W2.
4. *Failed-search walk* (popped at final label): case (iii) in the
   walking call; at that call's boundary the vertex settles
   (d < B′) or joins the W-residue (d ≥ B′). Residue custody: the
   pocket root f ∈ S is re-entried by line 33 (B′ ≤ d[f] < B) at
   dis(f) ≤ dis(z) — custody (i); if instead d[f] < B′, f settled
   and (ii)'s lattice pushes completeness and gated-pass entries up
   the pocket; independently, the W′ sweep's exact-tie re-offer
   re-entries the first residue layer through the d ≥ B′ gate
   (addendum 12's sharpening). Some live stored value ≤ dis(z)
   survives in every branch — rows A/B/E's recorded mechanics.
5. *Capped or merged improver*: the improver root's partition group
   holds a pivot entry at argmin ≤ d[root] < dis(z) (mechanisms 4/5
   unified, addendum 13) — proxy custody (i). Consumption of any
   custody entry by a pull turns the anchor into a child seed
   (pivot expansion for proxies), and the boundary collapse
   (addendum 13's corrected dichotomy, V5-scoped per addendum 14)
   re-materializes custody at every return: settled, or entry-held
   via line 33 / child-D merge / re-selection — each ≥ B′ under V5
   or an earlier settlement. Prefix containment covers pulls that
   take the anchor without the ward (a smaller value is always in
   the same or an earlier pull prefix).

*Seeds need no separate hypothesis*: a call's seeds may be
incomplete — an incomplete seed's tight path carries its anchor
in the same ds at a strictly smaller stored value, so the anchor is
pulled in the same or an earlier batch and the induction recurses;
the top call's S = {source} is complete outright.

**What dropped out.** Addendum 10's base-pull argument consumed E-c's
mid-iteration half to exclude a stale pulled entry (the escort
pairing needed the ward's entry fresh). The recast never touches
freshness: the settlement reads the vertex's CURRENT label, β is the
STORED value (≥ the label, so incompleteness of the label transfers),
and A′'s custody is any stored value at all. E-c stays true as
measured (stale-inc zero everywhere) and stays part of the invariant
picture, but it is no longer a named debt of (H | V5). The cold audit
should attack this elimination first — it is the recast's largest
single claim.

**The attempted falsification (the row-I shape that does not exist).**
Before writing the lemma down, the strongest available attack was run
on paper: aim addendum 14's relay channel at V5 itself — let a
W′-settled x's sub-B′ tight successor v complete entry-less at the
final sweep (gate-refused, custody-less), give v's successor v₁ a
stale side path, and let a later call's failed search walk v₁ into W
below a later B′: v₁ would settle incomplete on every engine,
falsifying (H | V5) and, since v never re-relaxes, ending the FULL
run with a wrong distance on the paper engine too. The assembly dies
at the first hop, and the blocker is pocket closure (mechanism 1): x
∈ W requires a FAILED search, a failed search's heap empties, so x
was popped — at a final label the edge (x, v) was relaxed THEN
(v joins W and settles complete in the same batch; or cand ≥ B_C
puts dis(v) outside every window of the call; or v was already
complete), and at a non-final label x's completion needs an improver
whose proxy custody contradicts W2 at the boundary. Every branch
closes without a new driver — the branch behaviors are rows A, B, F,
G's recorded mechanics. The W′ sweep's insert gate — the one site
that refuses sub-B′ entries by design — cannot orphan a completion
under V5: the refused target is always already in W or already
complete. On the paper engine the same assembly is NOT blocked (the
first hop rides the Q3 leak instead of a pocket, row G), which is one
more way of stating that the falsification family and the V5 repair
are the same object.

**Status and debts.** (H | V5) now rests on: Lemma W (proved above,
modulo Q12), Lemma A′ (proved over the channel inventory at sketch
grade; the inventory's exhaustiveness is addendum 12's audited
relabel-site enumeration), and the reduction (proved above). Named
debts after this addendum: (a) the cold re-derivation of the entire
recast — specific attack points: the E-c elimination, the reduction's
same-batch corner, channel 2's descent termination, the seed
recursion, and whether Q12's carve-out stays confined to W2; (b) Q12
itself (measured zero under V5, open as mathematics — the only
statement-level gap between this sketch and a theorem); (c) the
tuple-grade upgrade of the completeness notion (Lemma N). If (a)
holds, the (H | V5) program is done modulo (b) and (c), and with it
V5+V8's exactness claim and the repaired-lemma write-up gain their
missing spine.

## Q14 addendum 17 (2026-07-18, fresh session) — cold audit of addendum 16: the recast holds on all five named targets; two wording defects repaired, the Q12 carve-out sharpened to its transitive form, and the one residual named — custody persistence as its own lemma

Debt (a) of addendum 16, per the standing cadence rule. Every step
re-derived against `src/solver.rs` (the four relax call sites,
`BlockDs::pull`/`insert`/`merge`, the V5 cap) and the recorded
definitions — none from memory. Verdict: the window/off-path-anchor
recast SURVIVES. The five named attack points in order:

**1. The E-c elimination — CONFIRMED (the recast's largest claim).**
The definitional spine first: stored values never sit below current
labels — every insert site stores `self.d[v]` at insert time (base
seeds, base post-relax, initial and re-selected pivots, line 33, both
sweeps; merges carry such values), keep-min keeps the minimum of
historical labels, and labels only improve, so the current label ≤
every historical insert ≤ the stored minimum. Incompleteness of the
label therefore transfers to β through the len slot of tuple order
(lex order, len first). Both event judgments read current labels only
(the base pull settles d[u]; the W′ filter reads d[x]); A′'s custody
is any stored value; and no channel of the inventory invokes
freshness anywhere — checked channel by channel. E-c stays true as
measured and stays part of the invariant picture, but nothing in
(H | V5) consumes it. The elimination is real.

**2. The same-batch corner — CONFIRMED**, and the case analysis
re-derives sharper than stated. (a) One child call per loop iteration
makes "z arrived in an active ancestor's CURRENT iteration"
structurally impossible: the current iteration's child is still
active, so nothing has returned into that iteration — every active
ancestor that received z did so in a finished iteration whose sweep
ran strictly before the event. (b) An ACTIVE ancestor cannot have
W′-settled z: the append is post-loop, and a call with a live
descendant is still in its loop — so a settlement of z inside an
active call is a base pull of the event's own call (the base-sweep
case) and nothing else. (c) Branch (iii)'s tie-refusal corner closes
at len grade: the offer to an incomplete x₁ is strict in the len slot
(len(cand) = dis(x₁) < len(d[x₁])), so a tuple-order refusal at equal
length cannot occur — this line is the known pressure point of the
tuple-grade upgrade (debt c), flagged for that session.

**3. Channel 2's descent termination — CONFIRMED.** Each descent step
moves to a strictly deeper FINISHED call (z settled in the returned
child's subtree; recurse to z's actual settling site), the call tree
is finite, and every terminal is an enumerated channel: base pull
(ungated insert — custody), FindPivots exits (channels 4/5), W′ sweep
pass (custody) or refusal (channel 3, whose branches end in same-batch
completion or the W2 contradiction). The relax-site enumeration behind
exhaustiveness re-verified against the code: exactly four call sites
(search, base, iteration sweep, W′ sweep). No circularity: the descent
consults only finished deeper calls and same-call structural facts,
never the event's own A′ instance.

**4. The seed recursion — HOLDS, but the written step skips its
justification; repaired here.** "Carries its anchor in the same ds" is
not free — custody could a priori sit in any live D. It is forced
into the pulling call's own ds by the ancestor half of Lemma W applied
at the pull moment: W as stated is event-scoped, but the W1 proof is
moment-independent (the no-insert span between pull and child call and
the pull invariant hold whenever a child subtree is active), so state
it as **W1's ancestor floor holds at every moment a call is
suspended**. A sub-pull-value custody entry then cannot be an
ancestor's, hence it is co-resident with the ward's entry, hence
prefix containment (strictly smaller in the len slot: len ≤ dis(z) <
len of the ward's stored value) pulls it with or before the ward.
Through a RECURSIVE child the anchor arrives as a seed and is
represented per FindPivots (pivot / W / proxy — channels 4/5), and the
recursion bottoms at the base case, where every seed enters the base D
directly. Second wording defect, channel 4: "re-entried by line 33 at
dis(f) ≤ dis(z)" presumes f complete, which f need not be; the correct
claim is **d[f] ≤ dis(z)**, by heap-pop monotonicity of the failed
search (every in-search label ≥ the root's start label — each pushed
candidate strictly exceeds its popped source — so dis(z) = z's popped
final label ≥ d[f] at search start ≥ d[f] at line 33, labels only
improving). Custody (i) needs only the stored len ≤ dis(z), which this
gives; the conclusion stands, the wording is repaired.

**5. Q12's confinement to W2 — CONFIRMED, carve-out sharpened to its
transitive form.** W1 is immune to Q12 by construction: the pull
invariant is insert-history-agnostic — a sub-b_i re-selection survivor
from an earlier iteration would have been consumed by the very pull
that defined b_i, so at child-call time the remainder minimum is
≥ b_i regardless of what line 29 ever inserted. W3 is immune (the base
D has no re-selection site). The single entry point is W2's loop-exit
clause via the transitive proposition — but the carve-out as written
names only the event call's OWN final-iteration re-selection, and the
inherited shape exists: a descendant's final-iteration line-29 leak
rides the child-D merge into an ancestor whose loop exits WITHOUT
another pull (limit exit in the same iteration), surfacing at that
ancestor's W′ filter unconsumed. Same single hole — it still enters
only through W2, via the transitive proposition's induction
hypothesis — but the carve-out must read "a final-iteration line-29
re-selection anywhere in the event call's subtree, unconsumed by any
later pull", not "the event's own". Measured status unchanged
(viol_bound_rec = 0 corpus-wide under V5).

**The residual, named.** The audit's one refusal to grade up: custody
SURVIVAL between z's completion and the settlement event is
distributed across channel 5's consumption/re-materialization
sentences (proxy → pulled pivot → child seed → boundary collapse →
line-33/merge/re-selection re-entry) rather than stated as one
induction. Every branch attacked here closed — consumption by a pull
turns custody into a seed whose subtree settles the anchor or
re-materializes an entry at its boundary; keep-min can only lower a
custody value, preserving len ≤ dis(z); keys leave a ds only by pull —
and no counterexample was found. But the write-up needs this as its
own lemma (**custody persistence**: once created, custody is
re-established at every event boundary until the anchored vertex
settles or the path's frontier advances), with the event interleaving
spelled out. The grade stays sketch until that lemma is written down.

**Status.** Addendum 16's debt (a) is discharged: the recast holds
under cold re-derivation, with two wording repairs (channel 4's
d[f]; the seed step's W1-at-any-moment justification) and the
transitive Q12 carve-out folded into the record. (H | V5) rests on
W + A′ + the reduction with named debts: (b) Q12 as mathematics — now
formally the last statement-level gap, carve-out in transitive form;
(c) the tuple-grade upgrade (Lemma N), pressure point = branch (iii)'s
tie strictness; (d) the custody-persistence lemma as the write-up's
spine. Next: Q12.

## Q15 — the wave-escape family: Lemma 3.7's bound clause violated at scale on plain positive weights; the Q3 leak is an accidental wave damper; V5 repairs every counter at once. CONFIRMED (recorded run)

Built to decide Q14 addendum 4's unproved item (i): force a
re-settlement wave longer than the child budget of the call holding its
trigger, and watch whether B′ regresses. Construction (driver
`examples/escape.rs`, raw record `benchmarks/results/escape.txt`): a
settled unit-weight path p_1..p_L, a phase-A block of leaf values that
drags the climbing leftover D to a mid-level call before the single
decoy trigger (planted at value A above phase A while d[p_1] = ∞, with
filler labels closing the decoy's base chunk so the plant survives
keyed dedup) is pulled, and a phase-B block above. All runs
distance-exact vs Dijkstra; all under the event ceiling. Five results:

1. **B′ monotonicity survives even here — zero regressions in every
   configuration.** The escape valve is not the progress measure but
   the bound clause: the wave-carrying child keeps its high separator
   as returned B′ and leaks the low front through the returned D.

2. **Lemma 3.7's clause 2 (dB[D] ≥ B′) is violated at recursive
   returns, in the wild, for the first time** — 366 violations at
   L = 5 000, 988 at L = 20 000 — and Lemma 3.1's base-case clause in
   the thousands (1 110 / 2 973), on a family with plain positive unit
   weights: no zero-weight edges, no exact ties, no crafted stars. The
   mechanism is Q3's undercut industrialized: a wave-carrying base
   chunk exits at its |U| cutoff right after the post-pull relaxation
   re-inserts the next front below the ballast-scale separator — one
   violation per chunk, confirmed by arithmetic: V8's counts are
   exactly ⌈wave/(t³+1)⌉ (555 = 5 000/9 at t = 2), the paper engine
   doubles them (first wave + duplicate wave). [Cold-audit correction
   2026-07-16: the formula is ⌊wave/(t³+1)⌋ — ⌈·⌉ overcounts the
   boundary chunk (555 = ⌊5 000/9⌋; V8 at L = 20 000 is
   2 222 = ⌊20 000/9⌋) — and the doubling holds only where the
   duplicate wave completes: at L = 5 000, 1 110 = 2·555, but at
   L = 20 000 the stranded wave (finding 4's damping) gives
   2 973 < 4 444, while V5's completed wave shows cap_hits
   4 444 = 2·2 222 exactly.] The full-execution
   dichotomy breaks at the same scale (up to 2 958 — Q10's sub-case,
   previously a corner). Q3's addendum "the adversarial family is
   contract-clean" is hereby superseded: cleanliness was a property of
   those inputs, not of the algorithm.

3. **V5's one-line cap zeroes all three counters simultaneously** —
   base, recursive, and dichotomy violations all drop to zero, with
   cap_hits exactly equal to the paper's base-violation count
   (44 / 1 110 / 618 / 4 444). This is experimental confirmation of
   the transitive contract restoration (VARIANTS.md "Algorithm 3's
   contract, transitively"): repairing the leaves repairs every
   recursive return above them. The recursive violations were
   inherited Q3, not the line-29 channel — Q12 proper remains
   unobserved even under deliberate escape pressure.

4. **The Q3 leak is an accidental wave damper.** Under V5 the waves
   roll FARTHER: 133 → 565 duplicates in the early-trigger row,
   6 757 → 20 000 (the full path, every vertex re-settled) at
   L = 20 000. The unrepaired algorithm's contract violation was
   killing its own re-settlement waves — the parent's healing pull
   consumes the leaked front out of order and strands the wave. So
   the two defects are load-bearing on each other: fixing the Lemma
   3.1 contract (V5) amplifies the Lemma 3.8 accounting excess on
   escape geometry. On the cascade family (Q14 addendum 3) V5 ≈ paper
   (313 vs 297 at L = 50; identical at natural parameters) — there
   the damping is trigger batch-merging, not the leak. Only V8 closes
   both defects at once.

5. **Batch-coarsening is a negative feedback that self-damps
   multi-trigger escapes** (analysis, not yet a theorem): every wave's
   events raise the cumulative count, which raises the level the
   leftover D climbs to, which coarsens the pull batches
   (M = t·2^((l−1)t)), which raises the rank-separation cost of the
   NEXT trigger linearly in consumed events. Staggered-trigger designs
   pay ballast quadratic in the wave count and their dup/n decays.
   This is the sharpest known damping mechanism and the refined
   branch-(b) proof target: bound the number of rank-separated stale
   pulls a run can afford.

Consequence for Lemma F (Q14 addendum 4) — CORRECTED same day after
measuring V5's own regressions (the first draft of this paragraph
claimed V5 satisfies both hypotheses; it does not): the escape must
break exactly one of Lemma F's two hypotheses, and WHICH one depends
on the engine. The paper engine keeps B′ monotone and leaks through
the bound clause; V5 keeps the bound clause and REGRESSES B′ — the
honest cap hands the parent the front's small value, with recorded
empty-interval witnesses (level 6, B′ 22059 → 9227 while |U_i| = 9216:
the containment {dis(U_i)} ⊆ [B′_{i−1}, B′_i) of Lemma 3.8's proof
instantiated on an empty interval with nine thousand events). V8
breaks neither. A conservation of trouble: on escape inputs the
trouble is not removable by return-boundary bookkeeping — it moves
between the two clauses — and only removing the re-settlements
themselves (the settled bit) removes it. The missing re-settlement
lemma is therefore hardest precisely where the contracts are honest.

## Q7 — nEdges overflow bound. NOTE

nEdges strictly increases along relaxations and is bounded by the longest
path in the reduced graph (< n₂). i32 suffices below n₂ = 2³¹ (same
assert as the first project's Label packing); the zero-weight cycles make
nEdges of practical labels larger than hop counts of the original graph
(up to +|C_v| detours), still < n₂.
