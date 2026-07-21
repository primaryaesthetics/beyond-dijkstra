# The completeness lemma for the V5 engine

This document consolidates the proof of the settlement-completeness
property (H) for the V5 variant of our port of arXiv:2602.07868 — the
result that the Q14 research program (QUESTIONS.md) was opened to decide.
The program's history, its falsifications, and the session-by-session
derivation trail live in QUESTIONS.md Q14 and its addenda 1–31; this
document is the linear write-up of the surviving chain at its final
grade, self-contained modulo SPEC.md (the pinned semantics) and
VARIANTS.md (the engine definitions).

**Statement proved.** On the V5 engine (`fixed_base`, VARIANTS.md),
every settlement event — a base-case pull or a W′ boundary append —
settles a vertex whose label carries the canonical **(len, hops)** pair:
the lexicographic minimum of (length, hop count) over all
source-to-vertex paths. We call this (H_lh | V5). The length-grade
reading — settled length equals true shortest-path distance — is its
len-slot projection.

**What it is not.** This is a careful hand proof against a specific,
frozen implementation, not a machine-checked proof and not a proof about
the paper's algorithm as written — (H) is *false* for the paper's
algorithm and for the V8 variant, by recorded counterexample (§10). Each
step of the chain was derived in one working session and re-derived from
the code in a separate, later session before anything was built on it;
the audit trail is QUESTIONS.md addenda 16–31 (eight derivation/audit
pairs). The label's fourth slot (pred) is excluded from the claim by
design, not by debt — §11; §12 sharpens the exclusion to its final
form: settlement labels are bit-final, pred included, and the pred
residual is an arrival-set property, never drift.

Every number in this document traces to a file under `benchmarks/results/`.
Line references are to `src/solver.rs` at the commit that introduced this
document; the solver is unchanged since, and every reference was
re-verified in the tuple-grade audit pass and again in the pass that
produced §12.

---

## 1. Objects and notation

The solver is the BMSSP recursion of arXiv:2602.07868 as pinned by
SPEC.md: labels are 4-tuples ⟨len, hops, curr, pred⟩ compared
lexicographically (a total order; `src/label.rs` 24–32; the Inf sentinel
sits strictly above every finite label and is decided by enum match —
it never reaches any slot), `d[v]` is v's current label, and a call
`bmssp(B, S, level)` receives a bound B and seed set S and returns
`(B′, U, D)` — a bound B′ ≤ B, the settlement log U, and its leftover
data structure D. Base calls (Algorithm 4) do the same with a
Dijkstra-style loop. Notation:

- **Path tuples.** For a path π to v write
  T(π) = ⟨len(π), hops(π), v, pred(π)⟩. Every finite stored label is a
  path tuple: the source label ⟨0, 0, s, −1⟩ (1532–1537) is the empty
  path's, and an accepted candidate ⟨d[u].len + w, d[u].hops + 1, v, u⟩
  is the tuple of the source's path extended by (u, v) — induction over
  accepts.
- **lh(v)** — the lexicographic minimum of (len(π), hops(π)) over all
  s→v paths; **dis(v)** = lh(v).len is the true shortest-path length.
- **complete** — (d[v].len, d[v].hops) = lh(v) (lh-completeness). By
  (Floor) below, incomplete means prefix-strictly above.
- **settlement event** — a base-case pull (solver.rs:1048–1055) or a W′
  boundary append (the filter at 1346–1354). These are exactly the two
  sites the settlement log records (1055 via `u_log.push`, 1450 via the
  W′ extension), and the two sites the `settle_incomplete` counter
  checks (1057–1062, 1360–1370).
- **(H_lh)** — no settlement event settles an incomplete vertex.
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

Standing facts used throughout, all structural:

- **(E) Stored values never sit below labels.** Every insert site stores
  the vertex's current label at insert time; keep-min (300–316) keeps
  the minimum of historical labels; labels only improve. So an entry's
  value ≥ the key's current label, always.
- **(P) Pull semantics.** `pull` (327–344) removes the M smallest stored
  values; the separator is the remainder's minimum (the block bound when
  drained). Every entry that survives a pull is ≥ that pull's separator.
- **(Floor).** Since every stored label is a path tuple,
  (d[v].len, d[v].hops) ≥_lex lh(v) always — labels sit at or above the
  canonical pair, never below.
- **(Prefix optimality).** A prefix of an lh-minimal path is lh-minimal
  for its own endpoint: a shorter prefix contradicts len-minimality; an
  equal-len fewer-hops prefix contradicts hops-minimality among
  len-minimal paths. The full-tuple analogue is *false* — the min-pred
  tie-break at v constrains nothing about the predecessor's own pred —
  which is why the pred slot is scope, not target (§§11–12).
- **(S) The slot lemma.** The candidate at 708–713 reads only
  (len, hops) of its source plus the two vertex ids — the candidate's
  pred slot is the source's *id*, not the source's pred. So: **(S1)**
  from a complete source u, every candidate out of u carries the pinned
  pair (lh(u).len + w, lh(u).hops + 1), independent of u's pred slot
  and of any later change to it; **(S2)** an improvement of d[u] in the
  pred slot alone leaves every candidate out of u bit-identical — pred
  drift is candidate-invisible.
- **(Z) Path monotonicity, natively strict.** Along any path,
  consecutive prefix tuples are lex-strictly increasing: len adds
  w ≥ 0 and hops increases by 1, so the order is strict even across
  zero-weight edges. Two consequences: lh-canonical paths are finite
  and cycle-free on the reduced graph (any cycle has len ≥ 0 and
  hops ≥ 1, so removing it is a lex improvement — the zero-weight
  cycles the degree reduction manufactures strip themselves), and
  **the workhorse**: a comparison that is lex-strict already in the
  (len, hops) prefix is decided there — full-tuple strict against
  arbitrary curr/pred tails.

## 2. The engine, pinned

The proof consumes a fixed inventory of code facts, each re-verified
against the file in the audit passes. They are the complete
inventory — exhaustiveness claims below rest on these lists being full.

**Relax sites (the only label mutations), all bounded by the call's own
B:** the FindPivots search relax (921–922), the base-case relax (1086),
the iteration relax sweep (1249), the W′ boundary sweep (1437–1438).
`relax` (700–731) forms the candidate ⟨d[u].len + w, d[u].hops + 1,
v, u⟩ and accepts iff `cand ≤ d[v] && cand < b` (714) — a **label
clause** and a **bound clause**; exact full-tuple ties are re-accepted
(a no-op store). Every accepted candidate tuple-strictly exceeds its
source's label ((Z)).

