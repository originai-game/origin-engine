<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-EntityPrefab"></a>

### `EntityPrefab` (struct)

Where an entity placed from a prefab came from (its og.PrefabLink, resolved) and what its instance overrides.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `prefab` | `string` | `""` | — | — | — | The prefab's asset id (the outermost one placed). |
| `node` | `string` | `""` | — | — | — | The entity's node path in the prefab: empty for its root, `lid/hinge` for a grandchild. |
| `root` | `string` | `""` | — | — | — | The instance root's probe id. |
| `overrides` | `readonly string[]` | `[]` | — | — | — | The fields its instance's text overrides on it, `Component.field`, in text order (empty for an instance placed at run time, which no text holds). |

```ogt
struct EntityPrefab {
  doc "Where an entity placed from a prefab came from (its og.PrefabLink, resolved) and what its instance overrides."
  field prefab String "The prefab's asset id (the outermost one placed)."
  field node String "The entity's node path in the prefab: empty for its root, `lid/hinge` for a grandchild."
  field root String "The instance root's probe id."
  field overrides List.String "The fields its instance's text overrides on it, `Component.field`, in text order (empty for an instance placed at run time, which no text holds)."
}
```

<sub>`api.ogt` line 24376</sub>
