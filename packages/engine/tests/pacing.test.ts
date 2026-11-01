// The engine Worker's frame cap (src/web/pacing.ts): which display frames
// are drawn under a cap, and the display's interval as measured.
import { expect, test } from "bun:test";
import { FramePacer } from "../src/web/pacing.ts";

/** Display frames at `hz` with a little jitter, for `seconds`: how many the pacer draws under `cap`, and the longest gap between drawn frames, ms. */
function run(hz: number, cap: number, seconds = 2): { drawn: number; frames: number; maxGap: number; pacer: FramePacer } {
  const pacer = new FramePacer();
  const step = 1000 / hz;
  let drawn = 0;
  let last = 0;
  let maxGap = 0;
  const frames = Math.round(seconds * hz);
  for (let i = 1; i <= frames; i++) {
    // ±0.4 ms of jitter, deterministic.
    const t = i * step + Math.sin(i * 12.9898) * 0.4;
    if (pacer.frame(t, cap)) {
      if (last) maxGap = Math.max(maxGap, t - last);
      last = t;
      drawn++;
    }
  }
  return { drawn, frames, maxGap, pacer };
}

test("a 60 cap draws every other frame of a 120 Hz display, evenly, and every frame of a 60 Hz one", () => {
  const pro = run(120, 60);
  expect(pro.drawn).toBeGreaterThanOrEqual(119);
  expect(pro.drawn).toBeLessThanOrEqual(121);
  // Never two skipped in a row: no gap reaches three display intervals.
  expect(pro.maxGap).toBeLessThan(2.5 * (1000 / 120));
  const sixty = run(60, 60);
  expect(sixty.drawn).toBe(sixty.frames);
});

test("a 30 cap on 144 Hz draws 30 a second, a cap of 0 draws every frame, and the display's interval is measured", () => {
  const gaming = run(144, 30);
  expect(Math.abs(gaming.drawn - 60)).toBeLessThanOrEqual(2);
  expect(gaming.pacer.displayMs).toBeCloseTo(1000 / 144, 0);
  const free = run(144, 0);
  expect(free.drawn).toBe(free.frames);
});

test("a long gap (a hidden page) is not the display's interval, and reset draws the next frame", () => {
  const p = new FramePacer();
  for (let i = 1; i <= 20; i++) p.frame(i * 8.33, 60);
  const measured = p.displayMs;
  p.frame(20 * 8.33 + 5000, 60);
  expect(p.displayMs).toBe(measured);
  p.reset();
  expect(p.frame(100_000, 30)).toBe(true);
  expect(p.frame(100_008, 30)).toBe(false);
});
