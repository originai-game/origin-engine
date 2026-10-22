// The kit's other HUDs in another language, on the native host: every word
// the platformer, shooter, puzzle and idle HUDs draw or name is their desc's
// (a German table's), with their touch controls shown, so none of the kit's
// English is left; with the fields left out they draw the English they
// always drew.
//
// Ways it could fail: "Pause", "Jump", "Fire", "kills", "moves left",
// "Target 1,000", "Lv 3" or "+12/s" drawn whatever the desc says; a
// `{target}`, `{level}` or `{rate}` left unfilled.
import { beforeAll, expect, test } from "bun:test";
import { og, defineSystem } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";

beforeAll(() => {
  loadNativeHost();
});

type Element = { id: string; parent: string; text: string };

const ENGLISH = ["Pause", "Jump", "Fire", "kills", "moves left", "Tap"];

/** The HUD built by `build` on a tablet (touch controls shown): every element's id, parent and text. */
function run(name: string, build: () => void): Element[] {
  const sys = defineSystem(name, {}, (_q, _t, _world: World) => build());
  const r = createEngine(og.game({ systems: [sys] }) as unknown as GameToken);
  if (!r.ok) throw new Error(r.reason);
  const engine: Engine = r.value;
  const viewport = { width: 1024, height: 768, dpr: 1, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: true, breakpoint: "tablet" };
  og.ui.previewViewport(engine.world, viewport as never);
  engine.advance(2);
  return (engine.probe.dump({ what: "ui" }) as unknown as { elements: Element[] }).elements;
}

const texts = (ui: Element[]): string[] => ui.map((e) => e.text).filter((t) => t !== "");
/** A button's name: its own text, or its label child's. */
const named = (ui: Element[], id: string): string | undefined => {
  const e = ui.find((x) => x.id === id);
  return e && (e.text || ui.find((c) => c.parent === id && c.text !== "")?.text);
};

test("og.ui.platformerHud: Pause and Jump are the desc's", () => {
  const de = run("UhPlatformer", () => og.ui.platformerHud("Hud", { lives: 2, touch: true, pauseLabel: "Pausieren", jumpLabel: "Springen" }));
  for (const word of ENGLISH) expect(texts(de)).not.toContain(word);
  expect(named(de, "Pause")).toBe("Pausieren");
  expect(named(de, "jump")).toBe("Springen");
  const en = run("UhPlatformerEn", () => og.ui.platformerHud("Hud", { lives: 2, touch: true }));
  expect(named(en, "Pause")).toBe("Pause");
  expect(named(en, "jump")).toBe("Jump");
});

test("og.ui.shooterHud: kills, Fire and Pause are the desc's", () => {
  const de = run("UhShooter", () => og.ui.shooterHud("Hud", { health: 80, kills: 7, touch: true, killsLabel: "Abschüsse", fireLabel: "Feuer", pauseLabel: "Pausieren" }));
  for (const word of ENGLISH) expect(texts(de)).not.toContain(word);
  expect(de.find((e) => e.id === "Unit" && e.parent === "Kills")?.text).toBe("Abschüsse");
  expect(named(de, "fire")).toBe("Feuer");
  expect(named(de, "Pause")).toBe("Pausieren");
  const en = run("UhShooterEn", () => og.ui.shooterHud("Hud", { health: 80, kills: 7, touch: true }));
  expect(en.find((e) => e.id === "Unit" && e.parent === "Kills")?.text).toBe("kills");
  expect(named(en, "fire")).toBe("Fire");
});

test("og.ui.puzzleHud: moves left and Target {target} are the desc's", () => {
  const de = run("UhPuzzle", () => og.ui.puzzleHud("Hud", { score: 680, target: 1000, moves: 12, movesLabel: "Züge übrig", targetLabel: "Ziel {target}", pauseLabel: "Pausieren" }));
  for (const word of ENGLISH) expect(texts(de)).not.toContain(word);
  expect(texts(de)).toContain("Züge übrig");
  expect(texts(de)).toContain("Ziel 1,000");
  expect(texts(de).some((t) => t.startsWith("Target"))).toBeFalse();
  expect(named(de, "Pause")).toBe("Pausieren");
  const en = run("UhPuzzleEn", () => og.ui.puzzleHud("Hud", { score: 680, target: 1000, moves: 12 }));
  expect(texts(en)).toContain("moves left");
  expect(texts(en)).toContain("Target 1,000");
});

test("og.ui.idleHud: Lv {level}, +{rate}/s, Tap and Pause are the desc's", () => {
  const desc = {
    currencies: [{ id: "gold", name: "Gold", amount: 120, perSecond: 12 }],
    generators: [{ id: "mill", name: "Mühle", level: 3, cost: 50, affordable: true }],
  };
  const de = run("UhIdle", () => og.ui.idleHud("Hud", { ...desc, tapLabel: "Tippen", levelLabel: "Stufe {level}", rateLabel: "+{rate}/Sek.", pauseLabel: "Pausieren" }));
  for (const word of ENGLISH) expect(texts(de)).not.toContain(word);
  expect(texts(de)).toContain("Stufe 3");
  expect(texts(de)).toContain("+12/Sek.");
  expect(texts(de).some((t) => t.startsWith("Lv ") || t.endsWith("/s"))).toBeFalse();
  expect(named(de, "Tap")).toBe("Tippen");
  expect(named(de, "Pause")).toBe("Pausieren");
  const en = run("UhIdleEn", () => og.ui.idleHud("Hud", desc));
  expect(texts(en)).toContain("Lv 3");
  expect(texts(en)).toContain("+12/s");
});
