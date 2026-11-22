# input-timing

Two actions defined in `src/main.ts`: `tap` (Space) records each press's
`ActionState.pressedAt`, and `jump` (KeyJ, `bufferTicks` 6) is taken with
og.input.consume on the ticks in `LANDINGS` (94, 95, 127).

What it checks (`tests/timing.og.ts`, `og test tests/fixtures/input-timing`):

- a press 0.2 of a tick into tick 30 and one 0.3 of a tick before tick 60
  (script times between ticks: the command channel stamps their keys with
  the offset) report 3.3 ms and -5 ms, each within 1 ms;
- a jump pressed on tick 90 is taken on tick 94, not again on 95; one
  pressed on tick 120 is gone by 127, past its 6 ticks;
- two runs end on one hash. The time comes from the press events' `at`,
  which a replay carries with them.

The rules themselves (the window's edges, a later press restarting it, a
press waiting through a snapshot, which control's time counts) are
og-input's `tests/actions.rs`.

Budget: under 10 s warm. No frames.
