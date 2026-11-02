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
  "31.22:screenshot:framesPhone200.png",
  "31.30:viewport:desktop",
  "31.35:show:abilities",
  "31.45:show:frames",
  "31.48:dump_ui:framesDesktop200.json",
  // The frames on a desktop at 100 % on the pale test ground, for the contrast of their names.
  "31.50:text:100",
  "32.70:dump_ui:framesDesktop.json",
  "32.72:screenshot:framesDesktop.png",
  // The racing screen in Arabic: its words come from the string tables, not left in English.
  "32.80:show:racing",
  "32.85:lang:ar",
  "33.40:dump_ui:raceAr.json",
  "33.50:quit",
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
    // Each frame's live name holds against its outline on a pale green ground: the glyphs' ink
    // (the 99th percentile of the box's relative luminance) against their outline. Live text under 3:1 is an error (docs/ui.md §15.4); dimmed names, class lines
    // and other text over the scene are judged by eye, not by a number.
    for (const [shot, dump] of [["framesDesktop.png", "framesDesktop.json"], ["framesPhone200.png", "framesPhone200.json"]] as const) {
      t.frame(shot, () => {
        const img = run.pixels(shot);
        // The run's window is 1920x1080; an emulated phone (390x844) is drawn scaled to the window's height at its corner.
        const k = shot.startsWith("framesPhone") ? img.height / 844 : img.width / 1920;
        const lin = (v: number) => (v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4);
        const lum = (i: number) => 0.2126 * lin(img.rgba[i]! / 255) + 0.7152 * lin(img.rgba[i + 1]! / 255) + 0.0722 * lin(img.rgba[i + 2]! / 255);
        const labels = els(run, dump).filter((e) => /^(PlayerHud|TargetHud|PartyHud)\/.*\/Name$/.test(e.path) && !/\/(dead|off)\//.test(e.path) && e.text && e.width > 4);
        t.ok(labels.length >= 3, `${shot}: the frames' names are on screen`);
        for (const el of labels) {
          // The glyphs' own run (a label's box spans the bars): its text's width at the box's start (its end in RTL).
          const w = Math.min(el.width, el.text.length * (el.height / 1.3) * 0.56 + 4);
          const e = { ...el, w, x0: el.x };
          const ls: number[] = [];
          for (let y = Math.max(0, Math.floor(e.y * k)); y < Math.min(img.height, Math.ceil((e.y + e.height) * k)); y++) {
            for (let x = Math.max(0, Math.floor(e.x0 * k)); x < Math.min(img.width, Math.ceil((e.x0 + e.w) * k)); x++) ls.push(lum((y * img.width + x) * 4));
          }
          ls.sort((a, b) => a - b);
          // Glyphs cover a fifth of their run or less: their ink is the brightest 2 %.
          const ink = ls[Math.floor(ls.length * 0.98)] ?? 0;
          // Names sit on the scene with an outline round their glyphs and no band: a live name reads against its outline
          // (the darkest 1 % of its run: a 1.5 px outline is a thin, antialiased line), whatever the scene; the scene is judged by eye.
          const ground = ls[Math.floor(ls.length * 0.01)] ?? 0;
          const ratio = (Math.max(ink, ground) + 0.05) / (Math.min(ink, ground) + 0.05);
          const floor = 3;
          t.ok(ratio >= floor, `${shot}: "${e.text}" reads at ${ratio.toFixed(2)}:1 on its ground (at least ${floor}:1)`);
        }
      });
    }
    for (const [path, word] of [["LapTimeHud/LapTime/Head/Position/Caption", "المركز"], ["LapTimeHud/LapTime/Best/Label", "أفضل لفة"], ["LapTimeHud/LapTime/Last/Label", "اللفة السابقة"], ["TowerHud/Tower/moreno/Gap", "المتصدر"], ["TowerHud/Tower/alvarez/Gap", "صيانة"]] as const) {
      t.equal(at(run, "raceAr.json", path)?.text, word, `in Arabic ${path} reads from the string table`);
    }
    const party = parts("framesDesktop200.json", "PartyHud");
    t.ok(party.length > 0 && party.every((e) => e.y >= 0 && e.y + e.height <= 1080.5), "on a desktop at 200 % the party lies on screen, its last member too");
  },
});
