# Three-Lane Beat — the rhythm archetype (2D)

Notes fall down three lanes to the hit line in time with "Neon Pulse"
(120 BPM, 24 bars). Hit each as it crosses the line. A hit is judged by
how far it was from the note's exact time (Perfect, Great, Good), and a
note let past is a Miss; hits build the combo and its multiplier and fill
the groove, misses drain it, and an empty groove ends the song. A
calibration measures how late this device's sound and picture reach you.

三道节拍：音符沿三条轨道落向判定线，与《霓虹脉冲》（120 BPM，24 小节）同拍。在音符经过判定线时击打它。每次击打按离音符准确时间的偏差判定（完美、很好、不错），漏掉的音符算失误；击中累积连击和倍率、填满律动槽，失误会让它下降，律动槽空了歌曲就结束。校准会测出这台设备的声音和画面传到你这里的延迟。

This is a complete Origin Engine project: `og run .` plays it headless,
`og build .` makes the play page, `og test .` runs its playtests and
`og polish .` scores them. It rebuilds the platform's retired Discover
rhythm prototype on the engine: its premise (lanes, a chart, judged hits,
a combo, a song to clear) and none of its code or music.

```sh
og cook .                    # assets/ into cooked/ (og run and og build cook too)
og run . --headless --commands "0.5:play;4:screenshot:highway.png;4.1:quit"
og test .                    # tests/*.og.ts through og run --headless --commands
og polish .                  # the playtests scored against og-taste and og-feel
og build . --publish         # dist/: the play page as it ships
```

## What it demonstrates, and the skills it exercises

- **og.Music's clock and og.MusicBeat** (og-audio §2, §5): the song's
  clock is an `og.Music` (120 BPM, 4/4, `start` the world time of bar 1)
  that the Clock system keeps on `Game.songStart`. The song is stems, not
  one file: on the last beat of each bar og.MusicBeat brings, the Band
  starts the next bar's stems with `AudioSource.startAt` on that bar line,
  so they start to the sample and stay in step (drums every bar; bass,
  chords and the lead's phrases two bars long on odd bars). The same
  events schedule the notes: each beat brings every note up to two beats
  ahead (`Tuning.scrollBeats`) onto the highway.
- **Hit windows in time, not ticks**: a note's time is
  `songStart + beat × 60 / bpm` exactly; a press is judged at its tick's
  world time less the calibrated offset, against that time, in
  milliseconds (Perfect ±45, Great ±90, Good ±135, `Tuning`). Early and
  late are shown under Great and Good.
- **Judgement text, a combo meter, the groove**: the judgement rises from
  the lane in its colour; the combo counts beside the highway with its
  multiplier (×1 to ×4, one more every ten) and an `og.ui.meter` to the
  next; the groove (the life gauge) is an `og.ui.meter` that hits fill
  and misses drain.
- **Latency calibration**: Calibrate (on the title or in the settings)
  clicks every beat; eight taps on the clicks give the mean error, saved
  as `Prefs.offsetMs` with the settings; the settings also take it by
  hand (±200 ms). og.audio gives no output latency (see Known limits), so
  the calibration measures the whole chain a player perceives.
- **A chart format in .ogt** (og-game §8): a chart is a scene,
  `assets/charts/neon-pulse.ogt`: one entity with a `Song` (tempo, metre,
  bars, notes) and an entity per note with a `Note` (index, lane 0–2,
  beat from 0 at bar 1). `start` instantiates it; the rows are the
  notes. It is text a Maker edits by hand; `tools/make-assets.py` writes
  this one from the same note lists as the stems.
- **Pause and resume in time**: pausing freezes the play's clock where it
  is and stops the band (og.audio runs on the world's ticks, which keep
  going); Resume counts in again and restarts the two-bar phrase it
  paused in, notes already judged staying judged.
- **Feel** (og-feel §4–§7): a press squashes its lane's receptor and
  plays its lane's tick (on the ui bus); a hit flashes the receptor in the
  lane's colour, bursts sparks from it and nudges the camera down; every
  tenth of a combo punches it; a miss shakes it with a dull thud (a combo
  of ten or more broken, louder); the rings breathe with the beat (not
  with reduced motion). There is no hit stop: it would hold the picture
  while the music runs on.
- **Screens on og.ui with a skin** (og-ui §2–§6, og-taste §1–§7): the
  engine's sci-fi pack recoloured as `assets/skins/lanebeat`; the title
  (Play, Calibrate, Settings), the HUD (the score as an `og.ui.counter`
  and the groove top left, Pause top right, the combo beside the
  highway), `og.ui.pauseMenu` (Resume, Restart, Settings, Title), the
  settings with the offset, the calibration and the results built from
  widgets (the grade popping in, a new best), a wipe between screens (a
  fade with reduced motion).
- **Input everywhere**: A S D, J K L or the arrows (Space too for the
  middle lane); the pad's d-pad or X A B; a finger on a lane (`og.Pointers`,
  so chords take two fingers).
