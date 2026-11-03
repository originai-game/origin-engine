// The kit's genre showcases (docs/ui.md §17, roadmap U8): one per genre
// family, each a playable set of its screens, HUD and world UI over the
// kit's fixtures, worn by every seed style. A showcase is a list of pages;
// a page is a kit screen (src/main.ts SCREENS): a HUD built here from the
// HUD components (packages/engine/src/ui/hud), a screen document the kit
// ejected with `og ui eject` (assets/ui/*.ogt, mounted with og.ui.screen
// and answered by its events), or a genre pattern's preview (patterns.ts,
// genres*.ts, starship.ts). The pages lead to each other as a game's do
// (the title's New game to the HUD, its pause to the pause menu, a finish
// to the results), and the kit's chrome steps through them.
//
// Every word is the game's, keyed in assets/strings (sc.*); names (racers,
// players, teams, places) are invented proper nouns and stay as they are.

import { og } from "@origingame/engine";
import type { Anchor, Size, World } from "@origingame/engine";
import { L } from "./patterns.ts";
import { fightHud, horrorHud, mmoHud, mobaHud, openHud, rhythmHud, rogueHud, rpgHud, survivalHud, tdHud } from "./huds2.ts";
import { actionHud, hudArt, loadHudArt, manageHud, raceHud, resetHuds, shooterHud, sportsHud, matchFacts, hudCommand } from "./huds.ts";

const grow: Size = { kind: "grow" };

/** A showcase's page: the kit screen it shows and its name (a string key). */
export interface ShowcasePage {
  readonly screen: string;
  readonly name: string;
}

/** A genre family's showcase: its primary seed style (the coverage matrix, tests/evidence/ui-art/concepts/README.md) and its pages. */
export interface Showcase {
  readonly key: string;
  readonly pack: string;
  readonly pages: readonly ShowcasePage[];
}

const page = (screen: string, name: string): ShowcasePage => ({ screen, name: `sc.page.${name}` });

/**
 * The families of docs/ui.md §17 (the first five, then the rest in §17's
 * order). Each family's name and line are `sc.<key>` and `sc.<key>.blurb`;
 * its pack is the seed style it is shown in first. The matrix has no row for
 * MOBA, fighting or open world: MOBA takes fantasy (its heroes' look), fighting
 * the party games' cartoon, open world storybook's painted land; survival
 * takes craft, wasteland's secondary, as the kit carries no wasteland pack.
 */
export const SHOWCASES: readonly Showcase[] = [
  { key: "action", pack: "pixel", pages: [page("actionTitle", "title"), page("actionHud", "hud"), page("actionPause", "pause"), page("actionTutorial", "tutorial"), page("actionLoading", "loading"), page("actionResults", "results")] },
  { key: "shooter", pack: "tactical", pages: [page("shooterHud", "hud"), page("shooterScore", "scoreboard"), page("loadout", "loadout"), page("lobby", "lobby")] },
  { key: "puzzle", pack: "cartoon", pages: [page("board", "board"), page("saga", "saga"), page("store", "store")] },
  { key: "management", pack: "craft", pages: [page("manageHud", "hud"), page("generators", "generators"), page("ledger", "ledger"), page("cityMap", "cityMap")] },
  { key: "racing", pack: "cyber", pages: [page("raceHud", "hud"), page("garage", "garage"), page("raceBoard", "leaderboard"), page("sportsHud", "sportsHud"), page("matchFacts", "matchFacts")] },
  { key: "moba", pack: "fantasy", pages: [page("mobaHud", "hud"), page("draft", "draft")] },
  { key: "rpg", pack: "fantasy", pages: [page("rpgHud", "hud"), page("gear", "gear"), page("quests", "quests"), page("dialogue", "dialogue")] },
  { key: "mmo", pack: "fantasy", pages: [page("mmoHud", "hud"), page("chat", "chat")] },
  { key: "roguelike", pack: "pixel", pages: [page("rogueHud", "hud"), page("cards", "cards"), page("rewards", "rewards"), page("deck", "deck")] },
  { key: "strategy", pack: "scifi", pages: [page("rts", "rts"), page("tdHud", "td"), page("shipStatus", "shipStatus"), page("turnOrder", "turnOrder"), page("abilityCards", "abilityCards"), page("sectorMap", "sectorMap"), page("hangar", "hangar")] },
  { key: "gacha", pack: "kawaii", pages: [page("gacha", "gacha"), page("pass", "pass")] },
  { key: "rhythm", pack: "cyber", pages: [page("rhythmHud", "hud"), page("songs", "songs")] },
  { key: "fighting", pack: "cartoon", pages: [page("fightHud", "hud"), page("select", "select"), page("versus", "versus")] },
  { key: "narrative", pack: "kawaii", pages: [page("vn", "vn"), page("routes", "routes"), page("cgGallery", "cgGallery")] },
  { key: "survival", pack: "craft", pages: [page("survivalHud", "hud")] },
  { key: "horror", pack: "dread", pages: [page("horrorHud", "hud")] },
  { key: "openworld", pack: "storybook", pages: [page("openHud", "hud"), page("worldMap", "worldMap"), page("photo", "photo")] },
];

