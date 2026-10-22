// The platformer HUD draws what the game gives it and nothing else
// (Brasswick's finding #4), on the native host: a puzzle-platformer with no
// lives and no score (lives 0, maxLives 0, its gears as coins) gets no
// hearts capsule and no score capsule, a HUD given neither coins nor score
// gets neither, and the kit's own HUD (lives, coins and score set) draws
// exactly the groups it drew before.
//
// Ways it could fail: an empty `Lives` capsule with maxLives 0; a `Score`
// capsule reading 0 when no score was given; the hearts or the score gone
// from a game that sets them.
import { beforeAll, expect, test } from "bun:test";
import { og, defineSystem } from "@origingame/engine";
import type { PlatformerHudDesc } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type GameToken } from "../src/runtime.ts";

beforeAll(() => {
  loadNativeHost();
});

type Element = { id: string; parent: string; text: string };

let n = 0;
function hud(desc: PlatformerHudDesc): Element[] {
  const Hud = defineSystem(`UpHud${n++}`, {}, () => {
    og.ui.platformerHud("Hud", desc);
  });
  const r = createEngine(og.game({ systems: [Hud] }) as GameToken);
  if (!r.ok) throw new Error(r.reason);
  r.value.advance(2);
  return (r.value.probe.dump({ what: "ui" }) as unknown as { elements: Element[] }).elements;
}

const ids = (ui: Element[]): string[] => ui.map((e) => e.id);

test("og.ui.platformerHud: no lives and no score draw no hearts and no score (Brasswick's HUD)", () => {
  const ui = hud({ lives: 0, maxLives: 0, coins: 2, level: "The Boiler Stair", timeLeft: null });
  expect(ids(ui)).not.toContain("Lives");
  expect(ids(ui)).not.toContain("Score");
  expect(ids(ui)).not.toContain("Points");
  expect(ui.find((e) => e.id === "Count" && e.parent === "Coins")?.text).toBe("2");
  expect(ui.find((e) => e.id === "Level")?.text).toBe("The Boiler Stair");
});

test("og.ui.platformerHud: neither coins nor score given, neither drawn; the pause button stays", () => {
  const ui = hud({ lives: 2 });
  expect(ids(ui)).not.toContain("Coins");
  expect(ids(ui)).not.toContain("Score");
  expect(ids(ui)).toContain("Pause");
  // maxLives' default (3) still draws three hearts, two of them full.
  expect(ui.filter((e) => e.parent === "Lives").length).toBe(3);
});

test("og.ui.platformerHud: a game that sets lives, coins and score draws them as before", () => {
  const ui = hud({ lives: 2, maxLives: 3, coins: 14, score: 1200, level: "1-1", timeLeft: 80 });
  expect(ui.filter((e) => e.parent === "Lives").length).toBe(3);
  expect(ui.find((e) => e.id === "Count" && e.parent === "Coins")?.text).toBe("14");
  expect(ui.find((e) => e.id === "Points")?.text).toBe("1,200");
  expect(ui.filter((e) => e.parent === "").map((e) => e.id)).toEqual(["Hud.left", "Hud.top", "Hud.right"]);
  // A score of 0 is a score: shown.
  expect(hud({ score: 0 }).find((e) => e.id === "Points")?.text).toBe("0");
});
