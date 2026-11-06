<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`transform`](../transform.md) · [index](../index.md)

<a id="record-Parent"></a>

### `og.Parent` (component)

The entity this one's Transform is relative to. Set by SpawnDesc.parent.

Token `og.Parent`, registered under the schema name `og.Parent`; its value type is `Parent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The parent. |

```ogt
component Parent {
  doc "The entity this one's Transform is relative to. Set by SpawnDesc.parent."
  field entity Entity "The parent."
}
```

<sub>`api.ogt` line 1499</sub>