/** The screens this file builds (HUDs and mounted documents), in the order SCREENS lists them. */
export const SHOWCASE_SCREENS = [
  "actionTitle", "actionHud", "actionPause", "actionTutorial", "actionLoading", "actionResults",
  "shooterHud", "shooterScore", "manageHud", "raceHud", "raceBoard", "sportsHud", "matchFacts",
  "mobaHud", "rpgHud", "mmoHud", "rogueHud", "tdHud", "rhythmHud", "fightHud", "survivalHud", "horrorHud", "openHud",
] as const;

/** The showcase a screen is a page of, and its index there (the first showcase that lists it). */
export function showcaseOf(screen: string): { showcase: Showcase; index: number } | null {
  for (const showcase of SHOWCASES) {
    const index = showcase.pages.findIndex((p) => p.screen === screen);
    if (index >= 0) return { showcase, index };
  }
  return null;
}

// ── the screen documents ────────────────────────────────────────────────

/** The documents the kit ejected (assets/ui), loaded at the start. */
const DOCS = ["main-menu", "pause", "results", "loading", "tutorial", "leaderboard", "scoreboard"] as const;
type Doc = (typeof DOCS)[number];

/** The documents this kit mounted, bottom first: what it pops when a page changes. */
let mounted: Doc[] = [];

/** Load the showcases' documents and art (the game's start). */
export function loadShowcases(world: World): void {
  for (const d of DOCS) {
    const r = og.load.document(world, `ui/${d}.ogt`);
    if (!r.ok) throw new Error(r.reason);
  }
  loadHudArt(world);
}

/**
 * The documents a screen mounts, bottom first; the kit pops the ones it no
 * longer wants (a page left, the gallery) before it builds.
 */
function docsOf(screen: string): Doc[] {
  switch (screen) {
    case "actionTitle":
      return ["main-menu"];
    case "actionPause":
      return ["pause"];
    case "actionTutorial":
      return ["tutorial"];
    case "actionLoading":
      return ["loading"];
    case "actionResults":
      return ["results"];
    case "shooterScore":
      return ["scoreboard"];
    case "raceBoard":
      return ["leaderboard"];
    default:
      return [];
  }
}

/** Pop what this screen does not mount (the stack is the world's: snapshots carry it). */
export function syncDocs(world: World, screen: string): void {
  const want = docsOf(screen);
  if (mounted.length === want.length && mounted.every((d, i) => d === want[i])) return;
  for (let n = og.ui.layers(world).length; n > 0 && mounted.length > 0; n--) {
    og.ui.pop(world);
    mounted.pop();
  }
  mounted = [];
}

/** The kit's chrome (the Gallery and the next page), which a document's Kit slot takes: set by showcaseScreen. */
let chrome: () => void = () => {};

/**
 * Mount a document for this tick with its viewmodel, and answer its events.
 * The kit's chrome goes in its Kit slot (assets/ui: the kit's copies have
 * one), inside the layer's focus scope, so keys and pads reach it.
 */
function doc(name: Doc, vm: Record<string, unknown>, on: (event: string) => void): void {
  if (!mounted.includes(name)) mounted.push(name);
  const r = og.ui.screen(name, vm);
  if (name !== "scoreboard" && og.ui.openSlot(`${name}/Kit`)) {
    og.ui.row("Chrome", { gap: 8, cross: "center" }, chrome);
    og.ui.close();
  }
  for (const e of r.events) if (e.kind === "emit") on(e.name);
}

/** The pages whose chrome a document's Kit slot takes (the menu and modal layers; the scoreboard is a HUD layer). */
export function chromeInDocument(screen: string): boolean {
  return docsOf(screen).some((d) => d !== "scoreboard");
}

