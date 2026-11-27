<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ArenaVersus"></a>
### `ArenaVersus`

A versus reveal: each side's team name and fighters facing each other, replayed when its id changes. An empty side reads Waiting for opponents. Read only.

`import { ArenaVersus } from "@origingame/engine/ui/components"` (kit `arena`, `packages/engine/src/ui/components/arena/versus.tsx`).

| Prop | Type | What |
|---|---|---|
| `nameA?` | `string` | The first side's team name; empty takes the kit's (Blue team). |
| `nameB?` | `string` | The second side's team name; empty takes the kit's (Red team). |
| `sideA?` | `ReadonlyArray<VersusFighter>` | The first side's fighters in the game's order. |
| `sideB?` | `ReadonlyArray<VersusFighter>` | The second side's fighters, keyed as sideA. An empty side reads Waiting for opponents. |
| `at?` | `number` | The reveal's id: a new value replays the sides' entrance. |
| `compact?` | `boolean` | Its place is narrow (under about 300 px): the cards and portraits tighten. The place decides, not the screen. |

Each `VersusFighter`:

| Field | Type | What |
|---|---|---|
| `id` | `string` | The fighter's key. |
| `name` | `string` | What a reader calls them. |
| `portrait?` | `Texture \| 0` | The fighter's portrait texture (0 shows the icon). |
| `icon?` | `string` | An icon name, shown while no portrait is set. |
| `character?` | `string` | The champion's name (empty hides). |
| `rank?` | `string` | The rank as the game words it (Gold II, 1840 SR; empty hides). |
| `local?` | `boolean` | Marks the local player (You). |

Pack slots: `Chip.frame`, `Panel.frame`.

Fixtures: `default`, `teams`, `overflow`.
