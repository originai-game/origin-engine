<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-flipbook"></a>

### `og.load.flipbook`

Load a cooked flipbook sheet (an `.ogt` of `type flipbook`: an Aseprite or TexturePacker atlas, or animated GIF, WebP or APNG sources the cook packs into `<sheet>#sheet`) and its texture, and register it with og.flipbook. A sheet with `array true` cooks into a texture array (FlipbookInfo.array), frame n drawn as layer n. A sheet in a streaming group loads once its group is resident (refused naming the group before), and its sprites draw nothing while the group is away. Loading the same path again answers the same handles.

See `loading §8`.

```ts
og.load.flipbook(world: World, path: string): Result<FlipbookInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose animation and renderer own it. |
| `path` | `string` | no | The sheet's asset id, such as `sprites/hero.ogt`. |

**Returns** `Result<FlipbookInfo>`: Refused for a path not in the bundle (run og cook) or a texture it cannot load.

Binding symbol `og_load_flipbook` · publish builds: kept

```ogt
fn flipbook {
  doc "Load a cooked flipbook sheet (an `.ogt` of `type flipbook`: an Aseprite or TexturePacker atlas, or animated GIF, WebP or APNG sources the cook packs into `<sheet>#sheet`) and its texture, and register it with og.flipbook. A sheet with `array true` cooks into a texture array (FlipbookInfo.array), frame n drawn as layer n. A sheet in a streaming group loads once its group is resident (refused naming the group before), and its sprites draw nothing while the group is away. Loading the same path again answers the same handles."
  guide "loading §8"
  param world World "The world whose animation and renderer own it."
  param path String "The sheet's asset id, such as `sprites/hero.ogt`."
  returns Result.FlipbookInfo "Refused for a path not in the bundle (run og cook) or a texture it cannot load."
}
```

<sub>Module `load` line 420</sub>
