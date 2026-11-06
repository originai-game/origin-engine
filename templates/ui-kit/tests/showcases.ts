// Every showcase page in its primary, light and dark packs must settle its
// entrances and exits. Dumps retain this behavior coverage; representative
// pixel checks live in the individual kit playtests, not duplicate sheets.

import type { PlaytestDesc } from "@origingame/engine/playtest";
import { SHOWCASES, type Showcase } from "../src/showcases.ts";
import { STYLE_SETTLE } from "./script.ts";
import { SURFACES, unsettled, withSettleDumps } from "../../styles/tests/styles.ts";
import { playtest } from "./lib.ts";
import { selectReferenceFrames } from "./reference-frames.ts";

/** The light and dark packs a family is checked in besides its own. */
const LIGHT = (s: Showcase): string => s.light ?? (s.pack === "storybook" ? "kawaii" : "storybook");
const DARK = (s: Showcase): string => s.dark ?? (s.pack === "dread" ? "fantasy" : "dread");

/** The frames of a family, in its sheet's order: [style, screen, file]. */
function shots(s: Showcase): Array<[string, string, string]> {
  const own = s.pages.map((p, i): [string, string, string] => [s.pack, p.screen, `sc-${s.key}-${i}-${p.screen}.png`]);
  const first = s.pages[0]!.screen;
  return [...own, [LIGHT(s), first, `sc-${s.key}-light-${first}.png`], [DARK(s), first, `sc-${s.key}-dark-${first}.png`]];
}

/** Every family's frames in one run: the pack set, the preview state reset, then each page shown and taken once settled. */
function script(viewport: string): { commands: string; families: Array<[Showcase, string[]]> } {
  const settle = STYLE_SETTLE.default ?? 1;
  const steps: string[] = [];
  const families: Array<[Showcase, string[]]> = [];
  let t = 0.3;
  let style = "";
  for (const s of SHOWCASES) {
    const files: string[] = [];
    for (const [pack, screen, file] of shots(s)) {
      if (pack !== style) {
        steps.push(`${t.toFixed(2)}:style:${pack}`);
        style = pack;
      }
      steps.push(`${(t + 0.05).toFixed(2)}:show:${screen}`, `${(t + 0.1).toFixed(2)}:pat:reset`, `${(t + 0.1).toFixed(2)}:sc:reset`);
      t += 0.1 + settle;
      steps.push(`${t.toFixed(2)}:screenshot:${file}`);
      files.push(file);
      t += 0.1;
    }
    families.push([s, files]);
  }
  steps.push(`${t.toFixed(2)}:quit`);
  const { commands } = selectReferenceFrames({ commands: viewport + withSettleDumps(steps.join(";")) }, "kit-showcase-");
  return { commands, families };
}

/** The playtest of every showcase on a surface. */
export function showcasesPlaytest(on: "desktop" | "phone"): PlaytestDesc {
  const surface = SURFACES[on];
  const { commands, families } = script(surface.viewport);
  return playtest({
    name: `every genre showcase in its primary pack, a light and a dark one${on === "phone" ? " on a phone" : ""} settles its entrances and exits`,
    size: surface.size,
    commands,
    check(run, t) {
      t.equal(run.warnings.join("; "), "", "the script ran without warnings");
      t.ok(run.quit, "the script reached quit");
      for (const [, files] of families) {
        for (const f of files) t.equal(unsettled(run, f.replace(/\.png$/, "")), "", `${f} is captured with every entrance and exit settled`);
      }
    },
  });
}
