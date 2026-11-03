// The genre showcases each in its own pack (docs/ui.md §17, src/showcases.ts):
// every page of a family in its primary seed style, then its first page in a
// light pack and in a dark one, composed into one reference sheet per family
// and surface (styles.ts' sheets): on a desktop
// crates/og-ui/references/<tier>/kit-showcase-<family>.png, on a phone in
// portrait kit-showcase-<family>-phone.png. The style sheets put every
// showcase HUD and document in all twelve seeds; these show each family as
// its game would ship it, the pattern pages included.

import { playtest, type PlaytestDesc } from "@origingame/engine/playtest";
import { SHOWCASES, type Showcase } from "../src/showcases.ts";
import { STYLE_SETTLE } from "./script.ts";
import { sheetCheck, SURFACES, unsettled, withSettleDumps } from "./styles.ts";

/** The light and dark packs a family is checked in besides its own. */
const LIGHT = (s: Showcase): string => s.light ?? (s.pack === "storybook" ? "kawaii" : "storybook");
const DARK = (s: Showcase): string => (s.pack === "dread" ? "fantasy" : "dread");

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
  return { commands: viewport + withSettleDumps(steps.join(";")), families };
}

/** The playtest of every showcase on a surface. */
export function showcasesPlaytest(on: "desktop" | "phone"): PlaytestDesc {
  const surface = SURFACES[on];
  const { commands, families } = script(surface.viewport);
  return playtest({
    name: `every genre showcase in its primary pack, a light and a dark one${on === "phone" ? " on a phone" : ""} matches its cell of kit-showcase-<family>${surface.suffix}.png`,
    size: surface.size,
    commands,
    check(run, t) {
      t.equal(run.warnings.join("; "), "", "the script ran without warnings");
      t.ok(run.quit, "the script reached quit");
      for (const [s, files] of families) {
        for (const f of files) t.equal(unsettled(run, f.replace(/\.png$/, "")), "", `${f} is captured with every entrance and exit settled`);
        sheetCheck(run, t, files, `kit-showcase-${s.key}${surface.suffix}.png`, surface, s.key);
      }
    },
  });
}
