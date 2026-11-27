<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ActionButton"></a>
### `ActionButton`

A touch disc: an action a finger holds, the primary one larger, an optional charge ring round its rim. The disc's box is a control zone of its og-input action (touchButton), so the action is 1 while a touch that started on it is held; the disc has no press handler of its own. The face's radial sheen is a linear one.

`import { ActionButton } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/action-button.tsx`).

| Prop | Type | What |
|---|---|---|
| `action` | `string` | The og-input action it feeds while a touch holds it (jump). |
| `label` | `string` | Its words, its accessible name; shown when it has no icon. |
| `icon?` | `string \| undefined` | An icon of the atlas on its face; empty shows the label. |
| `primary?` | `boolean \| undefined` | The main action: the primary fill and a larger disc. |
| `charge?` | `number \| undefined` | A charge round its rim, 0 to 1 (an ultimate); negative is none. Default -1. |
| `disabled?` | `boolean \| undefined` | It feeds nothing and shows faded. |

Pack slots: `Touch.button`, `Touch.button.primary`, `Touch.charge`, `Touch.charge.ready`, `Touch.icon`.

Fixtures: `default`.