**Insert sites of a bmssp call's ds, with gates:** initial pivots
(1159, ungated); child-D merge (1212, ungated — 321–325); iteration
sweep (1250, gated `d ≥ b_i`); line-29 re-selection (1292, ungated);
line 33 (1334–1335, gated `B′ ≤ d < B`); W′ sweep (1439, gated
`d ≥ B′`). The base case inserts at two sites: seeds (1043–1045) and
post-pull relax (1086–1088, ungated — the Q3 channel). All gates and
the W′ filter (1351) compare full tuples.

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
to the current label — a full-tuple test, 918 — is skipped); every
accepted relax adopts the target into K and pushes its new label —
except the contact-triggering accept, which breaks before the push
(943–955). Only site 922 changes labels while FindPivots runs, so a
label that drifts mid-search (including a pred-only drift) strands only
stale heap entries, which are popped and skipped. Two consequences used
repeatedly:

- **(Drain)** In a failed search every pushed entry is popped. Since
  every in-search improvement of v re-pushes v (956), the last accepted
  label of each K-member is popped *valid* — the member is "walked" at
  its search-end label, all its out-edges relaxed at the call's full
  bound.
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
batch read off one filter state. Suppose (H_lh | V5) fails and consider
the first violating batch — the **first-violation regime**: every
settlement strictly before the current batch is complete. Within the
batch, analyze a violator y whose lh is lex-minimal among the batch's
incomplete members (ties broken by vertex id; the tie-break is never
read — see below).

Fix an lh-canonical path to y (cycle-free by (Z)), let z be its last
complete vertex (the source is complete and unimprovable — a len ≤ 0
candidate into s would close a zero-length cycle whose hops ≥ 1 lose at
the second slot — so z exists) and x₁ the next vertex after z (possibly
y). Write τ_z := d[z]; completeness pins prefix(τ_z) = lh(z), and since
a later pred-only drift of z moves no prefix slot, every *strict*
comparison against τ_z below is decided in the prefix and is insensitive
to which same-prefix version of z's label is read. The strictness chain

  prefix(τ_z) = lh(z) <_lex lh(x₁) ≤_lex lh(y)
  <_lex (d[y].len, d[y].hops) ≤_lex prefix(β)

is strict at the first step by (Z) along the canonical path, strict at
the incompleteness step by (Floor), and the β-transfer is full-tuple
through (E), where β is the governing bound of the event (the pulled
entry's stored value at a base pull, B′ at a W′ append). By the
workhorse, every comparison the chain decides in the prefix is decided
full-tuple: in particular τ_z < β and every value with prefix
≤_lex lh(z) sits full-tuple below β, below d[y], and below every bound
that dominates β. All recursion measures below are integers: position
along y's canonical path, program-order event time, and the batch pick.

**The batch tie-break is unread.** The only within-batch fact any step
consumes is: a batch-mate settling with prefix ≤_lex lh(z) is complete.
Indeed, were such a mate incomplete, its lh would sit lex-strictly
below its settled prefix ≤_lex lh(z) <_lex lh(y) — a lex-smaller
violator, contradicting the pick, without ever comparing ids.

**The refusal-floor lemma.** *Let u be complete and (u, x) an edge of
an lh-canonical path through u, so the candidate
c = ⟨lh(u).len + w, lh(u).hops + 1, x, u⟩ carries exactly lh(x) in its
prefix — by prefix optimality, and by (S1) regardless of u's pred slot
or any later drift in it. If a relax of (u, x) at candidate c is
refused by the* label *clause — d[x] < c full-tuple — then x is already
complete.* Proof: by (Floor), prefix(d[x]) ≥_lex lh(x) = prefix(c);
combined with d[x] < c full-tuple, the prefixes must be equal (a
lex-greater prefix would force d[x] > c), so the strict comparison was
decided at curr (equal — both x) or pred: x holds the canonical
(len, hops) with a smaller pred id. ∎ A label-test refusal of a
canonical candidate certifies the very completeness the relax would
have delivered: the frontier advances one path position whether the
relax accepts (x adopts c's prefix) or label-refuses (x was complete
already).

**The bound face.** The lemma reads only the label clause; the
acceptance test (714) also refuses when ¬(c < b), and a bound-refused
relax implies nothing about d[x]. At every site where the chain
consumes the lemma the bound face is closed in one of two ways:

- *(The strict chain.)* At branch (iii)'s pop relax, the DCA echo (§8
  case A), and the cascade re-mints (§8 ending 2), the relax runs at a
  bound b_site with d[y] < b_site full-tuple — at a W′ event because
  y ∈ W(C_e) was adopted by a failed search's accept under the call's
  relax bound B(C_e) (922) and labels only improve, with every echo
  bound dominating B(C_e) by full-tuple bound nesting; at a base event
  because d[y] ≤ β < B through (E) and W1's per-site audit (§4). Then

    prefix(c) = lh(x₁) ≤_lex lh(y) <_lex prefix(d[y]) ≤_lex prefix(b_site)

  is lex-strict in the prefix through the incompleteness gap, so by the
  workhorse c < b_site full-tuple: the bound face cannot fire.
- *(Self-cancellation at the drop corner.)* At the one consuming site
  outside the violator geometry — the co-settlement corollary of §7 —
  the bound face self-cancels: the final-label pop and the W′ sweep
  relax the same edge at the *same* bound with bit-identical candidates,
  so a bound-refused pop forces a bound-refused sweep and the drop in
  question never happens. Derived in §7.

The contradiction machine has three branches, established over the
sections that follow: **(i)** some live D holds a value with prefix
≤_lex lh(z) — killed by the window (§4) and the no-leak lemma (§5);
**(iii)** z was popped at its final label by the event call's own
FindPivots — killed because the pop offered x₁ its canonical candidate
under the call's full bound (the bound face closed by the strict
chain), so x₁ was complete before the event — by adoption or by the
refusal floor — advancing the frontier; **(ii)** z was settled — killed
by the echo argument (§8, case A). The channel inventory (§6) shows one
of (i)/(ii)/(iii) always holds. The W′ case of the theorem is exactly
Lemma d2 (§8); the base case closes in §9.

## 4. The window (Lemma W)

*At a settlement event with governing bound β, no live D holds a stored
value < β.*

