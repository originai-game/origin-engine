<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`world`](../world.md) · [index](../index.md)

<a id="type-QueryDesc"></a>

### `QueryDesc` (desc struct)

Which entities a query matches. The change filters (changed, added, removed) narrow an archetype's rows to those that pass every filter (ArchetypeView.rows) and read what happened since the last tick began; a component is tracked once a system filters on it (SystemDesc.changed, added, removed), and filtering on one no system tracks is refused.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `with` | `readonly AnyComponent[]` | `[]` | — | — | — | Components an entity must have. |
| `without` | `readonly AnyComponent[]` | `[]` | — | — | — | Components an entity must not have. |
| `changed` | `readonly AnyComponent[]` | `[]` | — | — | — | Components (required, as `with`) whose value changed: a write that leaves a value as it was is not a change. |
| `added` | `readonly AnyComponent[]` | `[]` | — | — | — | Components (required, as `with`) added: spawned with, or inserted into an entity without them. |
| `removed` | `readonly AnyComponent[]` | `[]` | — | — | — | Components the entity lost (removed; a despawned entity is in no archetype, so it is in world.removed only). |

```ogt
struct QueryDesc {
  doc "Which entities a query matches. The change filters (changed, added, removed) narrow an archetype's rows to those that pass every filter (ArchetypeView.rows) and read what happened since the last tick began; a component is tracked once a system filters on it (SystemDesc.changed, added, removed), and filtering on one no system tracks is refused."
  desc true
  field with List.AnyComponent "Components an entity must have."
  field without List.AnyComponent "Components an entity must not have."
  field changed List.AnyComponent "Components (required, as `with`) whose value changed: a write that leaves a value as it was is not a change."
  field added List.AnyComponent "Components (required, as `with`) added: spawned with, or inserted into an entity without them."
  field removed List.AnyComponent "Components the entity lost (removed; a despawned entity is in no archetype, so it is in world.removed only)."
}
```

<sub>Module `world` line 188</sub>
