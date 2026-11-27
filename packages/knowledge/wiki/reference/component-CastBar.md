<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-CastBar"></a>
### `CastBar`

A cast or channel bar: icon, name and remaining time over a track whose fill follows `elapsed / duration` (drains for a channel). The game owns the timing and the states; the bar shows `locked` and `interrupted` through `data-locked` and `data-interrupted`. The remaining time reads tenths under ten seconds. Dressed by `CastBar.frame`, `CastBar.track` and `CastBar.fill`.

`import { CastBar } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/cast-bar.tsx`).

| Prop | Type | What |
|---|---|---|
| `name?` | `string \| undefined` | What is being cast (Fireball, Reviving); empty uses Casting. |
| `icon?` | `Texture \| 0 \| undefined` | A texture (og.load.texture); 0 or absent draws `glyph` instead. |
| `glyph?` | `string \| undefined` | An icon name, in art's place. |
| `elapsed?` | `number \| undefined` | Seconds elapsed since the cast began, as the game counts them. |
| `duration?` | `number \| undefined` | The cast's full length in seconds; nonpositive shows an empty bar. |
| `channel?` | `boolean \| undefined` | A channel: the fill drains from full instead of filling. |
| `locked?` | `boolean \| undefined` | The cast cannot be interrupted: the fill turns neutral and a shield marks it. |
| `interrupted?` | `boolean \| undefined` | The cast was broken off: the fill turns danger and Interrupted replaces the time. |
| `hideTime?` | `boolean \| undefined` | Hide the remaining time. |
| `class?` | `string \| undefined` | Classes after its own (a width, a placement). |

Pack slots: `CastBar.fill`, `CastBar.frame`, `CastBar.track`.

Fixtures: `default`, `channel`, `locked`, `interrupted`, `long`.
