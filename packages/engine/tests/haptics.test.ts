// web/haptics.ts: a UI haptic pattern as the Vibration API takes it.
// Ways it could fail: a delayed first pulse vibrates at once (the pattern
// must start with an off), pulses lose their gaps, a pulse too weak to
// feel still buzzes, or a weak one buzzes as long as a strong one.
import { expect, test } from "bun:test";
import { vibrationPattern } from "../src/web/haptics.ts";

test("a pattern keeps its timing, drops what cannot be felt and shortens weak pulses", () => {
  expect(vibrationPattern([[0, 40, 1, 0]])).toEqual([40]);
  expect(vibrationPattern([[30, 20, 0, 1]])).toEqual([0, 30, 20]);
  // Two taps 80 ms apart, the second after the first's 20 ms: a 60 ms gap.
  expect(vibrationPattern([[80, 20, 0.9, 0], [0, 20, 0.9, 0]])).toEqual([20, 60, 20]);
  expect(vibrationPattern([[0, 40, 0.1, 0.15]])).toEqual([]);
  expect(vibrationPattern([[0, 40, 0, 0.4]])).toEqual([24]);
});
