# Demolition yard

og.destruction on four props, each a `type model` with a `fracture` block
and the profile of its material (`assets/props/`): a brick wall tied in at
its ends that breaks along its mortar into blocks and bricks, a concrete
pillar that breaks into chunky lumps showing their gravel, a wooden crate (a
dynamic body) that splinters along its grain, and a clear glass pane that
cracks radially from where it is hit. Their textures are drawn by
`tools/textures.py`. `src/main.ts` gives each an og.Destructible with
its material's numbers (the og-destruction skill's table) and dust, sparks
and GPU debris emitters (`assets/fx/`).

```sh
og run examples/demolition-yard --size 1280x720 --commands "1:boom;1.6:screenshot:wall.png;2.6:strike;3:drop;3.1:throw;6.5:screenshot:yard.png;6.55:quit"
```

`boom` explodes at the wall's foot (the stress solver drops what hung over
the hole and leaves its two piers), `strike` hits the pillar's middle, `drop` lets the crate fall 4 m
onto the slab, `throw` sends an iron ball through the pane.
