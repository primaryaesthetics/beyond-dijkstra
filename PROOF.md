# The completeness lemma for the V5 engine

This document consolidates the proof of the settlement-completeness
property (H) for the V5 variant of our port of arXiv:2602.07868 — the
result that the Q14 research program (QUESTIONS.md) was opened to decide.
The program's history, its falsifications, and the session-by-session
derivation trail live in QUESTIONS.md Q14 and its addenda 1–26; this
document is the linear write-up of the surviving chain, self-contained
modulo SPEC.md (the pinned semantics) and VARIANTS.md (the engine
definitions).

**Statement proved.** On the V5 engine (`fixed_base`, VARIANTS.md), at
length grade, every settlement event — a base-case pull or a W′ boundary
append — settles a vertex whose label length equals its true shortest-path
distance.

**What it is not.** This is a careful hand proof against a specific,
frozen implementation, not a machine-checked proof and not a proof about
the paper's algorithm as written — (H) is *false* for the paper's
algorithm and for the V8 variant, by recorded counterexample (§10). Each
step of the chain was derived in one working session and re-derived from
the code in a separate, later session before anything was built on it;
the audit trail is QUESTIONS.md addenda 16–25 (five derivation/audit
pairs). The one remaining grade limitation is §11's tuple-grade gap.

Every number in this document traces to a file under `benchmarks/results/`.
Line references are to `src/solver.rs` at the commit that introduces this
document.

---

## 1. Objects and notation

The solver is the BMSSP recursion of arXiv:2602.07868 as pinned by
SPEC.md: labels are 4-tuples ⟨len, hops, curr, pred⟩ compared
lexicographically (a total order; `src/label.rs`), `d[v]` is v's current
label, and a call `bmssp(B, S, level)` receives a bound B and seed set S
and returns `(B′, U, D)` — a bound B′ ≤ B, the settlement log U, and its
leftover data structure D. Base calls (Algorithm 4) do the same with a
Dijkstra-style loop. Notation:

- **dis(v)** — the true shortest-path length to v (at length grade we
  compare lengths; where a tuple inequality is needed it is derived
  explicitly, see §11).
- **complete** — `len(d[v]) = dis(v)`. Labels never sit below true
  distance, so incomplete means strictly above.
- **settlement event** — a base-case pull (solver.rs:1048–1055) or a W′
  boundary append (the filter at 1346–1354). These are exactly the two
  sites the settlement log records (1055 via `u_log.push`, 1450 via the
  W′ extension), and the two sites the `settle_incomplete` counter
  checks (1057–1062, 1360–1370).
- **(H)** — no settlement event settles an incomplete vertex.
- **live D** — the `ds` of each call on the active stack plus the current
  base D. Returned Ds merge synchronously at the child-return line
  (1212), so no other D exists.
- **B, b_i, B′** — a call's block bound; the separator returned by the
  i-th pull; the call's returned bound. B′ is overwritten with each
  child's returned bound (1317), so at exit it is the final child's.
- **W, W′** — the failed-search residue of FindPivots (976–987), and the
  subset of W that the boundary filter settles: unsettled members with
  `d < B′` (1346–1354).
- **V5** (`fixed_base`) — the one behavioral change: the base case caps
  its returned bound at the minimum value left in its D (1097–1105). It
  never touches the D itself.

Two standing facts used throughout, both structural:

- **(E) Stored values never sit below labels.** Every insert site stores
  the vertex's current label at insert time; keep-min (300–316) keeps
  the minimum of historical labels; labels only improve. So an entry's
  value ≥ the key's current label, always.
- **(P) Pull semantics.** `pull` (327–344) removes the M smallest stored
  values; the separator is the remainder's minimum (the block bound when
  drained). Every entry that survives a pull is ≥ that pull's separator.

## 2. The engine, pinned

The proof consumes a fixed inventory of code facts, each re-verified
against the file in the final audit pass. They are the complete
inventory — exhaustiveness claims below rest on these lists being full.

