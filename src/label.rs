//! Distance labels: the 4-tuple ⟨length, nEdges, curr, pred⟩ of §2.3
//! (SPEC "Labels and bounds"), compared lexicographically. The unset label
//! is the +inf sentinel; the reference represents it as a whole-tuple
//! `(inf, inf, inf, inf)`, and no partially-infinite tuple is ever stored
//! (relaxation only runs from vertices with finite labels), so a two-state
//! enum models it exactly — the differential gate would catch any run where
//! that assumption failed, because such a label could not serialize equal.

use std::cmp::Ordering;

/// A finite distance label. `pred` is `-1` for the source.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Lab {
    pub len: f64,
    pub hops: u64,
    pub curr: u32,
    pub pred: i64,
}

// `len` is never NaN: edge weights are finite and non-negative, and lengths
// are finite sums of them.
impl Eq for Lab {}

impl Ord for Lab {
    fn cmp(&self, other: &Self) -> Ordering {
        self.len
            .partial_cmp(&other.len)
            .expect("label length is never NaN")
            .then_with(|| self.hops.cmp(&other.hops))
            .then_with(|| self.curr.cmp(&other.curr))
            .then_with(|| self.pred.cmp(&other.pred))
    }
}

impl PartialOrd for Lab {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A label value or the `LABEL_INF` sentinel, which is strictly greater
/// than every finite label (mirrors Python tuple comparison against
/// `(inf, inf, inf, inf)`: the finite length decides at the first slot).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Label {
    Fin(Lab),
    Inf,
}

impl Label {
    /// The finite label inside, panicking on the sentinel. Call sites rely
    /// on the invariant that the vertex has been reached; a panic here means
    /// the port diverged from the reference and must fail loudly.
    pub fn fin(self) -> Lab {
        match self {
            Label::Fin(l) => l,
            Label::Inf => panic!("expected a finite label"),
        }
    }
}

impl Ord for Label {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Label::Fin(a), Label::Fin(b)) => a.cmp(b),
            (Label::Fin(_), Label::Inf) => Ordering::Less,
            (Label::Inf, Label::Fin(_)) => Ordering::Greater,
            (Label::Inf, Label::Inf) => Ordering::Equal,
        }
    }
}

impl PartialOrd for Label {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