- **Languages and the platform** (og-i18n, og-platform-sdk §1–§4): every
  string is a key of `assets/strings/en.ogt` and `zh-CN.ogt`; settings
  through `loadSettings`/`saveSettings`; the best score read on tick 0 and
  written at the revision read; the board; a `song.over` event.

`og polish .` (2026-10-19, og and the native addon built release from
this checkout, load 18): taste 99, feel 99, in
`tests/evidence/P2/archetypes/rhythm/` of the engine repository (feel 74
at load 27–73, frame time's p95 following the load:
`tests/evidence/P2/archetypes/frame-time.md`). Hit feedback is n/a: no
note is damage, and a hit stop would hold the picture while the music
runs on.

## Rules

- **Hits.** Each lane's press hits that lane's nearest unjudged note
  within ±135 ms of its time: within 45 ms Perfect (300 points), 90 Great
  (200), 135 Good (100), times the multiplier. A press with no note near
  is only a sound. A note 135 ms past its time is a Miss.
- **Combo.** Hits in a row; the multiplier is one more every ten, up to
  four. A Miss breaks it.
- **Groove.** Starts at half; a Perfect adds 0.03, a Great 0.021, a Good
  0.012; a Miss takes 0.1. Empty, the song fails.
- **The end.** A beat and a half after the last note is judged, the
  results: accuracy (a Perfect whole, a Great 70 %, a Good 40 %), the
  grade (S from 95 %, A 85, B 70, C 50, else D), the counts, the best
  combo, a new best kept and submitted.

Everything is deterministic: the clock is the world's ticks, the chart is
data, and `tests/replay.og.ts` replays a play to the same hash.

## How it feels

| Moment | What happens | Where |
|---|---|---|
| a press | the lane's receptor squashes 1.12 × 0.82, the lane's tick (C, E or G) | `Lanes`, `Voices` |
| a hit | the receptor flashes in the lane's colour, 14 sparks, the judgement rises (and early or late), a 0.04 m nudge down | `judge`, `Lanes` |
| every tenth of a combo | a 0.03 punch | `judge` |
| a miss | `Miss` in red, trauma 0.18, a dull thud (louder when a combo of ten or more breaks) | `judge` |
| the beat | the receptors' rings brighten and fade | `Lanes` |
| the count-in | four clicks, `4 3 2 1` over the middle lane | `Band`, `Play` |
| the end | a fanfare (cleared) or a falling tone (failed, the band stops), a wipe to the results, the grade popping in | `Play`, `results` |

## State

| Type | Kind | What it holds |
|---|---|---|
| `Game` | resource | the phase, the clock (`songStart`, the pause, the count-in), the score, the combo, the counts by judgement, the groove, the end, the calibration's taps, the best, the layout, sound requests, the band's requests |
| `Tuning` | resource | the knobs below |
| `Prefs` | resource | the volumes, the language, reduced motion, the offset, the settings request |
| `Song` | the chart's song row | tempo, metre, bars, notes |
| `Note` | the chart's note rows | index, lane, beat; live, judgement, error |
| `Stem` | the 15 stems (and clicks) | the part, starts |
| `Receptor` | the three receptors | lane, presses |
| `Sfx` | one row per sound | its `Sound.*` bit, its plays |

## Systems (in tick order)

| System | Job |
|---|---|
| `Ui` | the screens, the layout for the surface, the buses, starting, pausing, resuming and ending a play, the calibration, the platform's answers |
| `Clock` | keeps the og.Music clock on `Game.songStart` |
| `Play` | presses, judgements, misses, the notes og.MusicBeat schedules, the highway, the count-in's numbers, the end |
| `Band` | starts the stems on their bar lines (og.MusicBeat), the count-in's and the calibration's clicks; stops them |
| `Lanes` | the receptors' squash, flash, burst and beat |
| `Voices` | plays each requested sound once |

og.music (before og.audio) writes the clock and sends og.MusicBeat.

## Tunables (`Tuning`)

| Field | Default | What it changes |
|---|---|---|
| `perfectMs`, `greatMs`, `goodMs` | 45, 90, 135 ms | the hit windows |
| `scrollBeats` | 2 | how far ahead notes appear (so how fast they fall) |
| `countInBeats` | 4 | the count-in's clicks |
| `grooveStart`, `grooveGain`, `grooveLoss` | 0.5, 0.03, 0.1 | the groove |
| `perfectPoints`, `greatPoints`, `goodPoints` | 300, 200, 100 | the points |
| `comboStep` | 10 | the combo per multiplier step |

## What a Maker changes first

1. **The chart**: `assets/charts/neon-pulse.ogt` by hand (a `Note` entity
   per note), or the note lists in `tools/make-assets.py`.
