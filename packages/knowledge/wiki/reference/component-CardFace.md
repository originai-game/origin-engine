<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-CardFace"></a>
### `CardFace`

One card's face: the cost gem, the art window over its rarity's glow, the name on its plate, the type line and the rules text. It draws what the game words and reports nothing; the hand, the collection and the reward wrap it in their buttons. It fills the box it is given (give the box the card's aspect ratio). The frame wears its type's colour; the name takes two lines and the rules text four, each ending in an ellipsis at a whole line, and the art window yields its height to them.

`import { CardFace } from "@origingame/engine/ui/components"` (kit `card`, `packages/engine/src/ui/components/card/face.tsx`).

| Prop | Type | What |
|---|---|---|
| `name` | `string` | The card's name as the game words it (an upgraded card's name included). |
| `cost` | `string` | Its cost as the game words it (2, X, 0); empty hides the cost gem (a relic, an unplayable curse). |
| `icon` | `string` | Its art as an icon in the style's or the game's atlas, used when art is 0. |
| `art` | `Texture \| 0` | A painted picture (og.load.texture) filling the art window; 0 shows the icon. |
| `type` | `string` | The card's type for the pack's frame (attack, skill, power, curse, status, or the game's own word); it reaches the frame as data-type. |
| `typeLabel` | `string` | The type line as the game words it (Attack, Skill · Rare); empty hides it. |
| `rarity` | `string` | common, uncommon, rare, epic, legendary or mythic; rare and above draw the ornate card frame. |
| `text` | `string` | The rules text as the game words it, every number already computed; it wraps, up to four lines and then an ellipsis, and the art window gives it the room. |
| `dim?` | `boolean` | Dims the art and the frame (a card that cannot be played or is not owned); the words keep their ink. |
| `size?` | `string` | How short its place is: empty is full height; medium, small and tiny clamp the rules text to 3, 2 and 1 lines (tiny also drops the type line). |

Pack slots: `Plate.frame`.

Fixtures: `default`, `relic`, `overflow`, `dim`.
