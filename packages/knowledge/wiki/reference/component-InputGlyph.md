<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-InputGlyph"></a>
### `InputGlyph`

One input binding, drawn the same everywhere a prompt names a key: given an action, the binding's own glyph on the device used last (the engine resolves it; it draws unboxed, since the glyph is its own cap); given only words, a key cap on the key gradient in the key ink. Pressed sinks and lights the cap; disabled greys it. Purely presentational: it takes no presses.

`import { InputGlyph } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/input-glyph.tsx`).

| Prop | Type | What |
|---|---|---|
| `action?` | `string \| undefined` | The input action whose binding it shows (interact, jump): the pack's icon for it on the device used last. Wins over `keyLabel`. |
| `keyLabel?` | `string \| undefined` | A fixed key cap's words (E, Space, LMB) when there is no action to look up. |
| `size?` | `"sm" \| "md" \| "lg" \| undefined` | `sm` (20 px), `md` (26 px, the default) or `lg` (32 px). |
| `pad?` | `boolean \| undefined` | A gamepad face button: a round cap. |
| `pressed?` | `boolean \| undefined` | The key is held: the cap sinks and lights. |
| `disabled?` | `boolean \| undefined` | The action cannot be taken now: the cap greys. |
| `class?` | `string \| undefined` | Classes after its own (a margin, a placement). |

Fixtures: `action`, `key`, `wide`, `pad`, `pressed`, `disabled`.
