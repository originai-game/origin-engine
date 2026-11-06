<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`system`](../system.md) · [index](../index.md)

<a id="type-RunCondition"></a>

### `RunCondition` (callback)

Whether a system runs this tick: called before its run, after the systems before it, with the system's reads and writes as read-only access. It may not change the world.

Game code supplies this function.

```ts
type RunCondition = (world: World) => boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world, read only. |

```ogt
callback RunCondition {
  doc "Whether a system runs this tick: called before its run, after the systems before it, with the system's reads and writes as read-only access. It may not change the world."
  param world World "The world, read only."
  returns bool
}
```

<sub>Module `system` line 24</sub>
