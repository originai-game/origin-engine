# The cluster build in compute: measured, not kept

og-render's README owed "the cluster build on compute on the full tier,
kept byte-identical to the CPU build, ... keep it only if it is faster for
256 lights". It is not faster, so the renderer keeps the CPU build on every
tier (WebGL2 needs it anyway); no compute build ships.

Apple M4 Pro, Metal, release builds, load average about 50 (other streams'
gates on the machine), og-tests' `lights-256` (243 lights in view, 16 x 9 x
24 clusters, at most 64 references each).

## The CPU build

`ClusterBuilder::build` timed in place for 500 frames (a local timer around
the call in `Renderer::render`, printed by og-gfx; not committed), three
runs: **37.9, 40.7 and 53.2 us a frame** for lights-256 (point-spot-lights,
7 lights: 43 to 64 us, the build's fixed cost of clearing and filling the
3,456-cluster table dominates). The table and references are then uploaded
with the light records the shaders read either way.

## The cheapest compute build there could be

`crates/og-tests/tests/cluster_compute.rs` (ignored; `cargo test -p
og-tests --release --test cluster_compute -- --ignored --nocapture`): the
lights' view-space spheres uploaded, one dispatch over the clusters, each
invocation testing every light against its cluster's view-space box and
writing up to 64 references (no importance order, so it is neither
byte-identical to the CPU build nor complete: a lower bound). Medians of
500 frames, three runs:

| | run 1 | run 2 | run 3 |
|---|---|---|---|
| upload, encode and submit (CPU) | 133.8 us | 105.2 us | 97.0 us |
| submit and wait, less an empty submission's (GPU and synchronisation) | 385.4 us | 332.5 us | 279.2 us |

Even without the importance order that keeps the frame identical to the
CPU build's (every reference frame at RMSE 0), the compute version costs
more CPU to record and submit than the whole CPU build, and its GPU time is
added to every lit frame's; the light records and tables must be uploaded
for the shaders in both cases. The CPU build stays; README §Forward+ and
§What phase 2 owes say so.