- **(W1) Suspended ancestors.** Every stored value of a call's ds is
  < its block bound B (every insert site above is either relax-fed with
  `cand < B`, gated inside [B′, B), or stores a label < B by its guard;
  merges carry child values < B_child ≤ B — all full-tuple facts).
  Hence pull separators are ≤ B and call bounds nest downward. While a
  child subtree is active, the parent's ds is untouched between its
  pull and the child call — no insert site executes in that span
  (1188–1210) — so the parent's minimum is the pull remainder's
  minimum, which is ≥ the child's bound by (P). Composing up the stack:
  every suspended ancestor's ds sits ≥ the event call's bound ≥ β. This
  floor holds at *every* moment a call is suspended, not only at
  events.
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
of p's *current* label: each improvement's source, backward. Every
link's source either settled — and in the regime settled *complete*,
so the offer's (len, hops) is pinned at the floor from settlement
onward: no later accept can move those slots, and pred drift does not
enter candidates ((S1)/(S2)) — or was itself improved; the chain
grounds, within the call that dropped d[p] below the bound, in a
settlement echo or in an S-member of that call walked by its
FindPivots. By (Z) the chain is tuple-strictly ascending, hence finite
and acyclic. Two mechanisms then force custody or kill the drop:

- *The cascade arm.* When the ground settles, relax re-accepts exact
  ties (714) — and the re-offer at an unchanged link is an exact
  *tuple* tie: same source id and same pinned prefix make the candidate
  bit-identical ((S2)) — so the chain re-fires link by link at
  settlement sites, where an entry is structural: the base insert is
  ungated, and a refusal at one level is the next level's insert gate —
  refusal of a relax at a child's bound B_c means cand ≥ B_c, which
  *is* the parent sweep's insert gate b_i = B_c. The first level that
  accepts the relax also fires the insert (**one-step
  complementarity**). The minted entry rides ungated merges upward.
  One face is specific to tuple grade — **the re-base face**: a
  re-fired bit-identical offer can be *label*-refused because the
  target link v itself improved below it since the chain was traced,
  including a pred-only drift of v (same prefix, smaller pred). The
  refusal means d[v] < cand: v's current label is a strictly smaller
  tuple, and the accept that wrote it — one of the four relax sites —
  roots a *new* provenance chain through v, again tuple-strictly
  ascending by (Z) and again grounded in a settlement echo or a walked
  S-member. Both arms apply to the re-based chain verbatim; no step of
  the machinery asks *why* a refusal fired, only what it implies in
  the total order.
- *The stall arm.* If a chain-ground's custody entry is never pulled, it
  survives the last pull, so B′ ≤ b_f ≤ value; but the improvement
  chain is tuple-strictly ascending ((Z): hops strictly increase along
  offers), so value < d[p] < B′ — a contradiction. Hence no sub-B′
  custody entry survives to the return: either it is pulled and the
  cascade runs the chain down (each consumption settles complete by
  the regime and re-offers, advancing the ground), or the label never
  dropped below the bound at all.

The single escape from both arms is a ground whose settlement fired at
an incomplete label — a stale-at-filter W′ pass at lh grade. In the
first-violation regime no such pass exists before the current batch.
Therefore no sub-B′ stored value exists at any return before the
violation, nor at the event call's own filter moment (the boundary
block only inserts at ≥ B′ after the loop, so filter-moment and
return-moment sub-B′ content coincide). ∎

Lemma V's len-slot face is exactly the recorded counter
`viol_bound_rec`: under V5 it is zero corpus-wide, including the
adversarial family built to pressure this corner
(`benchmarks/results/escape.txt`: paper engine 366 violations at
L = 5000, V5 zero), and rows R1/R2 of `benchmarks/results/rowi.txt`
graze the bound at exact equality — dB[D] = B′ — without breaching it.
On the paper engine the same site breaches freely; that difference is
the entire content of V5.

