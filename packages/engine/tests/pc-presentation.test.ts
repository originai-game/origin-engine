// PC in the browser (G1-7): the shell's pointer-lock state machine, the
// device-pixel-ratio watch, the keyboard layout read for glyphs and the
// canvas held inside a game's aspect range.
//
// Ways it could fail, and the check that catches each:
// - Escape or focus loss leaves the machine `locked` or `free` → the next press asks no lock
// - a request the browser never answers blocks every later one → the timeout re-ask
// - a lock granted after the game stopped wanting it is kept → the stray-grant exit
// - NotSupportedError ends the gesture with no lock → the plain re-ask inside it
// - the DPR watch fires once and is never re-armed → the second monitor move
// - the layout map is ignored or unsorted → the AZERTY labels
// - `fill` stretches past aspect_max or under aspect_min → the 32:9 and 4:3 boxes
import { expect, test } from "bun:test";
import { ASK_TIMEOUT_MS, PointerLockMachine } from "../src/web/pointer-lock.ts";
import { LayoutLearner, readLayoutMap } from "../src/web/keyboard-layout.ts";
import { watchPixelRatio } from "../src/web/shell.ts";
import { layoutCanvas } from "../shell/page.ts";

test("pointer lock: a click locks, Escape re-arms it and the next click locks again, raw where granted", () => {
  const m = new PointerLockMachine();
  expect(m.want(true, true)).toEqual([]);
  expect(m.phase).toBe("armed");
  expect(m.press(0)).toEqual([{ kind: "request", raw: true }]);
  m.answered(true);
  expect(m.changed(true)).toEqual([]);
  expect([m.phase, m.rawActive]).toEqual(["locked", true]);
  // Escape: the browser releases it.
  m.changed(false);
  expect([m.phase, m.rawActive]).toEqual(["armed", false]);
  expect(m.press(2000)).toEqual([{ kind: "request", raw: true }]);
});

test("pointer lock: focus loss releases a held lock and drops an unanswered request, both re-armed", () => {
  const m = new PointerLockMachine();
  m.want(true, false);
  m.press(0);
  m.changed(true);
  expect(m.blurred()).toEqual([{ kind: "exit" }]);
  m.changed(false);
  expect(m.phase).toBe("armed");
  m.press(10);
  expect(m.phase).toBe("asking");
  expect(m.blurred()).toEqual([]);
  expect(m.phase).toBe("armed");
  expect(m.press(20)).toEqual([{ kind: "request", raw: false }]);
});

test("pointer lock: an unanswered request is asked again by a press after the timeout, a refused one by the next press", () => {
  const m = new PointerLockMachine();
  m.want(true, true);
  m.press(0);
  expect(m.press(ASK_TIMEOUT_MS - 1)).toEqual([]);
  expect(m.press(ASK_TIMEOUT_MS)).toEqual([{ kind: "request", raw: true }]);
  // Chrome refuses a request just after the player's Escape.
  expect(m.refused(ASK_TIMEOUT_MS + 5, false)).toEqual([]);
  expect(m.phase).toBe("armed");
  expect(m.press(ASK_TIMEOUT_MS + 10)).toEqual([{ kind: "request", raw: true }]);
});

test("pointer lock: unadjustedMovement refused is asked again plain inside the gesture, and stays plain", () => {
  const m = new PointerLockMachine();
  m.want(true, true);
  m.press(0);
  expect(m.refused(1, true)).toEqual([{ kind: "request", raw: false }]);
  expect(m.rawUnsupported).toBe(true);
  m.answered(true);
  m.changed(true);
  expect([m.phase, m.rawActive]).toEqual(["locked", false]);
  m.changed(false);
  expect(m.press(50)).toEqual([{ kind: "request", raw: false }]);
  // A browser that returns no promise ignored the option.
  const f = new PointerLockMachine();
  f.want(true, true);
  f.press(0);
  f.answered(false);
  expect([f.rawUnsupported, f.rawActive]).toEqual([true, false]);
});

