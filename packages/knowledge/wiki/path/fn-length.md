<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`path`](../path.md) · [index](../index.md)

<a id="fn-length"></a>

### `og.path.length`

Its length along the curve.

```ts
og.path.length(world: World, path: Path): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `path` | `Path` | no | The path. |

**Returns** `Result<number>`: Metres; refused for a dead path.

Binding symbol `og_path_length` · publish builds: kept

```ogt
fn length {
  doc "Its length along the curve."
  param world World "The world."
  param path Path "The path."
  returns Result.f32 "Metres; refused for a dead path."
}
```

<sub>Module `path` line 112</sub>
