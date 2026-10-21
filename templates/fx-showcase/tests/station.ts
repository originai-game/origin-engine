// One E5 station as a playtest: show it, wait for its effects and physics,
// take its frame at 1280×720 and compare it with references/<tier>/<name>.png
// (frames.ts' rule), dump the entities its checks read, and run the
// station's own checks. A second frame half a second earlier lets a check
// see what moves.
//
// Without a renderer (tier `none`) the two frames are reported unchecked:
// a station's checks read its dumped fields first and its frames after, so
// they run up to their first frame read and stop there, every field check
// made.
import { FrameUnchecked, playtest, type PlaytestChecks, type PlaytestRun } from "@origingame/engine/playtest";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { checkFrames, decodePng, outDir, type Image } from "./frames.ts";

export interface Dumped {
  components: Record<string, Record<string, unknown>>;
}

/** Mean RGB over a rectangle of a frame. */
export function mean(img: Image, x0: number, y0: number, x1: number, y1: number): [number, number, number] {
  const sum = [0, 0, 0];
  for (let y = y0; y < y1; y++) {
    for (let x = x0; x < x1; x++) {
      for (let k = 0; k < 3; k++) sum[k]! += img.rgba[(y * img.width + x) * 4 + k]!;
    }
  }
  const n = (x1 - x0) * (y1 - y0);
  return [sum[0]! / n, sum[1]! / n, sum[2]! / n];
}

/** Mean absolute difference of two frames over a rectangle (0..255). */
export function moved(a: Image, b: Image, x0 = 0, y0 = 0, x1 = a.width, y1 = a.height): number {
  let sum = 0;
  for (let y = y0; y < y1; y++) {
    for (let x = x0; x < x1; x++) {
      for (let k = 0; k < 3; k++) {
        const i = (y * a.width + x) * 4 + k;
        sum += Math.abs(a.rgba[i]! - b.rgba[i]!);
      }
    }
  }
  return sum / ((x1 - x0) * (y1 - y0) * 3);
}

/** Pixels whose brightest channel changed by more than `by` between two frames. */
export function changed(a: Image, b: Image, by = 30): number {
  let n = 0;
  for (let p = 0; p < a.width * a.height; p++) {
    let d = 0;
    for (let k = 0; k < 3; k++) d = Math.max(d, Math.abs(a.rgba[p * 4 + k]! - b.rgba[p * 4 + k]!));
    if (d > by) n++;
  }
  return n;
}

export interface Station {
  run: PlaytestRun;
  t: PlaytestChecks;
  /** The station's frame and the one half a second before it. */
  frame: Image;
  before: Image;
  /** A field of a dumped entity's component (`Transform`, `og.Transform`, a game component). */
  field<T = unknown>(entity: string, component: string, field: string): T;
}

/**
 * A playtest of `station`: shown at 0.5 s, its frame after `wait` seconds
 * (and one 0.5 s earlier), the named entities dumped, then `checks`.
 */
export function stationPlaytest(station: string, wait: number, entities: readonly string[], checks: (s: Station) => void) {
  const at = 0.5 + wait;
  const script = [
    `0.5:station:${station}`,
    `${(at - 0.5).toFixed(2)}:screenshot:${station}-before.png`,
    `${at.toFixed(2)}:screenshot:${station}.png`,
    ...entities.map((e, i) => `${(at + 0.01 * (i + 1)).toFixed(2)}:dump_entity:${e}:${e}.json`),
    `${(at + 0.2).toFixed(2)}:quit`,
  ];
  return playtest({
    name: `the ${station} station matches its reference and behaves`,
    size: "1280x720",
    commands: script.join(";"),
    check(run, t) {
      t.equal(run.warnings.join("; "), "", "the script ran without warnings");
      t.ok(run.quit, "the script reached quit");
      const shots = [`${station}.png`, `${station}-before.png`];
      // With a renderer the frames themselves; without one, images whose every read throws FrameUnchecked.
      const load = (f: string): Image =>
        run.rendered
          ? decodePng(readFileSync(join(outDir(), f)))
          : (new Proxy({} as Image, {
              get() {
                throw new FrameUnchecked(f);
              },
            }) as Image);
      const field = <T,>(entity: string, component: string, field: string): T => {
        const c = run.json<Dumped>(`${entity}.json`).components;
        const v = (c[component] ?? c[`og.${component}`])?.[field];
        if (v === undefined) throw new Error(`${entity} has no ${component}.${field} (it has ${Object.keys(c).join(", ")})`);
        return v as T;
      };
      try {
        checks({ run, t, frame: load(shots[0]!), before: load(shots[1]!), field });
      } catch (e) {
        if (run.rendered || !(e instanceof Error && e.name === "FrameUnchecked")) throw e;
      }
      t.frame(shots, () => {});
      checkFrames(run, t, [`${station}.png`], "");
    },
  });
}
