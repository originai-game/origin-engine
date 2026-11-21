# rhythm: built-in components

Song select, difficulty, the results screen and the play HUD (the song
plate, its progress line, the chain, the score, the life and fever gauges,
the combo and judgement, the timing tally and the lane keys). They draw
what the game tells them and report intents with stable row keys. They
never judge a hit, keep time, count a combo, compute a score, accuracy,
life, grade or verdict, or decide what is unlocked: every figure arrives
from the game, formatted where it is text.

A game imports one by name from `@origingame/engine/ui/components`, with
its props as typed in its `.tsx` (`<Name>Props`, each prop documented
there); `fixtures` in the same file are its cases. How components are built
and dressed is `../README.md`.

## Members

| Component | File | What it is |
| --- | --- | --- |
| `RhythmChain` | `rhythm/chain` | The score chain. |
| `RhythmDifficulty` | `rhythm/difficulty` | Choosing a chart. |
| `RhythmGauge` | `rhythm/gauge` | A tall gauge at a screen edge, cut into ten cells and leaning toward the play field. |
| `RhythmJudgement` | `rhythm/judgement` | The play field's centre. |
| `RhythmLaneKeys` | `rhythm/lane-keys` | The lanes' key caps under the judgement line, spread across the field's width, one per lane, each lit while the player holds it. |
| `RhythmResults` | `rhythm/results` | After the song. |
| `RhythmScoreHud` | `rhythm/score-hud` | The run's figures over the play field, set against the screen's end. |
| `RhythmSongPlate` | `rhythm/song-plate` | The song in play, at the screen's top corner. |
| `RhythmSongProgress` | `rhythm/song-progress` | The song's progress as a thin lit line along the screen's top edge, and the time played over the length riding under its played end. |
| `RhythmSongSelect` | `rhythm/song-select` | Choosing a song. |
| `RhythmTiming` | `rhythm/timing` | How the run is being judged, at a corner of the field. |

## Gallery

`bun packages/engine/src/node/gallery.ts --only rhythm/` draws every member's fixtures under every pack.