*A note on complete sub-B′ entries.* Lemma V pins *stored values*
≥ B′ through the stale-settlement escape; it does not need to forbid —
and the machinery nowhere forbids — an entry of an already-complete
vertex sitting low: a pred-drift accept can insert a settled vertex at
its floor prefix with a smaller pred. Pulled, such an entry settles a
duplicate of a complete vertex (harmless to (H_lh); duplicates are a
recorded, counted phenomenon — the paper engine heals row H through
one); unpulled, it meets step 1's confinement like any value. A
separator moved by such an entry — at a new (len, hops) or at pred
alone — is consumed by every proof step as a value in the total order
((P), confinement, complementarity's bound identity): slot-agnostic
throughout, since bounds are stored values (or Inf) and any comparison
reaching the pred slot forces (len, hops, curr) equality — the same
vertex.

## 6. Custody (Lemma Rep, domination, the anchor chain)

Branch (i) of the contradiction machine needs more than "a value exists
somewhere": it needs that from the moment z completes until the event,
some live D holds a value ≤ τ_z — equivalently, with prefix
≤_lex lh(z), which is the form every strict consumption below reads
(the workhorse decides it against every bound in the violator chain).
Three devices carry this.

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
happen strictly below the subtree's root bound — a full-tuple fact:
settled labels pass full-tuple gates — so d[y] < b for the current
child bound b of every active ancestor; hence every value with prefix
≤_lex lh(z) sits full-tuple below every active gate (prefix-strict
through the incompleteness gap, decided by the workhorse).
Consequences: (D1) no suspended ancestor's ds holds a value ≤ τ_z at
the event (its current pull consumed anything that small — a survivor
would drag the separator below τ_z, contradicting the nesting); (D2)
chain representation is pulled into the active child at every level, so
at the event it lives in the event call's own D; (D3) a group holding
an entry ≤ τ_z had its pivot pulled in the current iteration at every
active level, and expansion moved the chain members down with it — the
re-arm gap between a pivot's settlement and its group's re-selection is
empty at violating events.

**The anchor chain (provenance).** Custody is created at z's completion
and maintained as a chain z = a₀, a₁, …, a_r where each link is a group
link (a_{i+1} is the pivot/argmin of a group containing a_i; entry
value ≤ d[a_i] ≤ τ_z — selection reads current labels after all
searches, and (Mono) bounds search roots by what they walked; all
heap-order facts, full tuple) or a tree/pocket link (a_{i+1} is the
root of a search that walked a_i, with the same (Mono) bound). Identity
custody (the entry is z's own, at τ_z) is planted by the ungated base
insert and by gate-passing sweeps; proxy custody by capped/merged
searches through their roots' groups. Keep-min only lowers values;
merges relocate entries upward intact; pulls turn heads into child
seeds (Rep) or expand members down (D3); a settled head re-arms through
its link (re-selection onto an unsettled argmin; bit-identical
re-offers down walk chains ((S1)/(S2)), with refusals closed by
one-step complementarity, by the refusal floor or the re-base face
(§5) at label refusals, or, at the one genuinely entry-less site, by
the drop lemma of §7). Since z itself, unsettled, anchors the chain's
bottom, walking from any settled head to the deepest unsettled element
always exhibits custody with an unsettled backing — stale duplicates of
settled custodians never stand alone.

## 7. The W′-sweep drop lemma

One corner of the machinery has no complementary insert site: the W′
boundary sweep runs after the loop (no child call follows it inside the
call), and its insert gate `d ≥ B′` (1438) refuses any target it drops
strictly below B′ — while every *ancestor* echo of the same settlement
also refuses (ancestors' separators dominate the returning call's bound
chain). If a W′ settler's out-edge could first deliver a sub-B′ label
there, a completed vertex would exist with no entry anywhere. This
section proves the corner is empty. The lemma was first assembled
during the length-grade audit and re-derived from scratch in the
consolidation pass; the devices it consumes (§2's Drain/Mono, §5, §6)
are each independently audited.

**Lemma D (drop lemma).** *In the first-violation regime under V5, let
z settle at call C's W′ boundary (z ∈ W(C), z ∉ U(C), d[z] < B′ at the
filter, the settlement complete). Then z was popped at its final label
— the pop label equal to d[z] at the filter, carrying prefix lh(z) —
by a failed search of C's FindPivots. Consequently every out-edge of z
was relaxed at that pop, at the call's full bound B(C), from z's final
label.*

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
L* = d[z]: z was popped at its final label. It remains to show the
popping search failed. Suppose it was capped or merged. By (Mono)
the search's root x′ has current label ≤ its start label ≤ L* = τ_z
at pivot-selection time. A capped root lands in its own tree; a merged
root is routed, K and all, into the contacted tree (943–955). Either
way x′ is an S-member of a tree, hence of a partition group (Q holds
failed roots only; the prior-group dedup can move x′ to an earlier
group, never out of all groups), and that group's pivot entry is
inserted ungated at the argmin's current label ≤ d[x′] ≤ τ_z: a
live entry ≤ τ_z < B′ in C's ds from loop start (τ_z < B′ full-tuple —
z's own filter pass). By Rep and the anchor chain this custody
persists — pulled heads settle complete (the regime) and re-arm the
chain, unpulled entries survive — so at C's filter either z has settled
inside C's own loop (z ∈ U(C), contradicting z ∈ W′) or a live entry
≤ τ_z sits in C's ds at the filter, contradicting Lemma V (which pins
every filter-moment value ≥ B′ > τ_z). So the popping search failed,
and (Drain) gives the full-bound relax of every out-edge. This also
closes the dichotomy's other face below, so we record it once: *any*
live custody ≤ τ_z persisting to C's filter is absurd — z ∈ U or a
Lemma-V contradiction.

**Case 2: d[z] improved after its last valid pop** — in any slot,
pred-only drift included. The improvement is a relax accept at one of
the four sites, executed by an actor that runs between C's FindPivots
and C's filter — later searches of the same FindPivots, or C's loop and
its descendants (C's own W′ sweep runs after the filter; suspended
ancestors execute nothing). Take the *last* improver e, which writes
d[z] — the filter-time label, prefix lh(z). Exhaust the sites:

- **(a) A later search of C's own FindPivots.** If that search failed,
  its heap drained and z's improved entry was popped valid — a later
  valid pop, contradicting L*'s maximality (this is the regress that
  makes the dichotomy exhaustive under multi-search label evolution:
  finitely many searches, so the last post-pop improver is not a failed
  search). If it was capped or merged — including the two no-push
  corners, an entry pushed but never popped because the search capped
  first, and the contact-triggering accept that pushes nothing — then
  by (Mono) through e's source, the improver's root x′ has label
  ≤ e's source's pop value < τ_z (strict by (Z) through the accepted
  candidate), and the Case-1 proxy argument applies verbatim: group
  pivot entry ≤ τ_z, custody persists, contradiction.
- **(b) A base relax in a descendant of C's loop.** The base insert is
  ungated (1086–1088): z's identity entry enters the base D at the
  accepted label ≤ τ_z, the V5 cap bounds that base's return at ≤ the
  entry, and the entry rides ungated merges upward. Any intermediate
  pull that consumes it seeds z into a deeper call, where a base pull
  settles z (z ∈ U(C) — contradiction) or the boundary hands the entry
  back (line 33 at the unchanged label, or the returned D). In every
  branch z settles inside C or an entry ≤ τ_z is live at C's filter —
  contradiction as in Case 1.
- **(c) An iteration relax sweep of C or a descendant.** If the sweep's
  insert gate passes, identity custody ≤ τ_z — contradiction as in
  (b). If it refuses, the accepted label < b_i = the bound of that
  iteration's child; the sweeping source settled complete in that child
  (the regime), so the bit-identical candidate ((S1)/(S2): the source's
  prefix is pinned at its floor and the candidate's pred slot is the
  source id) was already offered inside the child, below the child's
  own relax bound, at the source's settlement site — the first level
  that accepted it also minted the entry (one-step complementarity,
  §5), grounding at the ungated base insert or at case (d). The entry
  merges up ungated — contradiction again.