**Relax sites (the only label mutations), all bounded by the call's own
B:** the FindPivots search relax (921–922), the base-case relax (1086),
the iteration relax sweep (1249), the W′ boundary sweep (1437–1438).
`relax` (700–731) accepts iff `cand ≤ d[v]` and `cand < B` — exact ties
re-accepted — and every accepted candidate tuple-strictly exceeds its
source's label (len adds w ≥ 0 and hops increases, so the lexicographic
order is strict even across zero-weight edges).

**Insert sites of a bmssp call's ds, with gates:** initial pivots
(1159, ungated); child-D merge (1212, ungated — 321–325); iteration
sweep (1250, gated `d ≥ b_i`); line-29 re-selection (1292, ungated);
line 33 (1334–1335, gated `B′ ≤ d < B`); W′ sweep (1439, gated
`d ≥ B′`). The base case inserts at two sites: seeds (1043–1045) and
post-pull relax (1086–1088, ungated — the Q3 channel).

**Iteration order (1176–1325):** pull → pivot expansion → child call →
merge → bookkeeping → relax sweep → re-selection → B′ overwrite → log
extension → loop-top check. One child call per iteration. The boundary
block (1329–1461) runs line 33, then the filter, then the W′ sweep — the
sweep strictly after the filter.

**FindPivots (884–1022):** runs before the call's ds exists. Sequential
searches from S-members; a search exits *failed* (heap drained with
|K| < k → K joins W), *capped* (|K| ≥ k → K becomes an F̄ tree), or
*merged* (contact with an existing tree routes the whole K, root
included, into that tree and reports nothing to W). Group pivots are
selected after all searches, reading current labels (1147–1159).

**Search mechanics** (916–958): lazy deletion (a popped entry not equal
to the current label is skipped); every accepted relax adopts the target
into K and pushes its new label — except the contact-triggering accept,
which breaks before the push (943–955). Two consequences used
repeatedly:

- **(Drain)** In a failed search every pushed entry is popped. Since
  every in-search improvement of v re-pushes v, the last accepted label
  of each K-member is popped *valid* — the member is "walked" at its
  search-end label, all its out-edges relaxed at the call's full bound.
- **(Mono)** Within one search, valid pop values are tuple-nondecreasing,
  and every accepted candidate strictly exceeds its source's pop value.
  Hence the root's start label ≤ every valid pop ≤ every accepted
  candidate. Labels only improve afterwards, so the root's *current*
  label at pivot-selection time obeys the same bound.

**u_log is the subtree settlement log.** A base call logs pulls only; a
bmssp call concatenates its children's logs (1319) and appends its W′
batch (1450). By induction, an appearance in an ancestor's u_i is a
settlement echo and nothing else, and every ancestor's relax sweep fires
every settled vertex's out-edges at its own bound.

## 3. The induction frame

The theorem is proved by induction on settlement **batches** in program
order: a base pull is a singleton batch; a W′ boundary append is one
batch read off one filter state. Suppose (H | V5) fails and consider the
first violating batch — the **first-violation regime**: every settlement
strictly before the current batch is complete. Within the batch, analyze
a violator y of minimal dis among the batch's incomplete members (the
batch tie-break: any batch-mate settling at length ≤ dis(z) below is
complete, else it would be a smaller-dis violator).

Fix a true shortest path to y, let z be its last complete vertex (the
source is complete, so z exists) and x₁ the next vertex after z
(possibly y). The strictness chain

  dis(z) ≤ dis(x₁) ≤ dis(y) < len(d[y]) ≤ len(β)

is strict at the incompleteness step, where β is the governing bound of
the event (the pulled entry's stored value at a base pull, B′ at a W′
append; incompleteness of the label transfers to β through (E)). All
recursion measures below are integers: position along y's true path,
program-order event time, and the within-batch dis-minimality.

The contradiction machine has three branches, established over the
sections that follow: **(i)** some live D holds a value ≤ dis(z) — killed
by the window (§4) and the no-leak lemma (§5); **(iii)** z was popped at
its final label by the event call's own FindPivots — killed because the
pop relaxed the tight edge (z, x₁) under the call's full bound and
completed x₁ before the event, advancing the frontier; **(ii)** z was
settled — killed by the echo argument (§8, case A). The channel
inventory (§6) shows one of (i)/(ii)/(iii) always holds. The W′ case of
the theorem is exactly Lemma d2 (§8); the base case closes in §9.

