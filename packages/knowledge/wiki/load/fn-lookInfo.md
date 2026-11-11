<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-lookInfo"></a>

### `og.load.lookInfo`

What a loaded look holds and sets.

```ts
og.load.lookInfo(world: World, look: Look): Result<LookInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the look. |
| `look` | `Look` | no | The look. |

**Returns** `Result<LookInfo>`: Refused for a look not loaded in this world.

Binding symbol `og_load_look_info` · publish builds: kept

```ogt
fn lookInfo {
  doc "What a loaded look holds and sets."
  param world World "The world that owns the look."
  param look Look "The look."
  returns Result.LookInfo "Refused for a look not loaded in this world."
}
```

<sub>Module `load` line 361</sub>
