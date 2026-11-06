// Compares `dump_ui:<screen>.uicheck.json` steps with shipped.ts: the same
// elements (parent, id, kind, role, text) in the same order; and og ui
// check's reading of the same frame (@origingame/engine/ui-check) finds no
// undeclared overlap, no element laid out past the screen and no text cut.
// Where each element sits is not pinned to the pixel: the frames
// (tests/frames.og.ts) hold the look, these rules what a player would see
// break (the owner, 2026-11-06).
import { checkUi } from "@origingame/engine/ui-check";
import type { PlaytestRun } from "@origingame/engine/playtest";
import { SHIPPED } from "./shipped.ts";

type Row = [parent: string, id: string, kind: string, role: string, text: string];
interface Checker { ok(cond: boolean, message: string): void }

/** og ui check's rules a shipped screen must pass (docs/ui.md §16.5). */
const RULES = ["overlap", "offscreen", "text-cut"];

export function checkScreens(run: PlaytestRun, t: Checker, screens: readonly string[]): void {
  for (const screen of screens) {
    const want = SHIPPED[screen]! as Row[];
    const got: Row[] = run.json<{ elements: Array<{ parent: string; id: string; kind: string; role: string; text: string }> }>(`${screen}.uicheck.json`).elements
      .map((e) => [e.parent, e.id, e.kind, e.role, e.text]);
    const name = (r: Row): string => `${r[0]} > ${r[1]} [${r[2]}/${r[3]}] ${JSON.stringify(r[4])}`;
    t.ok(JSON.stringify(got.map(name)) === JSON.stringify(want.map(name)), `${screen}: the elements as shipped (${got.map(name).filter((n) => !want.map(name).includes(n)).join("; ") || "same names"} | missing ${want.map(name).filter((n) => !got.map(name).includes(n)).join("; ") || "none"})`);
    const found = checkUi(run, screen, { skip: ["contrast"] }).filter((f) => RULES.includes(f.rule));
    t.ok(found.length === 0, `${screen}: no undeclared overlap, nothing past the screen, no text cut${found.length ? `: ${found.map((f) => `${f.rule} ${f.paths.join(" × ")} (${f.measured})`).join("; ")}` : ""}`);
  }
}
