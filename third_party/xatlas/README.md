# xatlas

Jonathan Young's xatlas (MIT, `LICENSE`), vendored unchanged from
https://github.com/jpcy/xatlas at commit
`f700c7790aaa030e794b52ba7791a05c085faf0c` (`source/xatlas/`: `xatlas.cpp`,
`xatlas.h`, `xatlas_c.h`). `crates/og-bake/build.rs` compiles it with its C
API (`XATLAS_C_API=1`) and single-threaded (`XA_MULTITHREADED=0`), so a
lightmap's charts and their packing are the same on every run whatever the
machine's thread count. og-bake (`src/atlas.rs`) wraps the C API for
lightmap UVs, HLOD proxies and independently baked model LOD surfaces.

One local change, in `xatlas.cpp`'s `ClusteredCharts::mergeCharts` (marked
`origin-engine`): upstream clears and scans arrays the size of every chart
for each chart it tries, and restarts from the last chart after each merge,
which is quadratic in the chart count (about 100 s for a 50,000-triangle
LOD surface of 8,700 charts). The local version gathers a chart's
neighbours sparsely, visits them in upstream's descending order, and skips
a chart that found nothing to merge until it or a chart beside it (either
way across an edge) changes, since its decision reads only those charts.
It makes the same merges in the same order: on the 15 LOD surfaces Night
Shift cooks, the charts, atlas, UVs and indices are byte-identical to
upstream's, 4 to 20 times faster.
