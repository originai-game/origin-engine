# Tilemap draw cost (E4)

Apple M4 Pro, Metal, 1280x720, release build, on a machine shared with three
template builds and another agent's work (numbers carry the usual ±10 %).

## The budget scene: `tilemap-256` (og-tests)

A 256x256 map, four layers built by og-tilemap (`MapState`, `chunk_quads`:
16x16-cell chunks), one 128x128 nearest-sampled atlas: layer 0 full ground,
layers 1-3 at 40 %, 20 % and 10 % fill, an animated tile (3 frames at 4 fps),
layer 3 at half parallax. One unit per tile and an orthographic height of 45:
16 px tiles, 80x45 tiles in view. The camera pans diagonally every frame, so
culling, repeat placement and animated uv patching run every frame.

```
OG_PERF_RECORD=1 cargo test --release -p og-tests --test perf -- --ignored perf_tilemap_256 --nocapture
perf tilemap-256 [metal on Apple M4 Pro]: best 0.223 ms per frame (runs: 0.223, 0.223, 0.247, 0.247, 0.268, 0.286, 0.346)
  recorded crates/og-tests/perf/metal-apple-m4-pro/tilemap-256.json

cargo run --release -p og-tests --bin og-gfx -- tilemap-256 --frames 120
tilemap-256: 1280x720 tier=metal adapter="Apple M4 Pro" aa=Off draws=2 ... sprite_draws=1 ... frame=0.236ms cpu=0.229ms (120 frames)
```

0.22 ms per frame (best of seven runs of 100 frames, CPU and GPU, waited
idle), under the 1 ms budget; `perf_tilemap_256` asserts the budget on this
adapter and the baseline (15 %) everywhere. The four layers share the atlas,
so the whole map is one instanced draw (`sprite_draws=1`). Frame:
`tilemap-256-metal.png`.

## The worst case: `tilemap-256-all`

The same map zoomed out until every chunk of every layer is in view (about
170,000 quads, every one rebuilt into the instance buffer each frame):

```
tilemap-256-all: 1280x720 tier=metal adapter="Apple M4 Pro" ... sprite_draws=1 ... frame=2.581ms cpu=2.569ms (120 frames)
```

2.6 ms, CPU-bound in building and uploading instances. Retained per-chunk
GPU buffers would remove most of it; a game showing a whole 256x256 map at
once is a minimap, which should be a texture instead.

## In the runtime

The runtime (`og-api` `runtime/feel.rs`) converts og-tilemap chunks to
og-render `TileChunk`s once per chunk version and resubmits the shared
`Arc` every frame; an edit (`og.tilemap.setTile`) rebuilds only the layer
whose chunk versions moved.
