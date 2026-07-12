"""Property tests: reference SSSP vs. textbook Dijkstra, bit-exact lengths.

Weight classes deliberately include zero and tiny integer domains: exact
ties and zero-weight edges are where this algorithm family breaks (see
QUESTIONS.md Q3/Q6 and the first project's tie-cascade record).
"""

import math
import os
import random
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "reference"))

from reference import dijkstra_lengths, sssp_lengths  # noqa: E402


def random_graph(rng, n, m, weight):
    edges = []
    for _ in range(m):
        u = rng.randrange(n)
        v = rng.randrange(n)
        while v == u and n > 1:
            v = rng.randrange(n)
        if u != v:
            edges.append((u, v, weight(rng)))
    return edges


def check(n, edges, s):
    expect = dijkstra_lengths(n, edges, s)
    got = sssp_lengths(n, edges, s)
    for v in range(n):
        if math.isinf(expect[v]):
            assert math.isinf(got[v]), (v, expect[v], got[v])
        else:
            assert got[v] == expect[v], (v, expect[v], got[v])


def run_class(weight, seeds, label):
    for seed in seeds:
        rng = random.Random(seed)
        n = rng.randrange(1, 41)
        m = rng.randrange(n, 5 * n + 1)
        edges = random_graph(rng, n, m, weight)
        s = rng.randrange(n)
        check(n, edges, s)


def test_float_weights():
    run_class(lambda r: r.random() * 10.0, range(150), "float")


def test_tie_rich_small_ints():
    run_class(lambda r: float(r.randrange(0, 4)), range(150, 300), "int0-3")


def test_binary_weights():
    run_class(lambda r: float(r.randrange(0, 2)), range(300, 400), "int0-1")


def test_all_zero_weights():
    run_class(lambda r: 0.0, range(400, 420), "all-zero")


def test_larger_tie_rich():
    rng = random.Random(9000)
    n = 300
    edges = random_graph(rng, n, 4 * n, lambda r: float(r.randrange(0, 3)))
    check(n, edges, 0)


def test_path_and_cycle_shapes():
    # long path (deep recursion pressure), pure cycle, star
    n = 200
    check(n, [(i, i + 1, 1.0) for i in range(n - 1)], 0)
    check(n, [(i, (i + 1) % n, 0.5) for i in range(n)], 3)
    check(n, [(0, v, float(v % 3)) for v in range(1, n)], 0)


def test_single_vertex_and_tiny():
    check(1, [], 0)
    check(2, [(0, 1, 2.5)], 0)
    check(2, [], 0)
    check(3, [(0, 1, 0.0), (1, 2, 0.0)], 0)
