# anim-markers

The walker (`assets/models/walker.glb`, the cook-assets fixture's rig,
pinned in `og-assets.json`) on two graphs, both fading over one second from
tick 30: `fade` from its walk into its run, `blend` from its wave into an
even blend of walk and run (three clips at once). `src/main.ts` checks every
og.AnimationEvent against the weight it carries.

What it checks (`tests/markers.og.ts`, `og test tests/fixtures/anim-markers`):

- every marker names its state (0 the state left, 1 the state entered) and
  layer 0;
- in the cross-fade the heavier clip is dominant, the walk through the
  fade's first half and the run through its second, and the lighter clip's
  markers still fire;
- in the three-way fade the wave is dominant until the blend outweighs it,
  then the run: the blend's later clip wins its tie with the walk;
- two runs end on one hash.

The rule itself (one dominant clip per layer each tick, ties, players) is
og-anim's `tests/graph.rs` and `tests/playback.rs`; both hosts are compared
on it in `packages/engine/tests/anim.test.ts`.

Budget: under 10 s warm. No frames.
