<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RhythmSongSelect"></a>
### `RhythmSongSelect`

Choosing a song: the game's list (jacket, title, artist, BPM, with New, Cleared, Locked, Selected and Previewing named in words) beside the selected song's details. Each song is a cut plate of dark glass; the selected one is washed in the HUD's accent and stepped in. A row reports choose with its key; Preview (the button, or the details action on a row) reports preview with its key. A locked song is a disabled row and reports nothing. The game owns unlocking, the selection, the preview audio and the loading; status shows its loading and error states, and the error state offers Retry (retry).

`import { RhythmSongSelect } from "@origingame/engine/ui/components"` (kit `rhythm`, `packages/engine/src/ui/components/rhythm/song-select.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The page's heading (a pack or folder name); empty takes the kit's (Songs). |
| `songs` | `ReadonlyArray<{ id: string; title: string; artist: string; bpm: string; length: string; jacket: Texture \| 0; locked: boolean; fresh: boolean; cleared: boolean }>` | The songs in the game's order, keyed rows of { id, title, artist, bpm, length, jacket, locked, fresh, cleared }: bpm and length as the game words them (128, 90–180; 2:14); jacket the cover art texture (og.load.texture, 0 none); locked the song cannot be played yet (its row is disabled); fresh newly added (New); cleared the player has cleared it (Cleared). |
| `selected` | `string` | The song in focus as the game holds it (its row selected, its details shown); empty is none. |
| `previewing` | `string` | The song whose preview the game is playing (Previewing); empty is none. |
| `status` | `string` | The list's state: empty when the songs are there, loading while the game fetches them, error when it could not. |
| `onChoose?` | `(e: UiEvent, key: string) => void` | Choose the song; the row's key. A locked song's row is disabled and reports nothing. |
| `onPreview?` | `(e: UiEvent, key: string) => void` | Preview; the details action (X, pad Y) on a focused row, with that row's key, and the Preview button with the selected song's key. |
| `onRetry?` | `(e: UiEvent) => void` | Retry; offered only while status is error. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Chip.frame`, `Panel.frame`, `Plate.frame`.

Fixtures: `default`, `selected`, `disabled`, `empty`, `loading`, `error`, `overflow`.
