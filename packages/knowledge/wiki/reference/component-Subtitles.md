<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Subtitles"></a>
### `Subtitles`

The latest lines at the foot (or top, for `position: "top"` WebVTT cues) of the screen, centred in HUD type over a soft dark band: the speaker in gold small capitals over the words in white, with rich inline spans. Only older history entries in `lines` step back smaller; simultaneous caption tracks (such as bilingual subtitles) draw at full legibility together.

`import { Subtitles } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/subtitles.tsx`).

| Prop | Type | What |
|---|---|---|
| `lines?` | `ReadonlyArray<{ id: string; speaker: string; text: string; spans?: readonly TextSpan[] \| undefined }> \| undefined` | The game's own lines, oldest first, each { id, speaker, text }; the game drops a line when it is done. |
| `captions?` | `ReadonlyArray<SubtitleCaption> \| undefined` | The captions the timelines are saying (og.timeline.captions(world), set each tick), after the game's lines: a film segment's SRT/WebVTT track, a cutscene's caption track. |
| `raised?` | `boolean \| undefined` | Lift bottom captions clear of a docked choice bar (FlowScreen sets this while a choice window is open). |

Each `SubtitleCaption`:

| Field | Type | What |
|---|---|---|
| `id` | `string` | Stable identifier while the caption shows. |
| `text` | `string` | Spoken text in the player's locale. |
| `speaker?` | `string \| undefined` | Optional speaker name (WebVTT `<v Speaker>` or caption track `speaker`). |
| `spans?` | `readonly TextSpan[] \| undefined` | Optional rich inline style spans (`strong`, `emphasis`, `color`). |
| `position?` | `string \| undefined` | Placement hint: `"bottom"` (default) or `"top"` (`{\an8}` or WebVTT `line:0`). |

Pack slots: `Plate.frame`, `Screen.frame`.

Fixtures: `default`, `captions`.
