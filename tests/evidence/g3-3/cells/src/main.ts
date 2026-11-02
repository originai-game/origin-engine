// The G3-3 world cells sample. tests/evidence/g3-3/NOTES.md has the
// commands and what each frame shows.
//
// The rover (scenes/world.ogt's og.StreamAnchor) drives along the road at
// 12 m/s from x = -36, so it passes the five cells' boxes (24 m wide, 32 m
// apart) one after another. The engine loads a cell the tick the rover is
// within 12 m of its box and unloads it once the rover is 15 m past: its
// slab, houses or trees and its crates or balls appear and go, and the
// crates fall onto the slab's collider the tick they spawn. The camera
// follows the rover from behind and above. The caption lists every cell's
// state as the engine's og.CellEvent reported it, with the tick.

import { og, defineSystem } from "@origingame/engine";
import type { World } from "@origingame/engine";

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};

/** The rover's speed, metres per second, and where it starts. */
const SPEED = 12;
const START = -36;

/** Per cell entity: its name and last event, for the caption. */
const states = new Map<number, string>();
let caption = "";

export const Drive = defineSystem("Drive", { writes: [og.Transform, og.Camera] }, (_q, time, world) => {
  const rover = world.find("rover");
  if (rover === null) return;
  const x = START + SPEED * time.elapsed;
  world.set(rover, og.Transform, { position: { x, y: 0, z: 0 } });
  const cam = must(world.resource(og.Camera));
  const eye = { x: x - 14, y: 22, z: 34 };
  cam.position = eye;
  cam.quaternion = og.transform.lookAt(eye, { x: x + 10, y: 0, z: 0 });
  cam.fov = 50;
});

export const Report = defineSystem("Report", { reads: [og.CellEvent, og.StreamCell] }, (_q, _time, world) => {
  for (const ev of world.events(og.CellEvent)) {
    const name = world.nameOf(ev.cell) ?? `${ev.cell}`;
    states.set(ev.cell, `${name} ${ev.loaded ? "loaded" : "unloaded"} (${ev.entities}) at ${world.tick}`);
  }
  const rover = world.find("rover");
  const x = rover === null ? 0 : START + SPEED * (world.tick / 60);
  caption = `tick ${world.tick}  rover x ${x.toFixed(1)}`;
});

export const Hud = defineSystem("Hud", {}, () => {
  og.ui.open("Root", { width: { kind: "percent", fraction: 1 }, height: { kind: "percent", fraction: 1 }, direction: "column", align: "start", cross: "start", padding: { left: 24, top: 20, right: 24, bottom: 20 }, gap: 4 } as never);
  og.ui.label("Caption", caption, { step: "lg", nowrap: true } as never);
  let k = 0;
  for (const line of [...states.entries()].sort((a, b) => a[0] - b[0]).map(([, s]) => s)) {
    og.ui.label(`Cell${k++}`, line, { step: "md", nowrap: true } as never);
  }
  og.ui.close();
});

export default og.game({
  tick: 60,
  seed: 1,
  systems: [Drive, Report, Hud],
  start(world: World) {
    const settings = must(world.resource(og.RenderSettings));
    settings.sky = true;
    const font = must(og.load.font(world, "fonts/Inter-Display.ttf"));
    og.ui.configure(world, { font, displayFont: font } as never);
  },
});
