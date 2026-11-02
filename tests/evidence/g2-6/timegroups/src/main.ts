// The G2-6 time groups and formatting sample (batch 3).
// tests/evidence/g2-6/NOTES.md has the commands and what each frame shows.
//
// Two columns of five squares bob and pulse on looping tweens, each a
// little behind the one above, so a column reads as a wave. The right
// column has og.TimeGroup { group: 1 }:
// - 0-2 s: group 1 at 1, both columns move alike;
// - 2-4 s: og.time.setGroupScale(world, 1, 0.25): the right column at a quarter pace;
// - 4-5.5 s: scale 0, the right column frozen mid-wave;
// - 5.5-6 s: back to 1;
// - 6-7.5 s: og.time.setScale(world, 0.5): both columns at half pace (the
//   group multiplies the global scale);
// - 7.5-8 s: everything at 1.
// Under them, one locale per second (en-US, de-DE, fr-FR, es-ES) formats a
// compact number, a price, a date, a time and a duration from a fixed
// moment plus the tick: never a clock read, so every replay matches.

import { og, defineComponent, defineSystem, u32 } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};

/** What must replay: on the entity "show". */
const Show = defineComponent("Show", {
  /** The schedule's step taken last (0 none). */
  step: u32(0),
});

/** The schedule: [tick, group 1's scale, the global scale, caption]. */
const STEPS: Array<[number, number, number, string]> = [
  [1, 1, 1, "group 1 x 1"],
  [120, 0.25, 1, "group 1 x 0.25"],
  [240, 0, 1, "group 1 frozen"],
  [330, 1, 1, "group 1 x 1"],
  [360, 1, 0.5, "global x 0.5"],
  [450, 1, 1, "group 1 x 1"],
];
const LOCALES = ["en-US", "de-DE", "fr-FR", "es-ES"];
/** 2026-11-02 15:05:09 UTC. */
const MOMENT = 1793631909;

let caption = "";
let lines: string[] = [];

function square(world: World, x: number, y: number, k: number, group: boolean): Entity {
  const e = must(world.spawn({ name: `sq-${group ? "r" : "l"}${k}`, position: { x, y, z: 0 } }));
  const hue = group ? { r: 0.95, g: 0.45, b: 0.12 } : { r: 0.2, g: 0.6, b: 0.95 };
  must(world.insert(e, og.Sprite, { width: 1, height: 1, color: hue }));
  if (group) must(world.insert(e, og.TimeGroup, { group: 1 }));
  // A bob and a pulse, each a little behind the one above: a wave.
  must(og.tween.to(world, { entity: e, field: "position.x", from: [x - 1.5], to: [x + 1.5], duration: 1, delay: k * 0.15, ease: "quadInOut", loops: -1, yoyo: true }));
  must(og.tween.to(world, { entity: e, field: "scale", from: [0.6], to: [1.1], duration: 0.5, delay: k * 0.15, ease: "quadInOut", loops: -1, yoyo: true }));
  return e;
}

export const Schedule = defineSystem("Schedule", { reads: [og.TimeGroups], writes: [Show] }, (_q, _time, world) => {
  const tick = world.tick;
  const show = world.find("show")!;
  const i = STEPS.findIndex(([at]) => at === tick);
  if (i >= 0) {
    const [, group, global, text] = STEPS[i]!;
    must(og.time.setGroupScale(world, 1, group));
    must(og.time.setScale(world, global));
    caption = text;
    world.set(show, Show, { step: i + 1 });
  }
  if (tick % 60 === 0 || tick === 1) {
    const locale = LOCALES[Math.floor(tick / 60) % LOCALES.length]!;
    og.i18n.setLocale(world, locale);
    const at = MOMENT + tick;
    lines = [
      locale,
      og.i18n.formatCompact(world, 1234567 + tick * 1000),
      og.i18n.formatCurrency(world, 1234.5, "EUR"),
      `${og.i18n.formatDate(world, at, { style: "long" })}  ${og.i18n.formatTime(world, at, { style: "medium" })}`,
      og.i18n.formatDuration(world, 3600 + tick),
    ];
  }
});

export const Hud = defineSystem("Hud", {}, () => {
  // The caption at the top, the formatted lines at the bottom.
  og.ui.open("Root", { width: { kind: "percent", fraction: 1 }, height: { kind: "percent", fraction: 1 }, direction: "column", align: "spaceBetween", cross: "center", padding: { left: 0, top: 24, right: 0, bottom: 24 } } as never);
  og.ui.label("Caption", caption, { step: "displayXl", font: "display" } as never);
  og.ui.open("Lines", { direction: "column", cross: "center", gap: 4 } as never);
  for (const [k, line] of lines.entries()) og.ui.label(`Line${k}`, line, { step: k === 0 ? "lg" : "md", nowrap: true } as never);
  og.ui.close();
  og.ui.close();
});

export default og.game({
  tick: 60,
  seed: 1,
  systems: [Schedule, Hud],
  start(world: World) {
    const settings = must(world.resource(og.RenderSettings));
    settings.environmentIntensity = 0;
    settings.toneMapped = false;
    const cam = must(world.resource(og.Camera));
    cam.orthographic = true;
    cam.background = { r: 0.012, g: 0.012, b: 0.016 };
    cam.position = { x: 0, y: 0.5, z: 50 };
    cam.height = 16;
    const font = must(og.load.font(world, "fonts/Inter-Display.ttf"));
    og.ui.configure(world, { font, displayFont: font } as never);
    for (let k = 0; k < 5; k++) {
      square(world, -4, 4 - k * 1.6, k, false);
      square(world, 4, 4 - k * 1.6, k, true);
    }
    must(world.insert(must(world.spawn({ name: "show" })), Show));
  },
});