## 4. The window (Lemma W)

*At a settlement event with governing bound β, no live D holds a stored
value < β.*

- **(W1) Suspended ancestors.** Every stored value of a call's ds is
  < its block bound B (every insert site above is either relax-fed with
  `cand < B`, gated inside [B′, B), or stores a label < B by its guard;
  merges carry child values < B_child ≤ B). Hence pull separators are
  ≤ B and call bounds nest downward. While a child subtree is active,
  the parent's ds is untouched between its pull and the child call — no
  insert site executes in that span — so the parent's minimum is the
  pull remainder's minimum, which is ≥ the child's bound by (P).
  Composing up the stack: every suspended ancestor's ds sits ≥ the
  event call's bound ≥ β. This floor holds at *every* moment a call is
  suspended, not only at events.
- **(W3) Own D at a base pull.** The pull pops the global value-order
  minimum; nothing co-resident is below it.
- **(W2) Own D at a W′ append.** At the filter, ds holds the loop-exit
  survivors plus line-33 re-inserts (≥ B′ by their gate). Loop-exit
  survivors are ≥ B′ by the no-leak lemma, §5 — this clause is where
  the whole difficulty of the program lived, and it is the only clause
  that needs more than the pull mechanics.

## 5. The bound clause: no leaks below B′ (Lemma V)

The paper's Lemma 3.7 asserts dB[D] ≥ B′ over all stored values at every
return. As an unconditional invariant this is false even under V5-style
reasoning as a statement about the algorithm's text — the line-29
re-selection inserts ungated, and QUESTIONS.md Q12 documents the breach
shape. What survives, and what the proof needs, is the conditional form:

*In the first-violation regime under V5, at every bmssp return (and
already at the filter moment) every stored value is ≥ B′.*

**Step 1 — confinement (Sp).** An entry that survived the returning
call's last pull is ≥ b_f ≥ B′ ((P) plus B′ = the final child's bound
≤ b_f). Post-last-pull arrivals are: the final child's merged D
(recurse — same B′), final-sweep inserts (≥ b_f), line 33 and the W′
sweep (≥ B′), and the final-iteration line-29 re-selection — the only
ungated site. So a sub-B′ stored value at a return can only be a
re-selection mint from the final iteration of a call on the last-child
chain, storing exactly the group argmin's current label d[p] at
re-selection time (keep-min resolves to it: any older surviving entry
for p is ≥ b_f, any merged same-key value is ≥ the current label by
(E)). The key p is an unsettled S-member of its call.

**Step 2 — the mint's label needs a stale settlement.** How did d[p]
get strictly below B′ with p never settled? Trace the provenance chain
of p's label: each improvement's source, backward. Every link's source
either settled (and in the regime settled *complete*, so the offer's
length is pinned from settlement onward) or was itself improved — the
chain grounds, within the call that dropped d[p] below the bound, in a
settlement echo or in an S-member of that call walked by its FindPivots.
Two mechanisms then force custody or kill the drop:

- *The cascade arm.* When the ground settles, relax re-accepts exact
  ties (714), so the chain re-fires link by link at settlement sites,
  where an entry is structural: the base insert is ungated, and a
  refusal at one level is the next level's insert gate — refusal of a
  relax at a child's bound B_c means cand ≥ B_c, which *is* the parent
  sweep's insert gate b_i = B_c. The first level that accepts the relax
  also fires the insert (**one-step complementarity**). The minted
  entry rides ungated merges upward.
- *The stall arm.* If a chain-ground's custody entry is never pulled, it
  survives the last pull, so B′ ≤ b_f ≤ value; but the improvement
  chain is tuple-strictly ascending (hops strictly increase), so
  value < d[p] < B′ — a contradiction. Hence no sub-B′ custody entry
  survives to the return: either it is pulled and the cascade runs the
  chain down (each consumption settles complete by the regime and
  re-offers, advancing the ground), or the label never dropped below
  the bound at all.