- **(d) A W′ boundary sweep of a descendant call C″.** The sweeping
  source u settled complete at C″'s boundary — an *earlier* W′
  settlement event, so the induction hypothesis applies: u was popped
  at its final label by a failed search of C″'s FindPivots, which
  relaxed (u, z) then, with the *bit-identical* candidate — same source
  id, same pinned prefix ((S1)/(S2)) — under the same bound B(C″)
  (both sites pass the call's own b: 922, 1438) — before C″'s sweep.
  So at the sweep the offer is a refusal or an exact tuple tie (a
  no-op store): if the pop's offer was accepted, d[z] ≤ cand ever
  since; if label-refused, d[z] < cand already; if bound-refused, the
  sweep's identical candidate is bound-refused too. Case (d) cannot be
  an improver in *any* slot — the length-grade wording "not a length
  improvement" had residual room at the hops slot, and this closes it.
  (Moreover the earlier pop's accept adopted z into u's failed search's
  K, so z ∈ W(C″), and d[z] < B′(C″) would settle z at C″'s own
  boundary — z ∈ U(C) by log extension — the co-settlement corollary
  one level down.)

Every branch of Case 2 is contradictory or vacuous, so Case 1 holds. ∎

**Remark (bit-finality from the last valid pop).** Case 2's site
exhaustion is slot-agnostic — a pred-only improver is an accept at one
of the four sites, and every arm ends in the same custody/Lemma-V
contradiction — so in the regime a W′ settler's label is bit-final
from its last valid pop: pop label, filter label, and settled label
are one bit-identical tuple, pred included. First recorded as an
unconsumed observation in the lh-grade revision; now consumed as the
W′ face of the bit-finality theorem (§12).

**Corollary (co-settlement).** At a complete W′ settlement of z in call
C, for every out-edge (z, x): the candidate
⟨lh(z).len + w, lh(z).hops + 1, x, z⟩ was offered at z's final-label
pop under B(C). Three faces, exhausting the acceptance test:

- *Accepted:* x joined the failed search's K ⊆ W(C), and if x's label
  at the filter is below B′, x settles complete in the same batch as z
  (or had settled earlier).
- *Label-refused* (`cand ≤ d[x]` fails): x's label was already < the
  candidate — for an edge of an lh-canonical path the refusal-floor
  lemma (§3) applies: x was already complete. **Proviso:** this face
  reads the label clause only; a bound-refused offer implies nothing
  about d[x].
- *Bound-refused* (`cand < B(C)` fails): the face **self-cancels**.
  The W′ sweep re-offers the bit-identical candidate ((S1)/(S2): z's
  prefix is pinned, the candidate's pred slot is z's id) at the same
  bound B(C) — the pop's relax site and the sweep's relax site pass
  the same call's b (922, 1438) — so the sweep's relax is bound-refused
  too: z's settlement never drops x at the sweep, and no sub-B′ drop
  arises through this face at all.

In all cases: every successor of z along a canonical-path edge whose
offer clears the bound is complete at or before z's settlement batch,
and a bound-refused edge produces no drop to certify. The corollary is
what the refused-echo corner actually needs: when a later echo of z's
settlement would drop x₁ below a bound with every insert gate refusing,
either x₁ was already complete (label face) or the drop never fires
(bound face) — there is nothing left for custody to certify, and the
frontier argument re-fires one vertex further along the path. At the
corollary's consuming instance inside the violator geometry (§8 case
A), the bound face is doubly closed: the strict chain of §3 shows the
canonical candidate clears every echo bound there.

*Audit note.* The two designated attack points for the consolidation
re-derivation were (1) the exhaustiveness of the
walked-at-final/completed-later dichotomy when several searches of one
FindPivots evolve z's label — closed above by basing the dichotomy on
the last valid pop and exhausting the improver's search type, with the
two no-push corners (cap-before-pop, contact accept) handled inside
the capped/merged arm — and (2) the cascade's ground when custody
crosses a call boundary mid-grind — closed by routing every boundary
crossing through the ungated child-D merge, the line-33 hand-back, or
case (d)'s event-order induction, so the chain never dangles across a
return. One sharpening over the recorded sketch: the failed-search
attribution in Case 1 is *forced* (a final-label pop by a capped or
merged search plants proxy custody that contradicts Lemma V), not
assumed. The tuple-grade audit added the bit-identical strengthening
of case (d) and the corollary's proviso with the bound face's
self-cancellation, both re-derived from the code in the pass that
produced this revision.

## 8. The frontier-order lemma (d2)

*In the first-violation regime under V5, no W-member with an incomplete
label below B′ passes the W′ filter.*

This is the W′ case of the theorem. Suppose y passes C_e's filter with
y incomplete and d[y] < B′. Fix y's canonical path, z, x₁, τ_z as in §3.

