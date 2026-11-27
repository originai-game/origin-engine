<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`path`](../path.md) · [index](../index.md)

<a id="fn-sample"></a>

### `og.path.sample`

The point at arc-length parameter t: 0 the start, 1 the end (a closed path wraps, an open one clamps). Equal steps of t are equal lengths along the curve.

```ts
og.path.sample(world: World, path: Path, t: number): Result<PathSample>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `path` | `Path` | no | The path. |
| `t` | `number` | no | The parameter, 0 to 1. |

**Returns** `Result<PathSample>`: Refused for a dead path.

Binding symbol `og_path_sample` · publish builds: kept

```ogt
fn sample {
  doc "The point at arc-length parameter t: 0 the start, 1 the end (a closed path wraps, an open one clamps). Equal steps of t are equal lengths along the curve."
  param world World "The world."
  param path Path "The path."
  param t f32 "The parameter, 0 to 1."
  returns Result.PathSample "Refused for a dead path."
}
```

<sub>Module `path` line 103</sub>
