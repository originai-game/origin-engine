# og-render perf against the committed baselines (M6)

`CARGO_TARGET_DIR=target/agent-qa cargo test --release -p og-tests --test perf -- --ignored perf --test-threads=1 --nocapture` on Apple M4 Pro, Metal, 2026-10-18 04:03 local (load average about 9), at 49ed935 (og-render and og-tests unmodified). Baselines: `crates/og-tests/perf/metal-apple-m4-pro/`, 15 % tolerance. 4/4 pass:

```
test perf_gltf_reference ... perf gltf-reference [metal on Apple M4 Pro]: best 1.391 ms per frame (runs: 1.391, 1.431, 1.450, 1.454, 1.492, 1.578, 1.747)
  baseline 1.385 ms, limit 1.593 ms
test perf_showcase ... perf showcase [metal on Apple M4 Pro]: best 2.012 ms per frame (runs: 2.012, 2.103, 2.355, 3.571, 3.654, 4.300, 4.307)
  baseline 3.245 ms, limit 3.732 ms
test perf_spheres_20k ... perf spheres-20k [metal on Apple M4 Pro]: best 6.080 ms per frame (runs: 6.080, 6.107, 6.291, 6.297, 6.299, 6.343, 6.620)
  baseline 6.675 ms, limit 7.676 ms
test perf_sprites_20k ... perf sprites-20k [metal on Apple M4 Pro]: best 0.575 ms per frame (runs: 0.575, 0.576, 0.615, 0.623, 0.626, 0.626, 0.628)
  baseline 1.051 ms, limit 1.209 ms
cargo test --release -p og-tests --test perf -- --ignored perf  --nocapture  2.20s user 3.11s system 58% cpu 9.088 total
```

`showcase`, `spheres-20k` and `sprites-20k` are faster than their baselines (sprites-20k 0.575 ms against the 2 ms M3 budget); nothing was re-recorded.
