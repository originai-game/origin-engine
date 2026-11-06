<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-flipbook"></a>

### `og.load.flipbook`

Load a cooked flipbook sheet (an `.ogt` of `type flipbook`, with its Aseprite or TexturePacker atlas resolved by the cook) and its texture, and register it with og.flipbook. A sheet may name animated GIF, WebP or APNG files instead of a texture (`source "hero_run.gif"`, a clip per file, with `key auto` to key out a flat background, `align feet` to hold the figure still and `scale` to resample): the cook turns their delays into clip durations, crops every frame to one size with the pivot at the anchor, shares identical frames and packs them into the texture `<sheet>#sheet`; `og sprite inspect` measures a source's frames, drift, loop seam and edge fringe. A sheet with `array true` is cooked into a texture array instead (its frames, one size, cut out of the texture as layers): its sprites draw frame n as layer n, every sprite of the sheet one draw whichever frame each shows, and FlipbookInfo.array names the array. A sheet in a streaming group (its array and layers with it) loads once its group is resident (refused naming the group before), and its sprites draw nothing while the group is away. Loading the same path again answers the same handles.

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
  doc "Load a cooked flipbook sheet (an `.ogt` of `type flipbook`, with its Aseprite or TexturePacker atlas resolved by the cook) and its texture, and register it with og.flipbook. A sheet may name animated GIF, WebP or APNG files instead of a texture (`source \"hero_run.gif\"`, a clip per file, with `key auto` to key out a flat background, `align feet` to hold the figure still and `scale` to resample): the cook turns their delays into clip durations, crops every frame to one size with the pivot at the anchor, shares identical frames and packs them into the texture `<sheet>#sheet`; `og sprite inspect` measures a source's frames, drift, loop seam and edge fringe. A sheet with `array true` is cooked into a texture array instead (its frames, one size, cut out of the texture as layers): its sprites draw frame n as layer n, every sprite of the sheet one draw whichever frame each shows, and FlipbookInfo.array names the array. A sheet in a streaming group (its array and layers with it) loads once its group is resident (refused naming the group before), and its sprites draw nothing while the group is away. Loading the same path again answers the same handles."
  param world World "The world whose animation and renderer own it."
  param path String "The sheet's asset id, such as `sprites/hero.ogt`."
  returns Result.FlipbookInfo "Refused for a path not in the bundle (run og cook) or a texture it cannot load."
}
```

<sub>Module `load` line 395</sub>
