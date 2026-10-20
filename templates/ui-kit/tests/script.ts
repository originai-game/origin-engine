// The screens the frames playtests show, and the command script that shows
// each and takes its frame: `show:<screen>`, then a screenshot once its
// entrance has settled (longer for the results' stars and count-up and the
// cutscene's typing).

export const FRAME_SCREENS = [
  "gallery", "widgets", "mainMenu", "pause", "settings", "platformer", "shooter", "puzzle", "idle", "racing",
  "cutscene", "shop", "inventory", "levels", "results", "leaderboard", "tutorial", "loading", "confirm", "alert", "toasts",
  "materials",
] as const;

const SETTLE: Record<string, number> = { results: 1.6, cutscene: 1.8 };

/** The script and the files it writes, starting at `t0` seconds. */
export function framesScript(t0: number): { commands: string; files: string[]; end: number } {
  const steps: string[] = [];
  const files: string[] = [];
  let t = t0;
  for (const s of FRAME_SCREENS) {
    steps.push(`${t.toFixed(2)}:show:${s}`);
    if (s === "toasts") {
      steps.push(`${(t + 0.05).toFixed(2)}:toast:Checkpoint reached`);
      steps.push(`${(t + 0.1).toFixed(2)}:toast:Achievement unlocked`);
    }
    const at = t + (SETTLE[s] ?? 0.6);
    steps.push(`${at.toFixed(2)}:screenshot:${s}.png`);
    files.push(`${s}.png`);
    t = at + 0.1;
  }
  steps.push(`${t.toFixed(2)}:quit`);
  return { commands: steps.join(";"), files, end: t };
}
