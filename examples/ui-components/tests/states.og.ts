// Every state a film names is the state on screen: acts fired seconds
// into a run (as the films fire them) land on the frame they are given, so
// a killfeed line, a hit marker, a headshot, a critical and an expired
// timer, a broken combo and a unit frame's hit all show when their shot is
// taken. Ways it could fail: a command stamping its act with og.ui.tick()
// between ticks (0 there), so a line is born expired and a timer reads the
// screen's clock from the run's start; a crit hidden behind an older hit.
// At 200 % text a label picking the step the doubled ramp already reaches
// (so it does not grow), or a HUD piece crowding another off a phone.
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
  // The strategy screen on a phone, at 100 % and at the player's 200 % text.
  "27.30:viewport:phone",
  "27.35:show:strategy",
  "28.60:dump_ui:phone100.json",
  "28.70:text:200",
  "29.90:dump_ui:phone200.json",
  // The unit frames at 200 % on a phone, then opened afresh on a desktop: a side stack that has just appeared
  // is placed by its own size from its first ticks (once a tall party lay half a screen low for a tick).
  "30.00:show:frames",
  "31.20:dump_ui:framesPhone200.json",
  "31.30:viewport:desktop",
  "31.35:show:abilities",
  "31.45:show:frames",
  "31.48:dump_ui:framesDesktop200.json",
  "31.60:quit",
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
    const title = "QuestsHud/Quests/main/Head/Title";
    const h100 = at(run, "phone100.json", title)?.height ?? 0;
    const h200 = at(run, "phone200.json", title)?.height ?? 0;
    t.ok(h100 > 0 && h200 >= h100 * 1.6, `the quest title grows with the player's text size (${h100} px, then ${h200} px)`);
    for (const file of ["phone100.json", "phone200.json"]) {
      const roots = els(run, file).filter((e) => /^(Res|Quests|Turns|Combo)Hud$/.test(e.path) && e.width > 0);
      t.equal(roots.length, 4, `${file}: the resources, tracker, turn order and combo are all shown`);
      t.ok(roots.every((e) => e.x >= 0 && e.x + e.width <= 390 && e.y >= 47 && e.y + e.height <= 844 - 34), `${file}: each lies inside the phone's safe area`);
      const overlap = (a: El, b: El) => a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height;
      t.ok(roots.every((a, i) => roots.every((b, j) => i >= j || !overlap(a, b))), `${file}: none covers another`);
    }
    // The player's and the boss's frames never share space on a phone at 200 %: every part of one clear of every part of the other.
    const parts = (file: string, root: string) => els(run, file).filter((e) => e.path.startsWith(`${root}/`) && e.width > 0 && e.height > 0);
    const meet = (a: El, b: El) => a.x < b.x + b.width - 0.5 && b.x < a.x + a.width - 0.5 && a.y < b.y + b.height - 0.5 && b.y < a.y + a.height - 0.5;
    const player = parts("framesPhone200.json", "PlayerHud");
    const boss = parts("framesPhone200.json", "TargetHud");
    const hits = player.flatMap((a) => boss.filter((b) => meet(a, b)).map((b) => `${a.path} × ${b.path}`));
    t.ok(player.length > 0 && boss.length > 0, "both frames are shown on the phone at 200 %");
    t.equal(hits.slice(0, 3).join("; "), "", "the player's and the boss's frames meet nowhere on the phone at 200 %");
    t.ok(player.every((e) => e.x >= 0 && e.x + e.width <= 390.5), "the player's frame lies across the phone, none of it past the edge");
    const party = parts("framesDesktop200.json", "PartyHud");
    t.ok(party.length > 0 && party.every((e) => e.y >= 0 && e.y + e.height <= 1080.5), "on a desktop at 200 % the party lies on screen, its last member too");
  },
});
