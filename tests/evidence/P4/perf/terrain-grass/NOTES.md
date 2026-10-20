# vulkan-cpu `terrain-grass`: a stale reference

docs/phase-1.md (culling round, "Not done"): og-tests' `terrain-grass` failed
on vulkan-cpu at RMSE 0.01488 on main, before that round.

How: og-gfx cross-built at this branch (`cargo zigbuild --release --target
x86_64-unknown-linux-gnu.2.31 -p og-tests --bin og-gfx`, after merging main
7696db4), run in the og-lavapipe image (Mesa llvmpipe, LLVM 19.1.7) with the
worktree mounted read-only: `og-gfx --all --capture` (`og-gfx-vulkan-cpu-all.txt`).
Frames compared with og-tests' rule (RMSE, channel difference > 24 of 255)
in `comparisons.txt`; `terrain-grass-vulkan-cpu.png` puts them side by side
(each halved).

| frame | against | RMSE |
|---|---|---|
| vulkan-cpu reference (recorded at 6d9974c) | metal reference before 1ced372 | 0.00537 |
| vulkan-cpu reference | metal reference now | 0.01499 |
| lavapipe at this branch | metal reference now | 0.00714 |
| lavapipe at this branch | vulkan-cpu reference | 0.01488 |
| metal before 1ced372 | metal now | 0.01416 |

Cause: 1ced372 (streaming round, "Scatter") made scattered copies fade by a
dither at full size instead of shrinking and re-recorded only the metal
reference. The vulkan-cpu reference still showed the shrinking tufts, and
lavapipe's frame of the new scatter matches metal's new frame as closely as
the other tiers match each other. Not a lavapipe sampling difference and not
an engine defect: the reference was stale.

Re-recorded: `crates/og-tests/references/vulkan-cpu/terrain-grass.png`
(the meadow's tufts at full size thinning towards the 60 m edge, the three
boulders and the range behind, as on metal). A second lavapipe run matches it
at RMSE 0. Every other vulkan-cpu scene passes (the largest RMSE, terrain-near
0.00647); none was re-recorded.
