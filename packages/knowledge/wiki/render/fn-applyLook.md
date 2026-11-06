<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-applyLook"></a>

### `og.render.applyLook`

Apply a look, in GameDesc.start (or name it in GameDesc.look) or at any tick for a level's mood, a flashback or a photo mode: every resource the look names is replaced whole (the fields it sets take its values, the rest their defaults; PostEffects.flash and PostEffects.shakeTrauma, which game code animates, keep theirs), and the UI is configured with its style, keeping the rest of the last UiDesc. Resources it does not name are left as they are. Inside a system, the system must declare those resources among its writes, except the art contract's (ArtPalette, ArtPixels, ArtTexels, ArtShape, ArtLine, ArtMotion, ArtCamera): one the system does not declare is left as it is, so a system that declares the frame's resources applies looks that carry a contract. A look that switches the stylisation (a Stylise pass on or off, the pixel grid, retro 3D, the material style) starts TAA's history over, as a camera cut does, so its first frame holds no ghost of the old look.

```ts
og.render.applyLook(world: World, look: Look): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose look changes. |
| `look` | `Look` | no | The look, from og.load.look. |

**Returns** `Result<void>`: Refused for a look not loaded in this world, one in a streaming group that is not resident (the reason names it), or a resource the system did not declare.

Binding symbol `og_render_apply_look` · publish builds: kept

```ogt
fn applyLook {
  doc "Apply a look, in GameDesc.start (or name it in GameDesc.look) or at any tick for a level's mood, a flashback or a photo mode: every resource the look names is replaced whole (the fields it sets take its values, the rest their defaults; PostEffects.flash and PostEffects.shakeTrauma, which game code animates, keep theirs), and the UI is configured with its style, keeping the rest of the last UiDesc. Resources it does not name are left as they are. Inside a system, the system must declare those resources among its writes, except the art contract's (ArtPalette, ArtPixels, ArtTexels, ArtShape, ArtLine, ArtMotion, ArtCamera): one the system does not declare is left as it is, so a system that declares the frame's resources applies looks that carry a contract. A look that switches the stylisation (a Stylise pass on or off, the pixel grid, retro 3D, the material style) starts TAA's history over, as a camera cut does, so its first frame holds no ghost of the old look."
  param world World "The world whose look changes."
  param look Look "The look, from og.load.look."
  returns Result.Void "Refused for a look not loaded in this world, one in a streaming group that is not resident (the reason names it), or a resource the system did not declare."
}
```

<sub>`api.ogt` line 6495</sub>
