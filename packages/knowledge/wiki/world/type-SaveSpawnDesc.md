<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`world`](../world.md) · [index](../index.md)

<a id="type-SaveSpawnDesc"></a>

### `SaveSpawnDesc` (desc struct)

An entity a migration adds to a save.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | A unique name; empty leaves it unnamed. |
| `components` | `unknown` | `null` | — | — | — | Its components by type name, each a value of the type as this build declares it (`{ "og.Transform": {}, Chest: { gold: 5 } }`). |

```ogt
struct SaveSpawnDesc {
  doc "An entity a migration adds to a save."
  desc true
  field name String "A unique name; empty leaves it unnamed."
  field components Json "Its components by type name, each a value of the type as this build declares it (`{ \"og.Transform\": {}, Chest: { gold: 5 } }`)."
}
```

<sub>Module `world` line 72</sub>