The single escape from both arms is a ground whose settlement fired at
an incomplete label — a stale-at-filter W′ pass. In the first-violation
regime no such pass exists before the current batch. Therefore no
sub-B′ stored value exists at any return before the violation, nor at
the event call's own filter moment (the boundary block only inserts at
≥ B′ after the loop, so filter-moment and return-moment sub-B′ content
coincide). ∎

Lemma V is exactly the recorded counter `viol_bound_rec`: under V5 it
is zero corpus-wide, including the adversarial family built to pressure
this corner (`benchmarks/results/escape.txt`: paper engine 366
violations at L = 5000, V5 zero), and rows R1/R2 of
`benchmarks/results/rowi.txt` graze the bound at exact equality —
dB[D] = B′ — without breaching it. On the paper engine the same site
breaches freely; that difference is the entire content of V5.

## 6. Custody (Lemma Rep, domination, the anchor chain)

Branch (i) of the contradiction machine needs more than "a value exists
somewhere": it needs that from the moment z completes until the event,
some live D holds a value ≤ dis(z). Three devices carry this.

**Lemma Rep (representation).** In the first-violation regime, every
unsettled S-member v of every call C is represented from FindPivots
return to C's return: some live D at C or in C's active child subtree
holds an entry with value ≤ d[v], or v settles by C's boundary.
*Mechanism, per site:* FindPivots places every S-member in Q or in a
partition group; a failed root is in W and either settles at the
boundary or is re-entered by line 33. A group member is represented by
its group's pivot entry (initial insert at the argmin, ungated);
bookkeeping removes only settled members; every pivot death with a
non-empty group triggers the ungated re-selection at the new argmin's
current label. A pulled pivot that does not settle is an S-member of
the child — representation recurses — and at the child's return the
hand-back is line-33 re-entry, a returned-D entry, or a settlement
(which triggers re-selection). Pivot expansion (1188–1204) moves every
group member with d < b_i into the child's S at the pull, so
representation descends with the frontier rather than gapping. A label
drop below the current pivot's value re-points the pivot (the in-sweep
swap, 1259–1268) when the sweep gate passes, and lands in the
complementarity analysis when it refuses.

**Bound domination.** At a violating event, settlements in any subtree
happen strictly below the subtree's root bound, so len(d[y]) ≤ len(b)
for the current child bound b of every active ancestor — and hence
every value ≤ dis(z) sits strictly below every active gate (strict in
the len slot through the incompleteness step). Consequences: (D1) no
suspended ancestor's ds holds a value ≤ dis(z) at the event (its
current pull consumed anything that small — a survivor would drag the
separator below dis(z), contradicting the nesting); (D2) chain
representation is pulled into the active child at every level, so at
the event it lives in the event call's own D; (D3) a group holding a
sub-dis(z) entry had its pivot pulled in the current iteration at every
active level, and expansion moved the chain members down with it — the
re-arm gap between a pivot's settlement and its group's re-selection is
empty at violating events.

**The anchor chain (provenance).** Custody is created at z's completion
and maintained as a chain z = a₀, a₁, …, a_r where each link is a group
link (a_{i+1} is the pivot/argmin of a group containing a_i; entry
value ≤ d[a_i] ≤ dis(z) — selection reads current labels after all
searches, and (Mono) bounds search roots by what they walked) or a
tree/pocket link (a_{i+1} is the root of a search that walked a_i, with
the same (Mono) bound). Identity custody (the entry is z's own, at
dis(z)) is planted by the ungated base insert and by gate-passing
sweeps; proxy custody by capped/merged searches through their roots'
groups. Keep-min only lowers values; merges relocate entries upward
intact; pulls turn heads into child seeds (Rep) or expand members down
(D3); a settled head re-arms through its link (re-selection onto an
unsettled argmin; exact-tie re-offers down walk chains, with refusals
closed by one-step complementarity or, at the one genuinely entry-less
site, by the drop lemma of §7). Since z itself, unsettled, anchors the
chain's bottom, walking from any settled head to the deepest unsettled
element always exhibits custody with an unsettled backing — stale
duplicates of settled custodians never stand alone.

