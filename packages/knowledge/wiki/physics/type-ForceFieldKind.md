<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-ForceFieldKind"></a>

### `ForceFieldKind` (enum)

How an og.ForceField accelerates.

A string: one of the variants below. The zero value is `"directional"`.

| Variant | Doc |
|---|---|
| `"directional"` | Along its `direction`: an updraft, a jump pad, a low-gravity room (up, against gravity). |
| `"radial"` | Away from the entity, or towards it with a negative `strength`: a gravity well, a magnet, a blast. |
| `"vortex"` | Around its `direction` through the entity, counter-clockwise seen against it (from above for the default): a whirlpool, a tornado. |

```ogt
enum ForceFieldKind {
  doc "How an og.ForceField accelerates."
  variant directional "Along its `direction`: an updraft, a jump pad, a low-gravity room (up, against gravity)."
  variant radial "Away from the entity, or towards it with a negative `strength`: a gravity well, a magnet, a blast."
  variant vortex "Around its `direction` through the entity, counter-clockwise seen against it (from above for the default): a whirlpool, a tornado."
}
```

<sub>Module `physics` line 155</sub>
