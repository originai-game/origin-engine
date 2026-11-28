<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-AuthoredTarget"></a>

### `AuthoredTarget` (struct)

A development source-edit target accepted by og set and Stage. Inherited fields resolve to their defining source; computed declarations refuse edits with their location.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scene` | `string` | `""` | — | — | — | Canonical scene or prefab content ID. |
| `entity` | `string` | `""` | — | — | — | Expanded entity path accepted by og set within that declaration. Pass it unchanged; it is not a runtime probe ID. |
| `component` | `string` | `""` | — | — | — | Authored component name. |
| `field` | `string` | `""` | — | — | — | Authored field name. |

```ogt
struct AuthoredTarget {
  doc "A development source-edit target accepted by og set and Stage. Inherited fields resolve to their defining source; computed declarations refuse edits with their location."
  field scene String "Canonical scene or prefab content ID."
  field entity String "Expanded entity path accepted by og set within that declaration. Pass it unchanged; it is not a runtime probe ID."
  field component String "Authored component name."
  field field String "Authored field name."
}
```

<sub>Module `probe` line 44</sub>
