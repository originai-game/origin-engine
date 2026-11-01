// Every state a film names is the state on screen: acts fired seconds
// into a run (as the films fire them) land on the frame they are given, so
// a killfeed line, a hit marker, a headshot, a critical and an expired
// timer, a broken combo and a unit frame's hit all show when their shot is
// taken. Ways it could fail: a command stamping its act with og.ui.tick()
// between ticks (0 there), so a line is born expired and a timer reads the
// screen's clock from the run's start; a crit hidden behind an older hit.
import { playtest, type PlaytestRun } from "@origingame/engine/playtest";

interface El {
  path: string;
  text: string;
  width: number;
  height: number;
  x: number;
  y: number;
  visualStates?: Record<string, string>;
}

const els = (run: PlaytestRun, file: string): El[] => run.json<{ elements: El[] }>(file).elements;
const at = (run: PlaytestRun, file: string, path: string): El | undefined => els(run, file).find((e) => e.path === path);
const state = (run: PlaytestRun, file: string, path: string, group: string): string => at(run, file, path)?.visualStates?.[group] ?? "";

// Eight idle seconds first: an act stamped at tick 0 would be long gone by then.
const steps = [
  "0.10:viewport:desktop",
  "8.00:show:shooter",
  "9.00:act:hit",
  "9.05:dump_ui:hit.json",
  "9.50:act:crit",
  "9.55:dump_ui:crit.json",
  "10.20:dump_ui:feed.json",
  "20.20:dump_ui:critical.json",
  "23.20:dump_ui:expired.json",
  "24.00:show:strategy",
  "25.00:act:hit",
  "25.50:act:break",
  "25.75:dump_ui:combo.json",
  "26.00:show:frames",
  "27.00:act:hit",
  "27.05:dump_ui:frames.json",
  "27.30:quit",
];

export default playtest({
  name: "the states the films name are on screen when their shots are taken",
  size: "1920x1080",
  commands: steps.join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(state(run, "hit.json", "AimHud/Aim", "hit"), "hit", "a hit marks the crosshair the tick after it");
    t.equal(state(run, "crit.json", "AimHud/Aim", "hit"), "headshot", "a crit after a hit marks a headshot");
    const lines = els(run, "feed.json").filter((e) => /^FeedHud\/Feed\/k\d$/.test(e.path));
    t.equal(lines.map((e) => e.path), ["FeedHud/Feed/k1", "FeedHud/Feed/k0"], "both kills are in the feed, newest first");
    t.ok(lines.every((e) => e.width > 80 && e.x >= 0 && e.x + e.width <= 1920 && e.y >= 0), "each line is laid out on screen");
    t.equal(state(run, "critical.json", "RoundHud/Round", "time"), "critical", "the round timer is critical in its last seconds");
    t.equal(state(run, "expired.json", "RoundHud/Round", "time"), "done", "and done at zero");
    t.equal(at(run, "expired.json", "RoundHud/Round/Body/Time")?.text, "0.0", "reading 0.0");
    t.equal(state(run, "combo.json", "ComboHud/Combo", "combo"), "broken", "a break breaks the combo");
    t.equal(state(run, "frames.json", "PlayerHud/Player", "hurt"), "hit", "a hit shows on the player's frame");
  },
});