## 7. The W′-sweep drop lemma

One corner of the machinery has no complementary insert site: the W′
boundary sweep runs after the loop (no child call follows it inside the
call), and its insert gate `d ≥ B′` (1438) refuses any target it drops
strictly below B′ — while every *ancestor* echo of the same settlement
also refuses (ancestors' separators dominate the returning call's bound
chain). If a W′ settler's out-edge could first deliver a sub-B′ label
there, a completed vertex would exist with no entry anywhere. This
section proves the corner is empty. It is the one lemma of the chain
that was assembled during the final audit rather than in its own
derivation session, so it is re-derived here from scratch; the
devices it consumes (§2's Drain/Mono, §5, §6) are each independently
audited.

**Lemma D (drop lemma).** *In the first-violation regime under V5, let
z settle at call C's W′ boundary (z ∈ W(C), z ∉ U(C), d[z] < B′ at the
filter, the settlement complete). Then z was popped at its final label
— d at the pop equal to dis(z) — by a failed search of C's FindPivots.
Consequently every out-edge of z was relaxed at that pop, at the call's
full bound B(C), from z's final label.*

*Proof,* by strong induction on W′ settlement events in program order
(all of them complete in the regime).

z ∈ W(C) comes only from failed searches of C's FindPivots (§2: capped
searches report trees, merged searches report nothing). Let S₀ be a
failed search that adopted z. By (Drain), z's last accepted label
within S₀ was popped valid. Let L* be z's label at its *last* valid pop
across all of C's FindPivots searches — pops exist, and only site
921 can change labels while FindPivots runs, so between searches labels
are stable.

**Case 1: d[z] never improves after that pop.** Then the filter reads
L* = d[z] = dis(z): z was popped at its final label. It remains to show
the popping search failed. Suppose it was capped or merged. By (Mono)
the search's root x′ has current label ≤ its start label ≤ L* = dis(z)
at pivot-selection time. A capped root lands in its own tree; a merged
root is routed, K and all, into the contacted tree (943–955). Either
way x′ is an S-member of a tree, hence of a partition group (Q holds
failed roots only; the prior-group dedup can move x′ to an earlier
group, never out of all groups), and that group's pivot entry is
inserted ungated at the argmin's current label ≤ d[x′] ≤ dis(z): a
live entry ≤ dis(z) < B′ in C's ds from loop start. By Rep and the
anchor chain this custody persists — pulled heads settle complete (the
regime) and re-arm the chain, unpulled entries survive — so at C's
filter either z has settled inside C's own loop (z ∈ U(C),
contradicting z ∈ W′) or a live entry ≤ dis(z) sits in C's ds at the
filter, contradicting Lemma V (which pins every filter-moment value
≥ B′ > dis(z)). So the popping search failed, and (Drain) gives the
full-bound relax of every out-edge. This also closes the dichotomy's
other face below, so we record it once: *any* live custody ≤ dis(z)
persisting to C's filter is absurd — z ∈ U or a Lemma-V contradiction.

**Case 2: d[z] improved after its last valid pop.** The improvement is
a relax accept at one of the four sites, executed by an actor that runs
between C's FindPivots and C's filter — later searches of the same
FindPivots, or C's loop and its descendants (C's own W′ sweep runs
after the filter; suspended ancestors execute nothing). Take the *last*
improver e, which writes d[z] with len dis(z). Exhaust the sites:

- **(a) A later search of C's own FindPivots.** If that search failed,
  its heap drained and z's improved entry was popped valid — a later
  valid pop, contradicting L*'s maximality (this is the regress that
  makes the dichotomy exhaustive under multi-search label evolution:
  finitely many searches, so the last post-pop improver is not a failed
  search). If it was capped or merged — including the two no-push
  corners, an entry pushed but never popped because the search capped
  first, and the contact-triggering accept that pushes nothing — then
  by (Mono) through e's source, the improver's root x′ has label
  ≤ e's source's pop value < dis(z), and the Case-1 proxy argument
  applies verbatim: group pivot entry ≤ dis(z), custody persists,
  contradiction.