2. **The song**: its stems in `tools/make-assets.py` (or recorded ones,
   bar-aligned) and `partsAt` in `src/systems.ts`, which says which stem
   starts on which bar.
3. **The difficulty**: the windows, the scroll and the groove in `Tuning`.
4. **The look**: `assets/skins/lanebeat/skin.ogt`'s palette, the lanes'
   colours (`LANE_COLOURS`) and the art in `tools/make-assets.py`.
5. **Words**: `assets/strings/*.ogt`; a new Chinese string needs the
   script run again to subset the font.

## Commands (for scripts and agents)

| Action | Does |
|---|---|
| `play[:<bar>]` | starts the song now, as Play does; from a later bar, the earlier notes skipped |
| `hit:<lane>` | presses a lane this tick, as a key or a finger does |
| `groove:<0-1>` | sets the groove |
| `offset:<ms>` | sets the calibrated offset |
| `calibrate` | opens the calibration |
| `hold`, `unhold` | pauses and resumes, as Escape and Resume do |
| `viewport:<desktop\|phone>` | lays the UI out for a phone in portrait (390 × 844 at 3×) |
| `language:<en\|zh-CN>` | switches the language |
| core actions | `touch_move_to:play`, `Pause`, `resume`, `Again`, `Title`, `Done`; keys `A S D`, `J K L`, arrows, `Space`, `Escape`; `gamepad:left` …; `touch_move` + `touch_start` on a lane |

`facts` (the probe's `state()`) reports the phase, the song's time, the
score, the combo and the best combo, the counts by judgement, the notes
judged and in the chart, the groove, the accuracy, cleared or failed, the
best and the language. The playtests read the same values from the
`Game` resource with `get:Game`, and press the lanes at the chart's times
(they read `assets/charts/neon-pulse.ogt` as data).

## Playtests (`og test .`)

| File | Proves |
|---|---|
| `tests/flow.og.ts` | Play by tap; the count-in; bars 1–2 hit on time: eight Perfects, a combo of 8, 2,400 points; 80 ms late is Great, 120 ms early Good; two notes let pass are Misses; an empty press judges nothing |
| `tests/band.og.ts` | the four clicks on the beats before bar 1, the drums on bar 1's line, the bass and chords on bar 5's, the lead on bar 9's, each `startAt` the exact bar time |
| `tests/clear.og.ts` | from bar 21 every note hit: cleared, grade S, 100 %, the best kept at the revision read and submitted, the telemetry, Play again |
| `tests/fail.og.ts` | five notes let pass empty the groove: failed, the band stopped, the results without a grade |
| `tests/pause.og.ts` | Escape freezes the clock (nothing missed while paused), stops the band, holds the time scale; 简体中文 saved; Resume counts in and restarts the phrase; the next notes hit at their new times |
| `tests/calibrate.og.ts` | eight taps 67 ms after the clicks calibrate 67 ms, saved; a play 67 ms late is then all Perfect |
| `tests/pad.og.ts` | the d-pad and the face buttons start the song and hit the lanes; Start pauses, A resumes |
| `tests/phone.og.ts` | 390 × 844 at 3×: the HUD inside the safe area, three lanes across, fingers hit eight notes on time |
| `tests/frames.og.ts` | the title, the highway in bar 9 and the results match `references/<tier>/` (`OG_FX_CAPTURE=1 og test .` records them) |
| `tests/replay.og.ts` | a play with hits on time, late and early and misses, run twice, ends on one hash |

Each one fails when the behaviour it names is broken (one was broken on
purpose and restored: `tests/evidence/P2/archetypes/break-and-restore.md`
in the engine repository).

## Assets

`tools/make-assets.py` regenerates every asset byte for byte (two runs
give the same bytes): the chart and the stems from one set of note lists,
the click, the taps, the miss, the clear and the fail, the note, the ring,
the lane's glow, the spark, the hit's burst, the `og sfx` preset files of
the UI's sounds and the font subset. `LICENSES.md` lists them; the skin's
are in `assets/skins/lanebeat/LICENSES.md`.

## Known limits (engine gaps, not design)

- **No output latency from og.audio.** Neither `og.Audio` nor any
  og.audio call reports the device's output latency (the Web Audio
  context's `outputLatency`, a native device's buffer), so the game cannot
  correct for it alone; the calibration measures it with the player's
  taps (which also takes in the display and the input).
- **Input has no time within the tick.** og.Input reports a key, a button
  or a pointer as pressed in the tick it arrived, with no timestamp, so a
  press is judged at its tick's time: at 60 ticks a second a press can be
  read up to 16.7 ms late, a constant part of which the calibration
  absorbs. The notes, the stems and the windows are exact.
- **No seek in an AudioSource.** A source starts from its beginning
  (`startAt` says when, not where), so a resume restarts the two-bar
  phrase it paused in rather than the exact beat.
- og.ui's `resultsScreen` and `settingsScreen` carry English labels of
  their own, so this game builds its results and settings from widgets.
