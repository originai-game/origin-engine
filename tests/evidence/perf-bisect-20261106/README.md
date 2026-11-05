# Perf regression bisect, 2026-11-06 (UNFINISHED)

Status: work in progress. Nothing here is a baseline. The perf baselines are unchanged.

Method: release test binaries built per commit (og-tests' perf test, and og-runtime's
raycast_record_cost), each commit's `crates/og-tests/assets` passed as `OG_TEST_ASSETS`,
run one test at a time (`--test-threads=1`) under one `og_gpu_acquire_all` per commit,
on macrunner (Apple M4 Pro, Metal). The machine was shared: load averages were 15 to 35,
so a run whose second-best of its 7 runs is more than 6 % above the best is treated as noisy
and left out ("!" below). `bisect.tsv` holds every number (round, commit, test, best ms).

Best ms per frame (first-parent commits on main, oldest first):

| test | 96ac0c89c 10-21 | 241162645 10-23 | 0678686a2 10-24 | 70c120c0b 10-25 | 5b2558d60 10-25 | 00e47e603 11-01 | 00037cff1 11-02 | af9bed852 wave end | baseline |
|---|---|---|---|---|---|---|---|---|---|
| sprites-20k | 0.834 | 0.947 | 0.949 | 1.721 | 1.832 | 1.793 | ! | 2.215 | 1.051 |
| gltf-reference | 1.291 | 1.481 | | 1.527 | 1.525 | 1.769 | 1.848 | 2.18 | 1.385 |
| lights-256 | 0.533 | 0.551 | | 0.660 | 0.658 | 0.691 | 1.029 | 0.88 | 0.546 |
| point-spot-lights | 0.445 | 0.463 | | 0.533 | 0.541 | 0.567 | ! | 5.38 | 0.452 |
| showcase | 2.531 | 2.508 | | 3.114 | 3.065 | 3.806 | ! | 7.27 | 3.245 |
| showcase-capture | | | | 8.073 | 7.920 | ! | 17.46 (noisy run) | 12.66 | 8.108 |
| spheres-20k | 3.966 | 4.074 | | 4.361 | 4.398 | 4.690 | ! | 6.04 | 5.245 |
| spheres-20k-moving | 4.076 | 4.136 | | 4.661 | ! | 4.781 | ! | 7.81 | 5.276 |
| tilemap-256 | 0.210 | 0.218 | | 0.224 | 0.224 | 0.240 | ! | 0.27 | 0.223 |
| materials-physical | | | | ! | ! | 0.815 | ! | 1.05 | 0.828 |

Findings so far:

- sprites-20k: one step, 0.95 to 1.72 ms, between 0678686a2 (10-24) and 70c120c0b (10-25).
  dc0084761 and 0678686a2 read 0.950 and 0.949. The prime suspect in that range is
  753e8d9d7 (merge of r4/render-billboards): `SpriteInstance` gained a `facing` u32
  (vertex attribute 8 on every sprite pipeline) and every sprite now goes through
  `facing_of`, a depth key and a frustum test when faced. Built and queued but not run:
  8095d4f21, 753e8d9d7, 80143ca5c, 15ca814c4 (the merges just before and after it).
- point-spot-lights: under 0.57 ms up to 11-01. The jump to 5.4 ms is later, almost
  surely b785e540b (midpoint local shadow maps, 11-05); de7809995 (its parent) and
  b785e540b are built but not measured. That cost belongs to the local-shadow-cost thread.
- The other 3D scenes rise in several steps rather than at one commit: 241162645 to
  70c120c0b (10-23 to 10-25: +0.1 to +0.6 ms, the S6/S7/S8 render merges and the views
  pass, 186 first-parent commits); 5b2558d60 to 00e47e603 (10-25 to 11-01: showcase
  +0.7 ms, gltf-reference +0.25 ms); and after 11-01 to the wave end (most of the rest).
  00037cff1's run was noisy (load), so its column needs repeating.
- raycast-record-20k: 0.320 baseline (cf85de9b7); 0.371 at the wave end, 0.399 at
  c83fa08df under load. Not yet measured at cf85de9b7 on this machine today.
- lights-2d-64: no baseline recorded yet (1.332 ms at c83fa08df under load, not recorded).

Next steps: measure the queued sprites-20k commits; measure d30632f52, b49bb670b,
64b554ae3, 506882e93, 71724b4f8, ebb512c3a, de7809995, b785e540b, af9bed852 for every
scene; split the 241162645..70c120c0b and 5b2558d60..00e47e603 ranges for showcase and
gltf-reference; record lights-2d-64 with OG_PERF_RECORD=1 on an idle machine; then
re-baseline each deliberate cost with its commit named, or fix it.
