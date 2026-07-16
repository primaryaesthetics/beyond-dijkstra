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
   doubles them (first wave + duplicate wave). The full-execution
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
