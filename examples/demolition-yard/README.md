# Demolition yard

og.destruction on four props, each a `type model` with a `fracture` block
(`assets/props/`): a brick wall and a concrete pillar standing on their
anchored chunks, a wooden crate that is a dynamic body, and a glass pane cut
into shards through its face. `src/main.ts` gives each an og.Destructible with
its material's numbers (the og-destruction skill's table) and dust, sparks
and GPU debris emitters (`assets/fx/`).

```sh
og run examples/demolition-yard --size 1280x720 --commands "1:boom;1.6:screenshot:wall.png;2.6:strike;3:drop;3.1:throw;6.5:screenshot:yard.png;6.55:quit"
```

`boom` explodes at the wall's foot (the stress solver brings down what it
held), `strike` hits the pillar's middle, `drop` lets the crate fall 4 m
onto the slab, `throw` sends an iron ball through the pane.
