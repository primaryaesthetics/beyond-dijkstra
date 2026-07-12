//! Single-source shortest paths: a production Dijkstra baseline plus the
//! first implementation of the Duan–Mao–Shu–Yin directed SSSP algorithm
//! (arXiv:2602.07868, O(m·√(log n · log log n)) for sparse graphs).
//!
//! The algorithm port ([`solver`]) is a faithful mainline: it is gated
//! bit-exactly — final labels and settlement log — against the Python
//! reference in `reference/reference.py` (see `tests/differential.rs`), and
//! stays frozen once green. Research-record documents (SPEC.md,
//! QUESTIONS.md, ASSESSMENT.md) pin the semantics and track paper findings.

pub mod dijkstra;
pub mod label;
pub mod solver;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
