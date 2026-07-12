"""Differential-test driver: Python reference vs the Rust port.

Invoked by the Rust integration test `tests/differential.rs` as

    python diff_driver.py <results_file> <seed> [<seed> ...]

The results file holds, per seed, the Rust port's chosen parameters, final
label array over the reduced graph, settlement log (the top-level call's U
list), and readback lengths (f64s as raw bit patterns in hex). This driver
regenerates the identical graphs from the seeds (the generator below is a
draw-for-draw mirror of ``gen_diff_graph`` in ``tests/common/mod.rs``), runs
the pure-Python reference, and compares token-for-token. On divergence the
first differing index is printed with context.

Unlike the first project's harness there is nothing to pin: the algorithm
draws no random numbers, and the reference observes no hash order (SPEC.md
"Determinism pins"), so the comparison needs no monkeypatching.
"""

from __future__ import annotations

import importlib.util
import struct
import sys
from pathlib import Path

MASK64 = (1 << 64) - 1


class SplitMix64:
    """Mirror of `common::SplitMix64`."""

    __slots__ = ("state",)

    def __init__(self, seed: int) -> None:
        self.state = seed & MASK64

    def next64(self) -> int:
        self.state = (self.state + 0x9E3779B97F4A7C15) & MASK64
        z = self.state
        z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & MASK64
        z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & MASK64
        return z ^ (z >> 31)


def gen_diff_graph(seed: int):
    """Draw-for-draw mirror of tests/common/mod.rs::gen_diff_graph."""
    r = SplitMix64(seed ^ 0xB0D1260207868AAA)
    tie_heavy = seed >= 1000
    cls = seed % 4
    if tie_heavy:
        if cls in (0, 1):
            n = 2 + r.next64() % 120
        else:
            n = 50 + r.next64() % 1500
        max_m = 6 * n
    else:
        if cls == 0:
            n = 1 + r.next64() % 40
        elif cls == 1:
            n = 2 + r.next64() % 459
        else:
            n = 500 + r.next64() % 4501
        max_m = 3 * n
    m = r.next64() % (max_m + 1)
    edges = []
    for _ in range(m):
        u = r.next64() % n
        v = r.next64() % n
        if tie_heavy:
            w = (r.next64() % 8) / 8.0
        elif r.next64() % 20 == 0:
            w = 0.0
        else:
            w = ((r.next64() % 1_000_000) + 1) / 1e6
        edges.append((u, v, w))
    source = r.next64() % n
    override = (3, 2, 2) if seed >= 2000 else None
    return n, edges, source, override


def f64_bits(x: float) -> int:
    return struct.unpack("<Q", struct.pack("<d", x))[0]


def load_reference():
    here = Path(__file__).resolve()
    ref_path = here.parents[1] / "reference" / "reference.py"
    spec = importlib.util.spec_from_file_location("_reference_under_test", ref_path)
    mod = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = mod
    spec.loader.exec_module(mod)
    return mod


def label_tok(lab, label_inf) -> str:
    """Mirror of differential.rs::label_tok. A partially-infinite label (a
    state the Rust port's Label enum cannot represent) serializes to a token
    Rust never emits, so the modeling assumption itself is gated."""
    if lab == label_inf:
        return "I"
    ln, hops, curr, pred = lab
    return f"{f64_bits(ln):x}:{hops}:{curr}:{pred}"


def parse_results(path: Path, seeds):
    """Parse the Rust results file into {seed: (ov, params, labels, settle, dist)}."""
    results = {}
    lines = path.read_text().splitlines()
    i = 0
    while i < len(lines):
        head = lines[i].split()
        assert head[0] == "GRAPH", lines[i]
        seed = int(head[1])
        ov = tuple(int(x) for x in head[3:6]) if len(head) > 2 and head[2] == "OV" else None
        params = tuple(int(x) for x in lines[i + 1].split()[1:])
        labels = lines[i + 2].split()[1:]
        settle = [int(x) for x in lines[i + 3].split()[1:]]
        dist = [int(x, 16) for x in lines[i + 4].split()[1:]]
        assert lines[i + 5] == "END", lines[i + 5]
        results[seed] = (ov, params, labels, settle, dist)
        i += 6
    assert sorted(results) == sorted(seeds), (len(results), len(seeds))
    return results


def first_mismatch(a, b):
    for i in range(min(len(a), len(b))):
        if a[i] != b[i]:
            return i
    if len(a) != len(b):
        return min(len(a), len(b))
    return None


def context(name, py, rust, idx, width=3):
    rows = [f"  {name} differ first at index {idx} (python has {len(py)}, rust {len(rust)}):"]
    for j in range(max(0, idx - width), min(max(len(py), len(rust)), idx + width + 1)):
        p = py[j] if j < len(py) else "<ended>"
        r = rust[j] if j < len(rust) else "<ended>"
        marker = " <-- first divergence" if j == idx else ""
        rows.append(f"    [{j}] python={p} rust={r}{marker}")
    return "\n".join(rows)


def main() -> int:
    results_path = Path(sys.argv[1])
    seeds = [int(s) for s in sys.argv[2:]]

    ref = load_reference()
    rust = parse_results(results_path, seeds)
    failures = 0

    for seed in seeds:
        n, edges, source, override = gen_diff_graph(seed)
        rust_ov, rust_params, rust_labels, rust_settle, rust_dist = rust[seed]
        assert rust_ov == override, (seed, rust_ov, override)

        lengths, labels, settle, params, n2 = ref.sssp_instrumented(
            n, edges, source, override
        )

        py_params = (*params, n2)
        py_labels = [label_tok(lab, ref.LABEL_INF) for lab in labels]
        py_dist = [f64_bits(d) for d in lengths]

        problems = []
        if py_params != rust_params:
            problems.append(
                f"  params differ: python (delta,t,k,l_top,n2)={py_params} rust={rust_params}"
            )
        li = first_mismatch(py_labels, rust_labels)
        if li is not None:
            problems.append(context("labels", py_labels, rust_labels, li))
        si = first_mismatch(settle, rust_settle)
        if si is not None:
            problems.append(context("settlement logs", settle, rust_settle, si))
        di = first_mismatch(py_dist, rust_dist)
        if di is not None:
            problems.append(context("readback lengths", py_dist, rust_dist, di))

        if problems:
            failures += 1
            print(f"MISMATCH seed={seed} n={n} m={len(edges)} source={source}")
            for p in problems:
                print(p)
        else:
            print(
                f"OK seed={seed} n={n} m={len(edges)} n2={n2} settle_events={len(settle)}"
            )

    if failures:
        print(f"FAILED {failures}/{len(seeds)} graphs diverged")
        return 1
    print(f"ALL OK {len(seeds)}/{len(seeds)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
