# World-placement replay

`world-placement.zlib` preserves 12,000 consecutive placement queries (calls 6000–17999) from `perf_hud_2000_mass_and_100_anchors_within_a_millisecond` at `f48aa184608cb8c1002169cefa9ad60f5d68ce21`. It contains 8,465 anchor queries and 3,535 mass-number queries. The committed revision's unmodified solver supplies the expected results, including the precise floating-point offsets and its 3,168 failed mass queries.

The replay protects candidate preference, screen containment, and f32 ties while the placement implementation changes. The test also reconstructs each append-only spawn batch and checks that continuous free-space certificates never reject a query the original discrete solver accepts. A failed discrete query alone is not a reusable certificate: a later obstacle can introduce new candidate coordinates.

The zlib stream expands to concatenated records, with no header. All words are little-endian; floats are stored as their original IEEE-754 binary32 bits:

1. One byte: declutter mode (`None=0`, `Shift=1`, `Stack=2`, `Hide=3`), with bit 4 set for mass queries.
2. Ten f32s: query rectangle (`x,y,width,height`), gap, reach, and containment rectangle (`x,y,width,height`).
3. One u32 obstacle count, then that many rectangles (four f32s each).
4. One byte indicating whether the original solver returned an offset; if set, two f32s containing that offset.

Do not refresh the expected results from the optimized implementation. A deliberate placement-contract change requires reviewing the new behavior independently, not accepting a regenerated golden because a test fails.
