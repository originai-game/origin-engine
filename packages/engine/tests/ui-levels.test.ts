// The level select and the settings for a pointer, a finger and a screen
// reader (Brasswick's findings #10, #11 and #12), on the native host: the
// level select closes from its Back button and from a tap on its scrim,
// answering `closed` as Escape does; in Spanish a level card is named by
// its level and its stars in the desc's words ("0 de 3 estrellas", a
// locked card "3, bloqueado"), never by the icons' English names; the
// settings' tab row, its select on a phone, the quality presets and the
// language list are named by the desc's words.
//
// Ways it could fail: no button under the level select, a tap on the scrim
// that does nothing; a card named "... star star star" or "lock"; a group
// named `Section`, `Languages` or `Quality` in Spanish.
import { beforeAll, expect, test } from "bun:test";
import { og, defineResource, defineSystem, u32 } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";

beforeAll(() => {
  loadNativeHost();
});

type Element = { id: string; parent: string; kind: string; text: string; role: string; name: string; x: number; y: number; width: number; height: number };

function start(game: unknown): Engine {
  const r = createEngine(game as GameToken);
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

function elements(engine: Engine): Element[] {
  return (engine.probe.dump({ what: "ui" }) as unknown as { elements: Element[] }).elements;
}

const Spanish = {
  title: "Niveles",
  backLabel: "Volver",
  starsLabel: "{stars} de {max} estrellas",
  lockedLabel: "{name}, bloqueado",
  levels: [
    { name: "1. La escalera de la caldera", stars: 0 },
    { name: "2. El pozo", stars: 2 },
    { name: "3", locked: true },
  ],
};

const Screen = defineResource("UlScreen", { open: u32(), closed: u32() });
const Levels = defineSystem("UlLevels", { writes: [Screen] }, (_q, _t, world: World) => {
  const s = world.resource(Screen);
  if (!s.ok || s.value.open === 0) return;
  if (og.ui.levelSelect("Levels", Spanish).closed) {
    s.value.open = 0;
    s.value.closed += 1;
  }
});

function opened(): Engine {
  const engine = start(og.game({ resources: [Screen], systems: [Levels] }));
  const s = engine.world.resource(Screen);
  if (!s.ok) throw new Error("no screen resource");
  s.value.open = 1;
  engine.advance(30);
  return engine;
}

function closed(engine: Engine): number {
  const s = engine.world.resource(Screen);
  if (!s.ok) throw new Error("no screen resource");
  return s.value.closed;
}

test("og.ui.levelSelect: its Back button closes it, answering closed", () => {
  const engine = opened();
  const back = elements(engine).find((e) => e.id === "Back");
  expect(back?.text === "Volver" || elements(engine).some((e) => e.parent === "Back" && e.text === "Volver")).toBeTrue();
  expect(engine.probe.commands("0:touch_move_to:Footer/Back;0.05:touch_start;0.1:touch_end") as unknown as { warnings: string[] }).toMatchObject({ warnings: [] });
  engine.advance(20);
  expect(closed(engine)).toBe(1);
});

test("og.ui.levelSelect: a tap on the scrim around it closes it, answering closed", () => {
  const engine = opened();
  const panel = elements(engine).find((e) => e.id === "Panel" && e.parent === "Levels")!;
  expect(panel.x).toBeGreaterThan(40);
  // Left of the panel, halfway down: the scrim.
  engine.probe.commands(`0:touch_move:${Math.floor(panel.x / 2)},360;0.05:touch_start;0.1:touch_end`);
  engine.advance(20);
  expect(closed(engine)).toBe(1);
});

test("og.ui.levelSelect: a card is named by its level and its stars in the desc's words, a locked one by its level", () => {
  const engine = opened();
  const ui = elements(engine);
  const cards = ui.filter((e) => e.kind === "cell").map((e) => e.name);
  expect(cards).toEqual(["1. La escalera de la caldera 0 de 3 estrellas", "2. El pozo 2 de 3 estrellas", "3, bloqueado"]);
  // The total's star says the whole count.
  expect(ui.find((e) => e.id === "Star" && e.parent === "Total")?.name).toBe("2 de 9 estrellas");
  for (const e of ui) expect(`${e.name} ${e.text}`).not.toMatch(/\b(star|lock)\b/);
});

const Prefs = defineResource("UlPrefs", { tab: u32() });
const Settings = defineSystem("UlSettings", { writes: [Prefs] }, (_q, _t, _world: World) => {
  og.ui.settingsScreen("Settings", {
    open: true,
    title: "Ajustes",
    audioLabel: "Sonido",
    graphicsLabel: "Pantalla",
    controlsLabel: "Controles",
    languageLabel: "Idioma",
    qualityLabel: "Calidad",
    sectionLabel: "Apartado",
    languagesLabel: "Idiomas",
    languages: [{ code: "en", label: "English" }, { code: "es", label: "Español" }],
    settings: { language: "es" },
  });
});

test("og.ui.settingsScreen: its tab row, quality presets and language list are named in the game's words", () => {
  const engine = start(og.game({ resources: [Prefs], systems: [Settings] }));
  engine.advance(30);
  const name = (id: string) => elements(engine).find((e) => e.id === id)?.name;
  expect(name("Section")).toBe("Apartado");
  engine.probe.commands("0:touch_move_to:Section/Tab#1;0.05:touch_start;0.1:touch_end");
  engine.advance(20);
  expect(name("Quality")).toBe("Calidad");
  engine.probe.commands("0:touch_move_to:Section/Tab#3;0.05:touch_start;0.1:touch_end");
  engine.advance(20);
  expect(name("Languages")).toBe("Idiomas");
  // On a phone the tabs are a select, named the same.
  og.ui.previewViewport(engine.world, { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" } as never);
  engine.advance(5);
  const select = elements(engine).find((e) => e.id === "Section")!;
  expect([select.role, select.name]).toEqual(["combobox", "Apartado"]);
});
