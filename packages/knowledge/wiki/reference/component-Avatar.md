<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Avatar"></a>
### `Avatar`

A portrait in a round well: presence is a dot on its rim (and words under the name, so it never rests on colour), the level a badge across its foot.

`import { Avatar } from "@origingame/engine/ui/components"` (kit `base`, `packages/engine/src/ui/components/base/avatar.tsx`).

| Prop | Type | What |
|---|---|---|
| `name?` | `string` | The displayed and accessible name; empty uses the localized unnamed-player label. |
| `showName?` | `boolean` | Shows the name caption; false keeps the accessible name for composition beside a heading. Default true. |
| `portrait?` | `Texture \| 0 \| undefined` | A loaded portrait texture; without one the fallback icon shows. |
| `icon?` | `string` | Fallback atlas icon when there is no portrait. Default "person". |
| `level?` | `number` | Level badge; 0 hides it. Default 0. |
| `status?` | `string` | online, away, busy or offline; empty hides the presence row. It is shown in words as well as colour. |
| `statusText?` | `string` | The game's wording of presence; empty uses the standard status label. |
| `frame?` | `string` | A frame key a pack selects through data-frame; the component imposes no decorative frame. |
| `loading?` | `boolean` | Portrait data is loading; shows a loading label and keeps the name and layout. Default false. |

A piece of a panel: drawn on a panel's ground.

Fixtures: `default`, `away`, `busy`, `offline`, `empty`, `loading`, `captionless`.