- **(b) A base relax in a descendant of C's loop.** The base insert is
  ungated (1086–1088): z's identity entry enters the base D at dis(z),
  the V5 cap bounds that base's return at ≤ dis(z), and the entry rides
  ungated merges upward. Any intermediate pull that consumes it seeds z
  into a deeper call, where a base pull settles z (z ∈ U(C) —
  contradiction) or the boundary hands the entry back (line 33 at
  d[z] = dis(z), or the returned D). In every branch z settles inside C
  or an entry ≤ dis(z) is live at C's filter — contradiction as in
  Case 1.
- **(c) An iteration relax sweep of C or a descendant.** If the sweep's
  insert gate passes, identity custody at dis(z) — contradiction as in
  (b). If it refuses, dis(z) < b_i = the bound of that iteration's
  child; the sweeping source settled complete in that child (the
  regime), so the same-length candidate was already offered inside the
  child, below the child's own relax bound, at the source's settlement
  site — the first level that accepted it also minted the entry
  (one-step complementarity, §5), grounding at the ungated base insert
  or at case (d). The entry merges up ungated — contradiction again.
- **(d) A W′ boundary sweep of a descendant call C″.** The sweeping
  source u settled complete at C″'s boundary — an *earlier* W′
  settlement event, so the induction hypothesis applies: u was popped
  at its final label dis(u) by a failed search of C″'s FindPivots,
  which relaxed (u, z) then, with the same candidate length
  dis(u) + w = dis(z), under B(C″) — before C″'s sweep. So z's label
  was already ≤ that candidate when the sweep ran, and the sweep's
  relax was a refusal or an exact length tie, not a length
  improvement: case (d) cannot be the last improver at length grade.
  (Moreover the earlier pop's accept adopted z into u's failed search's
  K, so z ∈ W(C″), and dis(z) < B′(C″) would settle z at C″'s own
  boundary — z ∈ U(C) by log extension — the co-settlement corollary
  one level down.)

Every branch of Case 2 is contradictory or vacuous, so Case 1 holds. ∎

**Corollary (co-settlement).** At a complete W′ settlement of z in call
C, for every out-edge (z, x): the candidate dis(z) + w was offered at
z's final-label pop under B(C). If it was accepted, x joined the failed
search's K ⊆ W(C), and if x's label at the filter is below B′, x
settles complete in the same batch as z (or had settled earlier). If it
was refused, x's label was already ≤ the candidate — for a tight edge
this means x was already complete. In all cases: every successor of z
along a tight edge with dis below B′ is complete at or before z's
settlement batch.

The corollary is what the refused-echo corner actually needs: when a
later echo of z's settlement drops x₁ below a bound with every insert
gate refusing, x₁ was already complete — there is nothing left for
custody to certify, and the frontier argument re-fires one vertex
further along the path.

*Audit note.* The two designated attack points for this re-derivation
were (1) the exhaustiveness of the walked-at-final/completed-later
dichotomy when several searches of one FindPivots evolve z's label —
closed above by basing the dichotomy on the last valid pop and
exhausting the improver's search type, with the two no-push corners
(cap-before-pop, contact accept) handled inside the capped/merged arm —
and (2) the cascade's ground when custody crosses a call boundary
mid-grind — closed by routing every boundary crossing through the
ungated child-D merge, the line-33 hand-back, or case (d)'s event-order
induction, so the chain never dangles across a return. One sharpening
over the recorded sketch: the failed-search attribution in Case 1 is
*forced* (a final-label pop by a capped or merged search plants proxy
custody that contradicts Lemma V), not assumed.

## 8. The frontier-order lemma (d2)

*In the first-violation regime under V5, no W-member with an incomplete
label below B′ passes the W′ filter.*

This is the W′ case of the theorem. Suppose y passes C_e's filter with
len(d[y]) > dis(y) and d[y] < B′. Fix y's true path, z, x₁ as in §3.

