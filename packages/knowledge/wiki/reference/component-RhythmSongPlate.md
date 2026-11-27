<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RhythmSongPlate"></a>
### `RhythmSongPlate`

The song in play, at the screen's top corner: the jacket with a lit edge, the title, the artist and tempo, the chart's chip in its tier's colour with its level, and the modifiers in play. Read only.

`import { RhythmSongPlate } from "@origingame/engine/ui/components"` (kit `rhythm`, `packages/engine/src/ui/components/rhythm/song-plate.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The song's title. |
| `artist` | `string` | The artist as the game words it; empty hides it. |
| `bpm` | `string` | The tempo as the game words it (174, 90–180); empty hides it. |
| `jacket` | `Texture \| 0` | The cover art texture (og.load.texture); 0 shows the music glyph in its place. |
| `tier` | `string` | The chart's family (easy, normal, hard, expert, master, special): the colour of its chip, on data-tier, never alone. |
| `chart` | `string` | The chart's name as the game words it (Expert); empty hides the chip. |
| `rating` | `number` | The chart's level, beside its name; 0 hides it. |
| `modifiers` | `ReadonlyArray<{ id: string; label: string }>` | The modifiers in play, keyed rows of { id, label } in the game's order, each on a tag (×1.0, Mirror). |

Pack slots: `Chip.frame`.

Fixtures: `default`, `plain`, `master`, `special`, `overflow`.