/** The racers' portraits on the boards (the kit's art). */
const AVATARS = ["portraits/aria", "portraits/kael", "portraits/mira"] as const;

// ── pages ───────────────────────────────────────────────────────────────

/** The showcases' state between ticks (a gallery, never saved). */
const sc0 = () => ({ played: 4520, tutorialStep: 2, loadingFrom: 0, resultsTab: "summary", boardTab: "global" });
let sc = sc0();

/** `sc:reset`, `sc:hud:<key>:<value>` (huds.ts): a showcase's preview state. */
export function showcaseCommand(args: readonly string[]): boolean {
  if (args[0] === "reset") {
    sc = sc0();
    resetHuds();
    return true;
  }
  if (args[0] === "tutorial") {
    sc.tutorialStep = Math.max(1, Math.min(5, Number(args[1] ?? 1)));
    return true;
  }
  return hudCommand(args[0] ?? "", args[1] ?? "", args[2] ?? "");
}

/**
 * Build a showcase screen. `go` changes the kit's screen (a page's buttons
 * lead to the next page, as a game's do). False when the screen is not one
 * of this file's.
 */
export function showcaseScreen(world: World, screen: string, go: (screen: string) => void, kitChrome: () => void): boolean {
  chrome = kitChrome;
  switch (screen) {
    case "actionTitle":
      // The title over the game's key art: the menu's events lead into the showcase.
      doc("main-menu", { canContinue: true, saveLine: L("sc.action.save"), version: "1.0.0", news: L("sc.action.news"), keyArt: hudArt("scenes/title") ?? 0 }, (e) => {
        if (e === "continue" || e === "newGame") go("actionLoading");
        if (e === "settings") go("settingsTree");
        if (e === "quit" || e === "credits" || e === "modes") go("gallery");
      });
      return true;
    case "actionHud":
      // A HUD is played with the game's actions, not by moving a focus (og.ui.setNavigation in actionHud); Back pauses.
      if (actionHud(world) || og.ui.back()) go("actionPause");
      return true;
    case "actionPause":
      // Over the HUD it pauses: the HUD is built under it, still, without its controls.
      actionHud(world, true);
      doc("pause", { objective: L("sc.action.objective"), played: sc.played, canSave: true }, (e) => {
        if (e === "resume") go("actionHud");
        if (e === "settings" || e === "controls") go("settingsTree");
        if (e === "quitToMenu") go("actionTitle");
        if (e === "quit") go("gallery");
      });
      return true;
    case "actionTutorial":
      actionHud(world, true);
      // The step's callout points at the HUD part it teaches: the jump button on a touch screen, else the combo.
      {
        const target = og.ui.bounds(sc.tutorialStep % 2 === 0 ? "Player" : "Run");
        doc("tutorial", { title: L(`sc.tutorial.${sc.tutorialStep}.title`), text: L(`sc.tutorial.${sc.tutorialStep}.text`), step: sc.tutorialStep, steps: 5, waits: false, x: target ? Math.round(target.x + target.width / 2) : 64, y: target ? Math.round(target.y + target.height + 12) : 120, texture: 0 }, (e) => {
          if (e === "next") {
            if (sc.tutorialStep >= 5) go("actionHud");
            else sc.tutorialStep += 1;
          }
          if (e === "skip" || e === "done") go("actionHud");
        });
      }
      return true;
    case "actionLoading": {
      // Real progress over the level's key art, the tip of the place; a press continues once it is ready.
      const t = og.ui.tick();
      if (sc.loadingFrom === 0) sc.loadingFrom = t;
      const progress = Math.min(1, (t - sc.loadingFrom) / 240);
      doc("loading", { place: L("sc.action.place"), tip: L("sc.action.tip"), progress, ready: progress >= 1, waits: true, keyArt: hudArt("scenes/genre-action") ?? 0 }, (e) => {
        if (e === "continue") go("actionHud");
      });
      return true;
    }
    case "actionResults":
      doc("results", {
        won: true, grade: "A", title: L("sc.action.clear"), tab: sc.resultsTab, level: 12, xp: 0.64, gained: 1240,
        stats: [{ name: L("sc.stat.score"), value: 48210, best: true }, { name: L("sc.stat.coins"), value: 86, best: false }, { name: L("sc.stat.time"), value: 102, best: false }],
        rewards: [{ name: L("sc.reward.gold"), amount: 320, texture: hudArt("boons/gold") ?? 0 }, { name: L("sc.reward.crystal"), amount: 4, texture: hudArt("boons/crystal") ?? 0 }],
      }, (e) => {
        if (e === "continue") go("actionTitle");
        if (e === "again" || e === "replay") go("actionHud");
      });
      return true;
    case "shooterHud":
      shooterHud(world);
      if (og.ui.back()) go("gallery");
      return true;
    case "shooterScore":
      // The scoreboard held over the match (Tab): the HUD stays under it.
      shooterHud(world);
      doc("scoreboard", { score: "7 - 5", clock: 412, map: "Harbour" }, () => {});
      if (og.ui.back()) go("shooterHud");
      return true;
    case "manageHud":
      manageHud(world);
      if (og.ui.back()) go("gallery");
      return true;
    case "raceHud":
      raceHud(world);
      if (og.ui.back()) go("gallery");
      return true;
    case "raceBoard":
      doc("leaderboard", {
        tab: sc.boardTab, period: L("sc.race.week"), selected: -1,
        rows: [["Nyx", 61_234], ["Ferro", 61_891], ["Lumen", 62_040], ["Quill", 62_455], ["Sable", 62_910], ["Orrin", 63_118]].map(([name, ms], i) => ({ rank: i + 1, name, score: ms, you: false, texture: hudArt(AVATARS[i % AVATARS.length]!) ?? 0 })),
        me: { rank: 1284, name: L("sc.you"), score: 66_002 },
        league: { name: L("sc.race.league"), promote: 10, demote: 25, ends: 259200 },
      }, (e) => {
        if (e === "close") go("gallery");
      });
      return true;
    case "sportsHud":
      sportsHud(world);
      if (og.ui.back()) go("gallery");
      return true;
    case "matchFacts":
      matchFacts(world);
      if (og.ui.back()) go("gallery");
      return true;
    case "mobaHud": case "rpgHud": case "mmoHud": case "rogueHud": case "tdHud": case "rhythmHud": case "fightHud": case "survivalHud": case "horrorHud": case "openHud":
      HUDS2[screen](world);
      if (og.ui.back()) go("gallery");
      return true;
    default:
      return false;
  }
}