**Case A: z settled before the event.** Let C_prev be z's settling call
and D_ca the deepest common ancestor of C_prev and C_e's seeding in the
call tree. Each loop iteration makes exactly one child call, so
distinct subtrees under D_ca sit in distinct iterations: C_prev's
iteration at D_ca strictly precedes C_e's seeding (if C_prev sits
inside C_e's own subtree, D_ca degenerates to C_e and the echo below is
the same-iteration mechanism of case B.2). Settlement logs propagate to
every ancestor (§2), so D_ca's relax sweep attempted (z, x₁) at D_ca's
own bound, and bounds nest: B(D_ca) ≥ B(C_e) > d[y] > dis(x₁). The echo
relax accepts, so x₁ was complete before C_e began. Custody of x₁ is
minted by the sweep gate or by one-step complementarity — and at the
one refusing corner (z settled at a W′ boundary, every gate above
refusing) the drop lemma's corollary applies instead: x₁ was complete
already at z's settlement batch. The frontier advances one path
position and the argument re-fires at x₁ — a strictly larger integer
measure, so the settled-z geometry self-destructs into case B.

**Case B: z unsettled at the event.** By Rep and the anchor chain, z
has live custody ≤ dis(z). By W1's suspended floor and D1–D3 it cannot
idle in an ancestor or an un-re-armed group: at the event it is a live
entry in C_e's own ds, value ≤ dis(z). Two endings:

1. **The entry survives to the filter.** By Lemma V's mechanics it then
   survived the last pull or rode the final child's returned D; both
   pin B′ ≤ value ≤ dis(z) < dis(y) < len(d[y]) — so y *fails* the
   filter. The interval (dis(y), len(d[y])) that B′ would have to
   enter to pass a stale y is empty: B′ cannot clear the stale label
   without first clearing the custody below it. (This is R4's recorded
   geometry: B′ = 50.4375 ≤ dis(z) = 50.5 < d[y] = 51.0625, the stale
   y refused.)
2. **The entry is pulled.** The pulled head settles complete in the
   iteration's child (the regime), and the settlement echoes at C_e's
   own relax sweep *in the same iteration*, before the loop-top check,
   at C_e's wide bound B > d[y] > every chain candidate. Each
   consumption therefore advances the true-path frontier at least one
   vertex and re-mints custody at the new frontier (sweep gate,
   one-step complementarity riding the child's returned D through the
   ungated merge, or — at the entry-less corner — the drop lemma's
   co-settlement). The frontier's value ≤ dis(y) < d[y] sits below
   every separator that could pass y, so it re-enters every next pull
   as part of the block minimum; heads pulled without settling hand
   back by line 33 or the returned D at unchanged value. The loop is
   finite, so at exit either the cascade passed y — y complete before
   the filter, no violation (row R3's recorded face) — or a frontier
   entry ≤ dis(z′) < d[y] is live at exit, which is ending 1 at the
   advanced frontier.

Both endings contradict the violating pass. ∎

## 9. The theorem

**Theorem (H | V5).** On the V5 engine, at length grade, every
settlement event settles a complete vertex.

*Proof.* Induct on settlement batches; suppose the first violating
batch exists and pick y, z, x₁ as in §3.

If the event is a W′ append, Lemma d2 (§8) already forbids it.

If the event is a base pull with governing value β (the pulled
minimum): z complete means z's completing relax happened at one of the
four sites, and the channel inventory of §6 leaves branch (i) custody,
branch (ii) z settled, or branch (iii) z popped at final label by the
event call's own search — but a base call runs no searches, so (iii)
here means z's final-label pop happened in some call's FindPivots whose
custody or settlement consequences land in (i)/(ii) anyway. Branch (i):
a live value ≤ dis(z) < len(β) sits in the base D itself —
contradicting the pull's minimality (W3) — or in an ancestor —
contradicting W1's floor. Branch (ii): z settled strictly earlier, so
its settlement echoed at the deepest common ancestor of its settling
call and the event call's seeding (or at the event call itself: a base
call re-relaxes every settled vertex's edges at its own pull,
1085–1094, ungated insert) with an accepting bound as in case A of §8 —
x₁ completed before the event and the argument re-fires at x₁, a
strictly larger path position. The measure is finite; no first
violation exists. ∎

