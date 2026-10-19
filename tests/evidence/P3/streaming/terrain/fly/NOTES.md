# Flying the terrains (phase 3, terrain streaming)

`cd packages/engine && bun bench/terrain-fly.ts` (release native addon,
Metal, 1280x720) generates and cooks the two terrains under
`target/evidence/terrain-fly/project`, flies each corner to corner 60 m
over the ground (1200 frames over the 2 km island, 2400 over the 8 km
terrain, after 30 warm-up frames) in three configurations, each in its own
process, and writes `frame-times.json` and `frame-times.md` here. A frame
is `engine.frame(16.667)` plus `og_node_wait_gpu` (the device polled until
the frame's GPU work is done), so tick, extraction, streaming, recording
and GPU time all count. "Streamer worst ms" is the streamer's own longest
frame including the warm-up: in `resident` mode that is the first frame
decoding every wanted tile inline (15.6 ms for the island, 23.6 ms for the
8 km terrain), which streaming spreads over worker threads (at most
0.12 ms a frame).

- The 2 km island fits the 16 MiB budget whole (85 tiles, the full
  quadtree), so after the warm-up nothing streams; at 4 MiB it holds 26
  tiles and swaps 59 through the cache.
- The 8 km terrain streams in every mode: 473 tile decodes over the flight
  at 16 MiB, 202 at 4 MiB, with up to 21 tiles waiting behind a fast
  camera (5 of 81 samples) while the coarser parent stays drawn.
- `8k-stream-16-per-frame.json` is the 8 km flight at 16 MiB sampled every
  frame (`--every 1`): each frame's time beside the tiles handed to the
  renderer in it. Frames handing 0 tiles: median 1.99 ms; 3 tiles: 2.23 ms;
  8 tiles (the per-frame cap): 3.10 ms. A handoff costs about 0.1 ms a
  tile, and the worst frame of that run is 4.7 ms.
- The PNGs are each terrain's last frame (a capture settles first, so the
  stream and resident captures are identical; at 4 MiB the 8 km capture
  differs only by its coarser tiles).

A first run on 2026-10-19 during other agents' cargo builds (load average
about 48) showed spikes only in the two `stream 16 MiB` runs (2 km: p99
25.6 ms, max 127.8 ms, CPU max 95 ms; 8 km: p99 21.5 ms, max 46.9 ms).
They did not repeat on three reruns at load average 11 to 20, the 2 km run
streams nothing after its warm-up, and the per-frame run shows no frame
over 4.7 ms with any handoff, so they were the machine's load, not the
streamer. The tables here are the quiet rerun.
