# E3 transcript: an iris screen transition and the results count-up

Produced with `og run templates/ui-kit --headless --size 1280x720 --commands` (native host, metal) and this script:

```
0.10:skin:cartoon;0.15:transition:iris;0.30:show:results;
then at t = 0.30, 0.40, … 2.40: screenshot:f<t>.png and dump_ui:u<t>.json;2.70:quit
```

`transition:iris` makes the demo pass its wanted screen through `og.ui.screenTransition("Screens", screen, { kind: "iris", durationMs: 700 })`
and build the screen it answers; the results template counts its score with `og.ui.counter("Score", 12840, { from: 0, durationMs: 900 })`.
The table reads each `dump_ui`: which screen was built (the gallery's `Screens` list or the `Results` dialog) and the `Score` text.

| t (s) | tick | screen built | Score |
|---|---|---|---|
| 0.30 | 18 | gallery |  |
| 0.40 | 24 | gallery |  |
| 0.50 | 30 | gallery |  |
| 0.60 | 36 | gallery |  |
| 0.70 | 42 | results | 2,023 |
| 0.80 | 48 | results | 5,409 |
| 0.90 | 54 | results | 8,003 |
| 1.00 | 60 | results | 9,910 |
| 1.10 | 66 | results | 11,235 |
| 1.20 | 72 | results | 12,085 |
| 1.30 | 78 | results | 12,565 |
| 1.40 | 84 | results | 12,781 |
| 1.50 | 90 | results | 12,838 |
| 1.60 | 96 | results | 12,840 |
| 1.70 | 102 | results | 12,840 |
| 1.80 | 108 | results | 12,840 |
| 1.90 | 114 | results | 12,840 |
| 2.00 | 120 | results | 12,840 |
| 2.10 | 126 | results | 12,840 |
| 2.20 | 132 | results | 12,840 |
| 2.30 | 138 | results | 12,840 |
| 2.40 | 144 | results | 12,840 |

What the frames show (`transition-iris-countup.png`: 0.4, 0.5, 0.6, 0.7, 0.8 s on the top row, 0.9, 1.0, 1.2, 1.5, 2.4 s below):

- 0.4–0.6 s: the cartoon `bg` closes in as a circle over the gallery, which is still built (the old screen stays while the cover closes);
- between 0.6 and 0.7 s (tick 39 of a 42-tick, 700 ms transition starting at tick 18): the cover is complete and the answer switches to `results`; the dialog is built under the cover and enters (its scrim fades, its panel pops);
- 0.7–1.0 s: the iris opens on the results while the score counts (ease-out cubic by the tick: 2,023, 5,409, 8,003, 9,910), the earned stars land with bursts;
- 1.6 s on: the score has landed on 12,840 and `New best` has popped in; the cover is long gone.

The same sequence is asserted by `templates/ui-kit/tests/juice.og.ts` (old screen while covering, new while revealing, a mid count, 12,840 at rest, UI voices on a press).
