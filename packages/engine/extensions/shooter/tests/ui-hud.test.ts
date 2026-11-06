import { expect, test } from "bun:test";

import { og } from "@origingame/engine";

import { at, dump, engineFor } from "../../../tests/support/ui.ts";

import { shooter, type KillfeedEntry } from "@origingame/engine/extensions/shooter/index.ts";

import { states, below, kitEngine, DESKTOP_SAFE } from "../../../tests/support/hud.ts";

import "../../../tests/support/hud.ts";

test("shooter.crosshair and ammoCounter: a hit marker flashes and leaves; low, empty and an infinite reserve", () => {
  let hitAt: number | null = null;
  let mag = 7;
  const e = engineFor("HudShooter", () => {
    shooter.crosshair("Aim", { spread: 4, hitAt, hit: "headshot" });
    shooter.ammoCounter("Ammo", { magazine: mag, magazineMax: 30, reserve: null, weapon: "R-301", reloadHint: "R Reload" });
  });
  e.advance(3);
  let els = dump(e);
  expect(at(els, "AimHud/Aim/Marker")).toBe(undefined);
  expect(states(at(els, "AmmoHud/Ammo")).ammo).toBe("low");
  expect(at(els, "AmmoHud/Ammo/Count/Reserve")?.text).toBe("∞");
  expect(at(els, "AmmoHud/Ammo/Hint")?.text).toBe("R Reload");
  expect(below(els, "AmmoHud/Ammo/Rounds").length).toBe(30);
  hitAt = og.ui.tick();
  mag = 0;
  e.advance(2);
  els = dump(e);
  expect(at(els, "AimHud/Aim/Marker")).toBeDefined();
  expect(states(at(els, "AimHud/Aim")).hit).toBe("headshot");
  expect(states(at(els, "AmmoHud/Ammo")).ammo).toBe("empty");
  e.advance(40);
  expect(at(dump(e), "AimHud/Aim/Marker")).toBe(undefined);
});

// The headshot's icon (target) is not in the atlas: its fallback mark is the name's initial, an identifier's letter,
// so the probe reads the letter drawn (T), as it reads every locale's (a locale-cased source letter read t in ja).
test("a missing icon's fallback mark reads its drawn initial in any locale", () => {
  let set = false;
  const e = kitEngine("HudMarkTr", (w) => {
    if (!set) {
      og.i18n.setLocale(w, "tr");
      set = true;
    }
    shooter.killfeed("Feed", { entries: [{ key: "k", killer: "Ana", victim: "Bo", weaponText: "R-301", headshot: true, at: 0, killerTeam: 0, victimTeam: 1 }], max: 4, durationMs: 60_000 });
  });
  e.advance(3);
  const mark = at(dump(e), "FeedHud/Feed/k/Headshot/Mark");
  expect([mark?.text, mark?.name]).toEqual(["T", "T"]);
  e.destroy();
});

test("shooter.killfeed: lines stamped late in a run are laid out on screen, inside the safe area, newest on top", () => {
  const entries: KillfeedEntry[] = [];
  let push: string[] = [];
  const e = engineFor("HudFeedLate", () => {
    for (const k of push) entries.push({ key: k, killer: "Wraith", killerTeam: 0, victim: "Octane", victimTeam: 1, weaponText: "R-99", at: og.ui.tick() });
    push = [];
    shooter.killfeed("Feed", { entries });
  }, { ...DESKTOP_SAFE });
  e.advance(500);
  push = ["a"];
  e.advance(20);
  push = ["b"];
  e.advance(20);
  const els = dump(e);
  const a = at(els, "FeedHud/Feed/a")!;
  const b = at(els, "FeedHud/Feed/b")!;
  for (const line of [a, b]) {
    expect(line.width).toBeGreaterThan(80);
    expect(line.height).toBeGreaterThan(10);
    expect(line.x).toBeGreaterThanOrEqual(0);
    expect(line.y).toBeGreaterThanOrEqual(40);
    expect(line.x + line.width).toBeLessThanOrEqual(1280 - 30);
  }
  expect(b.y).toBeLessThan(a.y);
  expect(at(els, "FeedHud/Feed/b/Killer")?.text).toBe("Wraith");
});
