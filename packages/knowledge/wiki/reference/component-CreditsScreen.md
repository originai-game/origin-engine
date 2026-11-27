<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-CreditsScreen"></a>
### `CreditsScreen`

The credits: the sections' titles and names rolling over the key art's veil, the logo at the head, and the end and licences the caller supplies. The game moves the roll (offset); it reports onBack and onFaster.

`import { CreditsScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/credits.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `sections` | `ReadonlyArray<{ id: string; names: ReadonlyArray<{ id: string; name: string; role: string }>; title: string }>` | The sections: { title, names: [{ name, role }] }. The engine's own credits (its art's authors) follow them. |
| `offset` | `number` | How far the roll has run, CSS px. |
| `keyArt` | `Texture \| 0` | The key art's texture under the roll's veil (og.load.texture; 0: the style's dark ground). |
| `logo` | `Texture \| 0` | The game's logo texture at the head of the roll (og.load.texture; 0: the logo slot's placeholder). |
| `end?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the end snippet). |
| `licences?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the licences snippet). |
| `onBack?` | `(e: UiEvent, key?: string) => void` | The back event, with the row's key where it comes from a row. |
| `onFaster?` | `(e: UiEvent, key?: string) => void` | The faster event, with the row's key where it comes from a row. |

Pack slots: `Screen.frame`.

Fixtures: `default`.

Actions: `confirm` (Faster), `back` (Back).