test("pointer lock: a lock granted after the game stopped wanting it is released, and an unwanted lock asks nothing", () => {
  const m = new PointerLockMachine();
  m.want(true, true);
  m.press(0);
  expect(m.want(false, true)).toEqual([]);
  expect(m.changed(true)).toEqual([{ kind: "exit" }]);
  expect(m.phase).toBe("free");
  expect(m.press(5000)).toEqual([]);
  m.want(true, true);
  m.press(6000);
  m.changed(true);
  expect(m.want(false, true)).toEqual([{ kind: "exit" }]);
});

test("a monitor move to another device pixel ratio reports each new ratio, re-armed at it", () => {
  let dpr = 1;
  const queries: string[] = [];
  const listeners = new Set<() => void>();
  const media = (q: string) => {
    queries.push(q);
    return { addEventListener: (_: string, f: () => void) => listeners.add(f), removeEventListener: (_: string, f: () => void) => listeners.delete(f) } as unknown as MediaQueryList;
  };
  const seen: number[] = [];
  const stop = watchPixelRatio((d) => seen.push(d), { matchMedia: media, dpr: () => dpr });
  const fire = () => [...listeners].forEach((f) => f());
  dpr = 2;
  fire();
  dpr = 1.25;
  fire();
  expect(seen).toEqual([2, 1.25]);
  expect(queries).toEqual(["(resolution: 1dppx)", "(resolution: 2dppx)", "(resolution: 1.25dppx)"]);
  expect(listeners.size).toBe(1);
  stop();
  expect(listeners.size).toBe(0);
});

test("an AZERTY layout map labels the physical keys it prints, sorted; no map is null", async () => {
  const azerty = new Map([["KeyW", "z"], ["KeyQ", "a"], ["KeyA", "q"], ["Semicolon", "m"]]);
  const labels = await readLayoutMap({ keyboard: { getLayoutMap: async () => azerty } });
  expect(labels).toEqual([
    { code: "KeyA", label: "q" },
    { code: "KeyQ", label: "a" },
    { code: "KeyW", label: "z" },
    { code: "Semicolon", label: "m" },
  ]);
  expect(await readLayoutMap({})).toBeNull();
  expect(await readLayoutMap({ keyboard: { getLayoutMap: () => Promise.reject(new Error("SecurityError")) } })).toBeNull();
});

test("without a layout map, QWERTZ is learnt from unmodified presses only", () => {
  const l = new LayoutLearner();
  expect(l.learn({ code: "KeyY", key: "z" })).toBe(true);
  expect(l.learn({ code: "KeyY", key: "z" })).toBe(false);
  expect(l.learn({ code: "KeyZ", key: "Y", shiftKey: true })).toBe(false);
  expect(l.learn({ code: "KeyQ", key: "@", getModifierState: (k) => k === "AltGraph" })).toBe(false);
  expect(l.learn({ code: "Enter", key: "Enter" })).toBe(false);
  expect(l.learn({ code: "KeyZ", key: "y" })).toBe(true);
  expect(l.labels()).toEqual([
    { code: "KeyY", label: "z" },
    { code: "KeyZ", label: "y" },
  ]);
});

test("a fill canvas is pillarboxed past aspect_max at 32:9 and letterboxed under aspect_min at 4:3", () => {
  const none = { top: 0, right: 0, bottom: 0, left: 0 };
  const range = { min: 16 / 10, max: 21 / 9 };
  const wide = layoutCanvas({ width: 3840, height: 1080 }, none, "fill", null, range);
  expect([wide.left, wide.top, wide.width, wide.height]).toEqual([660, 0, 2520, 1080]);
  const square = layoutCanvas({ width: 1600, height: 1200 }, none, "fill", null, range);
  expect([square.left, square.top, square.width, square.height]).toEqual([0, 100, 1600, 1000]);
  // Inside the range (16:9) it fills; without one, 32:9 fills too.
  expect(layoutCanvas({ width: 1920, height: 1080 }, none, "fill", null, range)).toMatchObject({ left: 0, top: 0, width: 1920, height: 1080 });
  expect(layoutCanvas({ width: 3840, height: 1080 }, none, "fill", null)).toMatchObject({ width: 3840, height: 1080 });
});