**Case A: z settled before the event.** Let C_prev be z's settling call
and D_ca the deepest common ancestor of C_prev and C_e's seeding in the
call tree. Each loop iteration makes exactly one child call, so
distinct subtrees under D_ca sit in distinct iterations: C_prev's
iteration at D_ca strictly precedes C_e's seeding (if C_prev sits
inside C_e's own subtree, D_ca degenerates to C_e and the echo below is
the same-iteration mechanism of case B.2). Settlement logs propagate to
every ancestor (§2), so D_ca's relax sweep attempted (z, x₁) at D_ca's
own bound. The echo offer is the bit-identical canonical candidate
((S1) — z settled complete strictly earlier, the regime pins its
prefix at the floor, and the candidate ignores z's pred slot), and the
bound face cannot fire (§3's strict chain: B(D_ca) ≥ B(C_e) > d[y]
full-tuple, and prefix(c₁) <_lex prefix(d[y]) through the gap). So the
echo relax accepted or label-refused — either way x₁ was complete
before C_e began (adoption or the refusal floor). Custody of x₁ is
minted by the sweep gate or by one-step complementarity — and at the
one refusing corner (z settled at a W′ boundary, every gate above
refusing) the drop lemma's corollary applies instead: x₁ was complete
already at z's settlement batch, or the drop never fired. The frontier
advances one path position and the argument re-fires at x₁ — a
strictly larger integer measure, so the settled-z geometry
self-destructs into case B.

**Case B: z unsettled at the event.** By Rep and the anchor chain, z
has live custody ≤ τ_z. By W1's suspended floor and D1–D3 it cannot
idle in an ancestor or an un-re-armed group: at the event it is a live
entry in C_e's own ds, value ≤ τ_z. Two endings:

1. **The entry survives to the filter.** By Lemma V's mechanics it then
   survived the last pull or rode the final child's returned D; both
   pin B′ ≤ value ≤ τ_z < d[y] — the strict step prefix-decided
   (lh(z) <_lex lh(y) <_lex prefix(d[y]), the workhorse) — so y
   *fails* the filter (which demands d[y] < B′). B′ cannot clear the
   stale label without first clearing the custody below it: the
   interval that B′ would have to enter to pass a stale y is empty.
   (This is R4's recorded geometry: B′ = 50.4375 ≤ dis(z) = 50.5
   < d[y] = 51.0625, the stale y refused.)
2. **The entry is pulled.** The pulled head settles complete in the
   iteration's child (the regime), and the settlement echoes at C_e's
   own relax sweep *in the same iteration*, before the loop-top check,
   at C_e's wide bound B — which every chain candidate clears
   full-tuple (candidate prefixes sit ≤_lex lh(y) <_lex prefix(d[y]),
   and d[y] < B; the workhorse). Each consumption therefore advances
   the true-path frontier at least one vertex — by adoption of the
   bit-identical canonical candidate or by the refusal floor — and
   re-mints custody at the new frontier (sweep gate, one-step
   complementarity riding the child's returned D through the ungated
   merge, the re-base face at a drifted target (§5), or — at the
   entry-less corner — the drop lemma's co-settlement). The frontier's
   value sits below every separator that could pass y, so it re-enters
   every next pull as part of the block minimum; heads pulled without
   settling hand back by line 33 or the returned D at unchanged value.
   The loop is finite, so at exit either the cascade passed y — y
   complete before the filter, no violation (row R3's recorded face) —
   or a frontier entry is live at exit, which is ending 1 at the
   advanced frontier.

Both endings contradict the violating pass. ∎

## 9. The theorem

**Theorem (H_lh | V5).** On the V5 engine, every settlement event
settles a complete vertex — the settled label carries the canonical
(len, hops) pair. In particular (the len slot), every settlement is at
true shortest-path distance.

*Proof.* Induct on settlement batches; suppose the first violating
batch exists and pick y, z, x₁, τ_z as in §3.

If the event is a W′ append, Lemma d2 (§8) already forbids it.

If the event is a base pull with governing value β (the pulled
minimum): z complete means z's completing relax happened at one of the
four sites, and the channel inventory of §6 leaves branch (i) custody,
branch (ii) z settled, or branch (iii) z popped at final label by the
event call's own search — but a base call runs no searches, so (iii)
here means z's final-label pop happened in some call's FindPivots whose
custody or settlement consequences land in (i)/(ii) anyway. Branch (i):
a live value ≤ τ_z < β (the strict step prefix-decided through the
incompleteness gap, §3) sits in the base D itself — contradicting the
pull's minimality (W3) — or in an ancestor — contradicting W1's floor.
Branch (ii): z settled strictly earlier, so its settlement echoed at
the deepest common ancestor of its settling call and the event call's
seeding (or at the event call itself: a base call re-relaxes every
settled vertex's edges at its own pull, 1085–1094, ungated insert)
with the bound face closed as in case A of §8 — x₁ complete before the
event, by adoption or the refusal floor, and the argument re-fires at
x₁, a strictly larger path position. The measure is finite; no first
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
| `rowj.txt` | The pred slot's recorded faces (row J, predictions first; addendum 27's predicted settle ≠ final flip corrected to *unassemblable* at design time — addendum 30, cold-audited addendum 31): J1, the arrival-set face — a diamond whose canonical pred is pre-labeled outside S never fires its offer, and y settles at the arrived pred with settle = final on all three engines, every counter 0; J2, the arrival-order face — S2's pre-settlement pred-only tie-break accept lands and y settles already canonical; part 2, the full recursion heals J1's artifact on all engines. Face (b), the lh-oracle: a hops-aware Dijkstra against final labels, four families × three engines — reach/len/hops/pred mismatches all zero in all 12 rows. |

Corpus-wide, every V5 row of every recorded family shows
`settle-inc = 0` and `viol base/rec = 0/0` (wprime, rowh, rowi,
triangle, escort, escape, stale_incomplete — see
`benchmarks/results/`); the only nonzero settle-inc rows anywhere are
row G's and row H's paper/V8 rows. Under the theorem this is no longer
an empirical observation but the predicted face of Lemmas V and d2.

One honest gap between theorem and record: the instrumentation is
length-grade — both settle-inc counters compare `.len` only (1059,
1366) — so the counter rows above measure the len-slot projection. The
hop slot's content (no settlement at a correct length with a
non-minimal hop count) has no recorded counter; row J's lh-oracle face
(`rowj.txt`, above) measures it driver-side — zero (len, hops)
mismatches corpus-wide on all three engines, the theorem's predicted
face for V5. The bound-clause counters, by contrast, compare full
tuples (1453): `viol base/rec = 0` measures exactly the clause Lemma V
consumes.

## 11. Grade, limitations, and open items

- **(len, hops) grade; the pred slot is scope, not debt — and the
  scope is sharp (§12).** "Complete" compares the label's (len, hops)
  prefix to the canonical pair lh(v). The pred slot is excluded from
  *completeness* by design: prefix optimality fails there (the
  min-pred tie-break at a vertex constrains nothing about the
  predecessor's own pred), and settlement-time pred is the min id
  among the tight offers that *arrived* before settlement — gate
  geometry, not canon (row J1's recorded exhibit: a diamond whose
  canonical pred is pre-labeled outside S settles at the arrived
  pred, all three engines identical). What the exclusion is *not*:
  drift. §12 proves settlement labels are bit-final — the flip an
  earlier revision of this bullet described (settlement-time pred ≠
  final pred via post-settlement drift) is unassemblable on V5, and
  in full runs the settled pred is canonical outright. Pre-settlement
  pred drift (S2's tie-break accept) stays live and harmless ((S2):
  candidate-invisible; row J2's exhibit). The differential gate pins
  determinism (port = reference, bit-exact), not pred canonicality.
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
  addenda 16–31: eight derivation/audit pairs; the §7 lemma re-derived
  in the consolidation pass; the tuple-grade upgrade derived and
  cold-audited as its own pair, with its two audit-assembled closures
  — the bound face's strict chain and self-cancellation, and the
  cascade's re-base face — re-derived from the code in the lh-grade
  revision; the bit-finality closure derived at row J's design time,
  cold-audited as the eighth pair, with its two audit-assembled
  closures — Lemma T and the delivery induction — re-derived from the
  code in the pass that produced §12). Not machine-checked.

## 12. Bit-finality of settlement labels

The theorem pins the settled (len, hops) prefix; this section pins the
whole tuple. The closure was found at row J's design time — the drift
probe the tuple-grade session had designed turned out to be
unassemblable — and cold-audited (QUESTIONS.md addenda 30/31); the two
lemmas that audit assembled, T and the delivery induction inside
Theorem BF, were re-derived from the code in the pass that produced
this section, per the cadence rule. Two facts of raw code structure
carry everything:

- **(Stack locality.)** Every block D is a local of an active call
  (`ds`, 1040/1141); a returning call's D merges into its parent
  synchronously (1212); the top call's D drains (the totality assert,
  1544). So the live Ds at any settlement event are exactly those of
  the settling call's ancestor chain: any live entry below the
  settling value is co-resident with the settling vertex's descent at
  their common level, and "the first context whose bound covers a
  candidate" is always an ancestor of the settling call.
- **(Value-prefix pulls.)** A pull (327–344) removes the M smallest
  stored values — all values distinct (the keep-min asserts, 307/313),
  so pulled values sit strictly below the separator — and any
  co-resident entry below a pulled value is in the same pulled prefix.
  The base D is the M = 1 case (1040): a pure value-order drain.
  Pivot proxies are compatible: the pivot is the argmin of its P_j at
  selection and re-selection (1147–1153, 1284–1291), so proxy custody
  sits at or below every member it represents, and a member expanded
  into a block (d < b_i, 1195) never outruns a smaller direct entry
  or a smaller proxy chain.

**Lemma T (every reached vertex settles).** *In a completed full run —
the `sssp_run` shape: top call at B = Inf from the source label alone
(1532–1538), the totality assert passing (1544) — on the V5 engine in
the first-violation regime, every vertex that ever holds a finite
label appears in the settlement log at least once.*

*Proof.* Induct on the tuple order of current labels ((Z) makes every
regress below tuple-strictly descending, so the induction is
well-founded). Three steps.

*(1) A labeled vertex holds a live entry or settles.* The label was
written by an accept at one of the four relax sites, or is the source
seed (custody at 1043–1045). The base accept inserts ungated
(1086–1088). A sweep accept whose gate passes inserts (1250, 1439); a
gate-refused sweep accept lands in audited machinery — at the
iteration sweep, one-step complementarity (§5: the bit-identical
candidate was accepted inside that iteration's child under the child's
own bound, and the first accepting level fired its own insert,
grounding at the ungated base insert or the W′ face); at the W′ sweep,
a refused gate means d < B′ after the accept, and the drop lemma turns
the corner into a settlement — the sweeping source was popped at its
final label by a failed search that relaxed the same edge with the
bit-identical candidate under the same bound (§7), so the target had
joined that search's K ⊆ W and passes that call's own filter. The one
insert-free site is the FindPivots relax (922). A prior entry, at
however stale a value, still serves: keys leave a D only by pulls, and
seeds re-read current labels (1044; a child's S is a key list). A
vertex whose entire label history is FindPivots accepts — every
non-root K-member joins by an accepting relax (922–932), so this face
includes W-members outside the call's S — regresses through its
accept's source: the source is tuple-strictly below ((Z)), so by the
induction it settles; its settlement is complete (the regime), the
candidate is bit-identical from then on ((S1)/(S2)), and the
settlement echo re-fires the tie — an exact-tie re-store is an accept
at 714's ≤ — at echo sites that do insert (the settling call's own
site, then each ancestor sweep on the return path, with gate refusals
closed by one-step complementarity as above), minting the vertex's
custody at the first covering level. The top call's bound is Inf, so a
covering level always exists.

*(2) Every entry is eventually pulled.* Entries leave a D only through
`pull`; keep-min rewrites values in place (300–316), merges relocate
entries upward intact (321–325), every non-top D merges into its
parent at the return (stack locality), and the top D drains (the
assert). No entry outlives the run unpulled.

*(3) A pulled member settles or hands a live entry back.* Pulled keys
and expanded members form the child's seed set (1186–1204); a
represented member whose pivot is pulled either descends with it
(d < b_i — proxy upgraded to identity) or keeps representation through
the ungated re-selection at the new argmin (1275–1299, Rep §6).
Induct on level: base pulls settle unconditionally (1048–1055; no V8
skip on this engine), and a base cutoff exit returns its remainder
intact — postponement, not loss (merged at 1212, re-pulled later). At
a bmssp level, a seed v that does not settle inside the child C_c is
unsettled at C_c's return with d[v] < B(C_c) (seeded strictly below
the child's bound — pulled values sit under the separator and expanded
members under b_i, both strict; labels only improve). d[v] < B′ is
impossible: Rep would put a live entry ≤ d[v] < B′ in C_c's Ds at the
return — all merged into C_c's own ds by then — contradicting Lemma V.
So B′ ≤ d[v] < B(C_c) and line 33 (1334–1335) re-inserts v: a live
entry handed back and merged up (1212).

Every consumption of v's entry therefore settles v or leaves a live
entry for v; at the run's end no live D exists (2), so the last
consumption settled v. ∎

*Scope note (the paper face).* On the paper engine the same proof runs
wherever Lemma V's conclusion holds — and first-violation-freeness
alone does *not* supply it there: `escape.txt` is the recorded witness
(settle-inc = 0 with `viol_bound_rec` = 366). The paper face of this
whole section is therefore conditioned on the **zero-counter record**
proper — settle-inc *and* viol base/rec all zero — which is the
recorded state of every family row J measures; the viol counters
compare full tuples (1108, 1453), so they measure exactly the clause
consumed. This is the precise form of the audit's scope map.

**Reach totality (remark).** In a completed full run every
graph-reachable vertex of the reduced graph is reached: a reached
vertex settles (Lemma T), every settlement enters the top call's log
(concatenation at 1319, W′ append at 1450), and the top call's relax
sweep fires the settled vertex's out-edges at B = Inf (1246–1272) —
the bound clause is mute, so every out-neighbor acquires a finite
label. Induction along any path from the source.

**Theorem BF (bit-finality of settlement labels).** *In the
first-violation regime under V5, the label a vertex carries at its
first settlement event is bit-final: no later accept strictly improves
any slot, pred included. On V8 the claim is raw code — the dedup gate
(701–706) refuses every post-settlement relax. On the paper engine it
holds under the zero-counter record above.*

Post-settlement pred drift therefore does not exist; the pred slot's
non-canonicality at settlement (§11) is purely an *arrival-set*
property — which tight offers fired before the event — never a drift
property. Pre-settlement drift (S2's tie-break accept) remains live
and harmless.

*Proof.* Let v settle at label L with governing bound β — the pulled
stored value at a base pull (β = L, with W3/W1 giving the floor) or B′
at a W′ append (L < B′, with Lemma V pinning the filter moment). In
both cases L ≤ β and Lemma W forbids any live stored value < β at the
event — in particular any value < L. The settlement is complete (§9), so
prefix(L) = lh(v), and by (Floor) a later strictly-improving accept
into v carries a candidate c with prefix(c) = lh(v), the same curr,
and a smaller pred: c < L full-tuple.

Suppose some strictly-improving accept with candidate tuple-below L
fires after v's event, into any target; let a₀ be the first in program
order, with source u₀ and candidate c₀ < L. (If the alleged violation
into v exists, this set is non-empty.) Exact-tie re-stores are no-ops
and are excluded by "strictly-improving"; they cannot violate
bit-finality. The accept that wrote u₀'s current label carried a
candidate equal to that label, tuple-strictly below c₀ ((Z)), and
precedes a₀ — by a₀'s firstness it is pre-event, or u₀ is the source
vertex. So u₀'s sub-L label predates v's event.

*u₀ settled before the event.* Every custody form a sub-L label can
take is itself sub-L: identity mints and line-33 hand-backs store
current labels, keep-min only lowers, and complementarity/regress
mints (Lemma T, step 1) store accepted candidates ≤ the label. At the
event no live value < L exists (above). So if u₀'s pre-event label
came with custody, that custody was consumed pre-event, and Lemma T's
step 3 makes every consumption settle u₀ or hand back live sub-L
custody — the last pre-event consumption settled u₀. If u₀'s label
history is FindPivots-only, regress through its source (tuple-smaller,
sub-L, pre-event by the same firstness argument): by induction on the
tuple order the source settled pre-event, and its settlement echo's
tie re-store minted u₀'s custody at the first covering level — and it
did so *pre-event*: the echo fires inside the source's own settling
iteration at each level of the return path, and the first covering
level sits at or below the deepest common ancestor with the event call
(whose bound exceeds L — next paragraph); an earlier iteration there
completes before the event's seeding, and the same-iteration face
descends into the co-descent ground below. Custody pre-event reduces
to the settled case.

*Delivery.* u₀ settled pre-event, complete (the regime), so from its
settlement on every (u₀, x) candidate is bit-identical ((S1)/(S2)).
Let D† be the deepest common ancestor of u₀'s settling call and the
event call (possibly the event call itself). Every bound on the event
stack exceeds L — settled labels sit strictly below every bound of
their descent (W1's nesting and the per-site gates) — so
B(D†) > L > c₀. Two cases:

- *Earlier iteration.* If u₀'s iteration at D† strictly precedes the
  event's seeding iteration, D†'s relax sweep fired every out-edge of
  u₀ at B(D†) when u₀'s subtree returned (u_log concatenation
  1319/1450 — W′ settlers enter the log before the return — and the
  sweep at 1241–1272), strictly before the event's seeding pull
  (in-iteration order: the sweep at 1241 precedes the loop's next pull
  at 1177). The bit-identical offer c₀ cleared the bound
  (c₀ < L < B(D†)), so it was accepted or label-refused then:
  d[x₀] ≤ c₀ pre-event.
- *Same iteration.* If the two share an iteration at D†, descend into
  the shared child — the level strictly decreases — grounding at a
  shared base call, where the base D drains in pure value order
  (M = 1): u₀'s sub-L custody was pulled before the event's L-value
  entry, u₀'s in-base settlement swept (u₀, x₀) at the base bound
  B > L > c₀ with the insert ungated (1085–1094), before the event's
  pull.
- *Same W′ batch.* If the event is a W′ append and u₀ co-settles in
  the event's own batch, the drop lemma supplies the offer: u₀ was
  popped at its final label by a failed search of the event call's
  FindPivots, which relaxed (u₀, x₀) at the call's full bound then
  (§7 and its remark — u₀'s label is bit-final from that pop), before
  the append.

In every case d[x₀] ≤ c₀ already held at the event, so a₀ does not
strictly improve — contradicting its choice. No post-event
strictly-improving accept below L exists; in particular none into v.
The settled label is bit-final. ∎

**Corollary (pred canonicality of final labels; paper/V5 full runs).**
*In a completed full run in the regime (V5; paper under the
zero-counter record), every reached non-source vertex's final label
carries the canonical pred: the minimum id among its tight
predecessors — sources u of edges (u, v, w) with
lh(u) + (w, 1) = lh(v).*

*Proof.* Final prefixes are canonical: every reached vertex settles
complete (Lemma T, §9) and labels never move below the floor. Stored
preds are tight: the accept that wrote the final label built its
candidate from the pred's write-time label (708–713), and a strictly
cheaper genuine prefix at the pred would extend along the edge to a
genuine path below lh(v) — contradicting lh-minimality — so the
pred's write-time prefix was canonical and the edge tight; S2 drift
moves only pred slots and never enters candidates, so tightness is
permanent. Hence the final pred is a tight pred: final pred ≥ p*, the
minimum tight id. Conversely p* is reached (reach totality), settles
(Lemma T), and the top call's sweep fires (p*, v) at B = Inf with the
canonical candidate c* = ⟨lh(v), v, p*⟩. A label-refusal would need
d[v] < c*: equal prefix (Floor + canonicity), equal curr, pred < p* —
impossible, the pred is tight. So the relax accepts (strict or tie)
and d[v] ≤ c* with pred ≤ p* thereafter; combined with ≥, the final
pred is p*. And by Theorem BF the settlement label *is* the final
label, so in full runs the settled pred is canonical too. ∎

Not claimed for V8: the dedup gate freezes labels at settlement, so a
late-arriving pred-healing accept is refused — a frozen non-canonical
pred is within V8's distance-exact contract (recorded zero on the row
J corpus, but recorded, not predicted).

*Scope map.* What this section consumes beyond raw engine mechanics:
Lemma V and Lemma W (the no-sub-L floor at events), the §7 drop lemma
and its remark (the W′ faces), §5's one-step complementarity, §6's
Rep, and (H_lh | V5) itself (settlements complete, prefixes pinned).
All are regime-conditional exactly as stated; on V8 bit-finality alone
survives, as raw dedup code. Row J1's exhibit marks the boundary of
the corollary, not of the theorem: in a *partial* call shape (a
driver-seeded direct call with the canonical pred pre-labeled outside
any custody) the settlement label is still bit-final — settle = final
on all three engines, as recorded — but Lemma T's premise fails for
the pre-labeled pred, its offer never fires, and the settled pred is
the min id among *arrived* tight offers. The full-recursion part of
row J records the corollary's face: final labels canonical including
pred, on all three engines.
