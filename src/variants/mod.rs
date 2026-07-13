//! Algorithm-level variants (the research track). The mainline stays
//! frozen; each variant answers to the Tier-2 gate — bit-exact distances
//! vs the crate's own Dijkstra (`tests/variants_correctness.rs`),
//! settlement order not pinned. Claims, proof sketches, and measurements
//! live in VARIANTS.md.

pub mod fast;
pub mod no_transform;
