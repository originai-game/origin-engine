// Compares `dump_ui:<screen>.json` steps with shipped.ts: the same
// elements (parent, id, kind, role, text) in the same order, each within a
// few px of its recorded rectangle (text shaping may move a width by a few).
import type { PlaytestRun } from "@origingame/engine/playtest";
import { SHIPPED } from "./shipped.ts";

type Row = [parent: string, id: string, kind: string, role: string, text: string, x: number, y: number, width: number, height: number];
interface Checker { ok(cond: boolean, message: string): void }


export function checkScreens(run: PlaytestRun, t: Checker, screens: readonly string[]): void {
  for (const screen of screens) {
    const want = SHIPPED[screen]! as Row[];
    const got: Row[] = run.json<{ elements: Array<{ parent: string; id: string; kind: string; role: string; text: string; x: number; y: number; width: number; height: number }> }>(`${screen}.json`).elements
      .map((e) => [e.parent, e.id, e.kind, e.role, e.text, Math.round(e.x), Math.round(e.y), Math.round(e.width), Math.round(e.height)]);
    const name = (r: Row): string => `${r[0]} > ${r[1]} [${r[2]}/${r[3]}] ${JSON.stringify(r[4])}`;
    t.ok(JSON.stringify(got.map(name)) === JSON.stringify(want.map(name)), `${screen}: the elements as shipped (${got.map(name).filter((n) => !want.map(name).includes(n)).join("; ") || "same names"} | missing ${want.map(name).filter((n) => !got.map(name).includes(n)).join("; ") || "none"})`);
    const moved = want.flatMap((w, i) => {
      const g = got[i];
      if (!g || name(g) !== name(w)) return [];
      // Positions and heights within 3 px, widths within 8 (a heading's text shaping).
      const off = [Math.abs(g[5] - w[5]), Math.abs(g[6] - w[6]), Math.abs(g[7] - w[7]) / 8 * 3, Math.abs(g[8] - w[8])].some((d) => d > 3);
      return off ? [`${name(w)} at ${g.slice(5).join(",")}, shipped ${w.slice(5).join(",")}`] : [];
    });
    t.ok(moved.length === 0, `${screen}: every element where it shipped${moved.length ? `: ${moved.join("; ")}` : ""}`);
  }
}
