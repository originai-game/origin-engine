<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Art"></a>
### `Art`

A slot of generated art: an image that covers its box, cropped about its centre.

`import { Art } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/art.tsx`).

| Prop | Type | What |
|---|---|---|
| `kind` | `string` | The slot's kind of generated art (keyArt, logo, levelThumb, bust …); the art tools fill slots by kind. |
| `src` | `Texture \| 0` | The loaded texture; 0 is none yet. |
| `class?` | `string \| undefined` |  |
| `label?` | `string \| undefined` |  |