## 10. The experimental record

The proof's negative space — what (H) looks like when it fails — is
recorded, and every counter the proof names was measured. All rows
below run three engines (paper, V5, V8) on the same input with a
Dijkstra oracle; `settle-inc` counts incomplete settlements,
`viol rec` counts sub-B′ returned values.

| Record | What it shows |
| --- | --- |
| `wprime.txt` row G | (H) falsified live for the paper engine and V8: `settle-inc=1`, y_t settles at 20.90625 against true 20.859375, with `viol base/rec = 1/1`. V5 on the same input: `settle-inc=0`, B′ capped to 20.84375. |
| `rowh.txt` | The same leak embedded in a full recursion (n = 70): V8 finishes with a **wrong final distance** (v67: got 23.59375, want 23.546875); the paper engine heals through one duplicate settlement (`dups=1`, distances exact); V5 is clean (`settle-inc=0`, `cap=1`, exact). |
| `escape.txt` | The bound clause at scale: paper engine `viol_bound_rec` = 12 at L = 200 and 366 at L = 5000; V5 zero at every L. |
| `rowi.txt` rows R1/R2 | Lemma V's two faces, built predictions-first: R1 exhibits the complementarity mint being pulled and settled (custody discharge, B′ = 52, every counter 0); R2 exhibits the equality wall — a cutoff exit returning D = {v@51.8125} with B′ = 51.8125 exactly, dB[D] = B′, grazed not breached. |
| `rowi.txt` rows R3/R4 | Lemma d2's two faces, built predictions-first: R3 the repair face (the wide child repairs y at 50.75 before the filter; y settles complete); R4 the refusal face (B′ = 50.4375 pinned at the unpulled custody below dis(z) = 50.5; the stale y@51.0625 refused; z's custody rides the returned D). All three engines identical on all four rows — `cap=0`, `dups=0`, so the engines' only divergence sites never trigger, as predicted in the driver header before the first run. |

Corpus-wide, every V5 row of every recorded family shows
`settle-inc = 0` and `viol base/rec = 0/0` (wprime, rowh, rowi,
triangle, escort, escape, stale_incomplete — see
`benchmarks/results/`); the only nonzero settle-inc rows anywhere are
row G's and row H's paper/V8 rows. Under the theorem this is no longer
an empirical observation but the predicted face of Lemmas V and d2.

## 11. Grade, limitations, and open items

- **Length grade.** "Complete" compares label *length* to true
  distance. The tuple-grade statement — settlement at the canonical
  4-tuple — is open, with three named pressure points recorded in
  QUESTIONS.md (addenda 17/21/23–25): the tie-refusal corner of branch
  (iii) at equal length; post-settlement tuple drift (a complete vertex
  tuple-improving at equal length re-offers invisibly to length-grade
  custody); and the places where this proof consumes tuple-strict
  inequalities (the stall arm's ascending chain, the hops-based path
  recursion) whose interaction with the older length-grade wording is
  an interface, not yet a theorem.
- **First-violation regime.** The theorem is an induction, so each
  lemma above is stated inside the regime; none of the conditional
  lemmas (V, Rep, D, d2) is claimed unconditionally.
- **The paper's contract stays broken.** Nothing here repairs Lemma 3.7
  as stated: its clause 2 (dB[D] ≥ B′, absolute) is breachable on the
  paper engine (recorded), and its three joint clauses admit no
  return-boundary repair (VARIANTS.md). V5 restores the base case's
  Lemma 3.1 contract, and the theorem shows that this single repair is
  enough for settlement completeness. The |U|-counting totality
  question (Q10/Q13) is separate and remains open.
- **Proof grade.** Hand proof against the frozen port, every step
  re-derived from the code in a later independent pass (QUESTIONS.md
  addenda 16–25: five derivation/audit pairs; the §7 lemma re-derived
  in the pass that produced this document). Not machine-checked.
