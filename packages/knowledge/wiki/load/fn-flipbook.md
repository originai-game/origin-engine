<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-flipbook"></a>

### `og.load.flipbook`

Load a cooked file.flipbook by canonical content ID and register its native sheet with og.flipbook. Author a texture ref with grid, frames or explicit atlas input, or animated GIF/WebP/APNG sources packed into <id>#sheet. array: true produces a texture array (FlipbookInfo.array), frame n drawn as layer n. A streaming sheet loads only when its group is resident; sprites draw nothing while it is away. Repeated loads return the same handles.

```ts
og.load.flipbook(world: World, path: string): Result<FlipbookInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose animation and renderer own it. |
| `path` | `string` | no | The sheet's canonical content ID, such as `base/sprites/hero`. |

**Returns** `Result<FlipbookInfo>`: Refused for a path not in the bundle (run og cook) or a texture it cannot load.

Binding symbol `og_load_flipbook` · publish builds: kept

```ogt
fn flipbook {
  doc "Load a cooked file.flipbook by canonical content ID and register its native sheet with og.flipbook. Author a texture ref with grid, frames or explicit atlas input, or animated GIF/WebP/APNG sources packed into <id>#sheet. array: true produces a texture array (FlipbookInfo.array), frame n drawn as layer n. A streaming sheet loads only when its group is resident; sprites draw nothing while it is away. Repeated loads return the same handles."
  param world World "The world whose animation and renderer own it."
  param path String "The sheet's canonical content ID, such as `base/sprites/hero`."
  returns Result.FlipbookInfo "Refused for a path not in the bundle (run og cook) or a texture it cannot load."
}
```

<sub>Module `load` line 495</sub>
