# xatlas

Jonathan Young's xatlas (MIT, `LICENSE`), vendored unchanged from
https://github.com/jpcy/xatlas at commit
`f700c7790aaa030e794b52ba7791a05c085faf0c` (`source/xatlas/`: `xatlas.cpp`,
`xatlas.h`, `xatlas_c.h`). `crates/og-bake/build.rs` compiles it with its C
API (`XATLAS_C_API=1`) and single-threaded (`XA_MULTITHREADED=0`), so a
lightmap's charts and their packing are the same on every run whatever the
machine's thread count. og-bake (`src/atlas.rs`) is its only user: the
lightmap UVs of `og bake lightmaps`.
