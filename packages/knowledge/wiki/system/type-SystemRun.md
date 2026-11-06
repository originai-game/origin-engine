<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`system`](../system.md) · [index](../index.md)

<a id="type-SystemRun"></a>

### `SystemRun` (callback)

A system's body. It is called once per tick; loop over `q.archetypes()`, then over rows.

Game code supplies this function.

```ts
type SystemRun = (q: Query, time: Time, world: World) => void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `q` | `Query` | no | The archetypes matching the declared components. |
| `time` | `Time` | no | The Time resource for this tick. |
| `world` | `World` | no | The world, for resources, events and structural changes. |

```ogt
callback SystemRun {
  doc "A system's body. It is called once per tick; loop over `q.archetypes()`, then over rows."
  param q Query "The archetypes matching the declared components."
  param time Time "The Time resource for this tick."
  param world World "The world, for resources, events and structural changes."
}
```

<sub>`api.ogt` line 1391</sub>
