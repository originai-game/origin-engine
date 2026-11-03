// The screens the frames playtests show, and the command script that shows
// each and takes its frame: `show:<screen>`, then a screenshot once its
// entrance has settled. Beside the gallery, the widget sheet, the toasts and
// the materials, the genre showcases' pages (src/showcases.ts
// SHOWCASE_SCREENS): the ejected screen documents (title, pause, tutorial,
// loading, results, scoreboard, leaderboard) and the HUDs built from the
// HUD components. The pattern screens have their own playtests (patterns.ts).

import { SHOWCASE_SCREENS } from "../src/showcases.ts";

export const FRAME_SCREENS = ["gallery", "widgets", ...SHOWCASE_SCREENS, "toasts", "materials"] as const;

const SETTLE: Record<string, number> = {};

/**
 * The style playtests' wait after each `show:`: every entrance settled in
 * every pack. A layer's entrance and the stagger of its controls together
 * take at most two of the pack's base durations (the longest, 380 ms: 0.76
 * s), the toasts start 0.1 s in. styles.ts checks each capture against it.
 */
export const STYLE_SETTLE: Record<string, number> = { default: 1.0, toasts: 1.2 };

/**
 * The script and the files it writes, starting at `t0` seconds; `settle`
 * the wait after each screen's `show:` (by screen, with a `default`).
 */
export function framesScript(t0: number, settle: Record<string, number> = SETTLE): { commands: string; files: string[]; end: number } {
  const steps: string[] = [];
  const files: string[] = [];
  let t = t0;
  for (const s of FRAME_SCREENS) {
    steps.push(`${t.toFixed(2)}:show:${s}`);
    if (s === "toasts") {
      steps.push(`${(t + 0.05).toFixed(2)}:toast:Checkpoint reached`);
      steps.push(`${(t + 0.1).toFixed(2)}:toast:Achievement unlocked`);
    }
    const at = t + (settle[s] ?? settle.default ?? 0.6);
    steps.push(`${at.toFixed(2)}:screenshot:${s}.png`);
    files.push(`${s}.png`);
    t = at + 0.1;
  }
  steps.push(`${t.toFixed(2)}:quit`);
  return { commands: steps.join(";"), files, end: t };
}
