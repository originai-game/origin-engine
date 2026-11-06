<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`destruction`](../destruction.md) · [index](../index.md)

<a id="type-DestructionCapabilities"></a>

### `DestructionCapabilities` (struct)

What destruction does on this device.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `visualDebris` | `boolean` | `false` | — | — | — | Whether og.Destructible.debris pieces and bursts are simulated and drawn on the GPU here (a renderer with compute passes: og.render.capabilities' compute); false draws none of them. The bodies are the same either way: pieces given to `debris` are never bodies, on any device, so a game never depends on it. |
| `maxVisualDebris` | `number` | `0` | — | — | — | The most fine pieces the GPU layer holds at once: past it the oldest go first (og.Destruction.debrisLifetime takes them sooner). |
| `visualDebrisContacts` | `string` | `""` | — | — | — | What the GPU pieces collide with: `world` (the colliders that are boxes, spheres and terrains); they pass through each other and through the chunks that are bodies. |

```ogt
struct DestructionCapabilities {
  doc "What destruction does on this device."
  field visualDebris bool "Whether og.Destructible.debris pieces and bursts are simulated and drawn on the GPU here (a renderer with compute passes: og.render.capabilities' compute); false draws none of them. The bodies are the same either way: pieces given to `debris` are never bodies, on any device, so a game never depends on it."
  field maxVisualDebris u32 "The most fine pieces the GPU layer holds at once: past it the oldest go first (og.Destruction.debrisLifetime takes them sooner)."
  field visualDebrisContacts String "What the GPU pieces collide with: `world` (the colliders that are boxes, spheres and terrains); they pass through each other and through the chunks that are bodies."
}
```

<sub>`api.ogt` line 27344</sub>