/** The other families' play screens (huds2.ts). */
const HUDS2 = { mobaHud, rpgHud, mmoHud, rogueHud, tdHud, rhythmHud, fightHud, survivalHud, horrorHud, openHud } as const;

/**
 * Where a HUD page leaves the kit's chrome, desktop then phone: a spot its
 * HUD keeps free (the middle of the start edge on most; the racing HUD's
 * standings take that on a desktop). Document pages take it in their Kit slot.
 */
const CHROME: Record<string, [Anchor, Anchor]> = {
  actionHud: ["left", "left"], shooterHud: ["left", "left"], shooterScore: ["left", "left"], manageHud: ["left", "left"],
  raceHud: ["bottom", "left"], sportsHud: ["left", "left"], matchFacts: ["bottomLeft", "bottom"],
  mobaHud: ["left", "left"], rpgHud: ["left", "left"], mmoHud: ["bottomLeft", "left"], rogueHud: ["left", "left"], tdHud: ["left", "left"],
  rhythmHud: ["left", "bottomLeft"], fightHud: ["bottom", "left"], survivalHud: ["left", "left"], horrorHud: ["left", "left"], openHud: ["left", "left"],
};

/** Where a showcase HUD page wants the kit's chrome, or null for the kit's own rule. */
export function chromeAnchor(screen: string): Anchor | null {
  const c = CHROME[screen];
  if (!c) return null;
  const bp = og.ui.frameViewport().breakpoint;
  return bp === "phonePortrait" || bp === "phoneLandscape" ? c[1] : c[0];
}

/**
 * The showcase's own chrome beside the Gallery button: the next page's
 * button (its name), so a reader steps through the family as a player
 * goes from screen to screen. Built inside the Gallery button's anchor.
 */
export function showcaseNext(go: (screen: string) => void, screen: string): void {
  const at = showcaseOf(screen);
  if (!at) return;
  const pages = at.showcase.pages;
  const next = pages[(at.index + 1) % pages.length]!;
  if (og.ui.pushButton("Next", L(next.name), { variant: "secondary", iconEnd: "play", width: { kind: "fit" } }).clicked) go(next.screen);
}

/** The gallery's showcase rows: a family's name and line. */
export function showcaseRow(showcase: Showcase): { name: string; blurb: string } {
  return { name: L(`sc.${showcase.key}`), blurb: L(`sc.${showcase.key}.blurb`) };
}

export { grow };
