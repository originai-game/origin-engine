<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`reflect`](../reflect.md) · [index](../index.md)

<a id="type-FieldOptions"></a>

### `FieldOptions` (desc struct)

Metadata of one field. Ranges and units are what Tune and the probe's bounds show. Nothing clamps writes: world.insert and writes from outside the game (the probe's `set`, `patch` and sweeps, Tune, a look's fields at cook) are refused outside min..max with the range named. Game code's other writes (world.set, a resource field, a column) are stored as given.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `min` | `number \| null` | `null` | — | — | — | The smallest meaningful value. |
| `max` | `number \| null` | `null` | — | — | — | The largest meaningful value. |
| `step` | `number \| null` | `null` | — | — | — | The increment Tune uses. |
| `unit` | `string` | `""` | — | — | — | A unit name from the API's unit table, or empty. |
| `label` | `string` | `""` | — | — | — | The person-facing name; empty uses the field name. |
| `doc` | `string` | `""` | — | — | — | One sentence on what the field means. |
| `tunable` | `boolean` | `false` | — | — | — | Publish the field to Tune and to the probe's bounds and patch. |
| `worldspace` | `string` | `""` | — | — | — | Where a Vector2, Vector3, f32 or f64 field's numbers lie when og.WorldOrigin shifts: `world` (a point, moved by every shift), `root` (in its Transform's frame: moved on a root entity), `none` (an offset, size, direction), then optionally axes (`world y`: an absolute height); empty: not a coordinate. |

```ogt
struct FieldOptions {
  doc "Metadata of one field. Ranges and units are what Tune and the probe's bounds show. Nothing clamps writes: world.insert and writes from outside the game (the probe's `set`, `patch` and sweeps, Tune, a look's fields at cook) are refused outside min..max with the range named. Game code's other writes (world.set, a resource field, a column) are stored as given."
  desc true
  field min Option.f32 "The smallest meaningful value."
  field max Option.f32 "The largest meaningful value."
  field step Option.f32 "The increment Tune uses."
  field unit String "A unit name from the API's unit table, or empty."
  field label String "The person-facing name; empty uses the field name."
  field doc String "One sentence on what the field means."
  field tunable bool "Publish the field to Tune and to the probe's bounds and patch."
  field worldspace String "Where a Vector2, Vector3, f32 or f64 field's numbers lie when og.WorldOrigin shifts: `world` (a point, moved by every shift), `root` (in its Transform's frame: moved on a root entity), `none` (an offset, size, direction), then optionally axes (`world y`: an absolute height); empty: not a coordinate."
}
```

<sub>Module `reflect` line 9</sub>
