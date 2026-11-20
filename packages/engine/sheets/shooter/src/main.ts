// The shooter components' review sheet (docs/ui.md §6.2): the normal page (the
// combat HUD over sample stage art, as the approved mockup lays it out) and
// the stress page (a loadout with long names, locked and equipped slots, and
// a dense killfeed), dressed by a style pack or the default style. The pages use the built-in
// components by name, as a game does, with fixture stores; loading a page
// loads them.
//
// Commands: style:<fantasy|default>, page:<combat|loadout>; for motion clips (`og
// render --keyframes`), hit:<kind> replays the hit mark, damage:<id>:<angle>
// :<strength>:<kind> adds an incoming hit, heal:<id> removes one, and
// reduced:<on|off> sets reduced motion.

import { og, type World } from "@origingame/engine";
import { ART, COMBAT, FEED, LOADOUT } from "./fixtures.ts";

const PAGES = ["combat", "loadout"] as const;
const HITS = ["none", "hit", "headshot", "kill", "shieldBreak"] as const;
const KINDS = ["normal", "shield", "critical"] as const;
const STYLES = ["fantasy", "scifi", "cartoon", "default"] as const;

/** The sample art's textures by name (sheet only, never installed). */
const art: Record<string, number> = {};

/** A fixture with each field that names sample art (portrait, icon, map) given its texture. */
function withArt<T>(value: T): T {
  if (Array.isArray(value)) return value.map(withArt) as T;
  if (value && typeof value === "object") {
    return Object.fromEntries(Object.entries(value).map(([k, v]) =>
      [k, ["portrait", "icon", "map"].includes(k) && typeof v === "string" ? (art[v] ?? 0) : withArt(v)])) as T;
  }
  return value;
}

const DOCUMENTS = [
  "ui/combat.svelte",
  "ui/loadout.svelte",
  "ui/sheet.svelte",
];

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }, what: string): T {
  if (!r.ok) throw new Error(`${what}: ${r.reason}`);
  return r.value;
}

/** The shared ink icon sheet's cells (templates/styles/shared/ink-icons.png): the loadout's, the killfeed's and the vitals' glyphs. */
const INK_NAMES = [
  "heart", "star", "coin", "lock", "pause", "gear", "sword", "shield",
  "potion", "gem", "key", "map", "bolt", "clock", "trophy", "home",
  "plus", "close", "check", "play", "back", "bag", "bomb", "crown",
  "flag", "apple", "leaf", "fire", "drop", "skull", "music", "person",
];

/**
 * The style and the fallback icons in one configure, so neither resets the
 * other. A pack brings its own fonts; the default style draws in the
 * engine's default face (Instrument Sans), as a game with no font does.
 */
function setStyle(world: World, name: string): void {
  const ink = must(og.load.texture(world, "icons/ink-icons.png"), "ink icons");
  const fallbackIcons = { texture: ink, columns: 8, rows: 4, names: INK_NAMES };
  if (name === "default") {
    og.ui.configure(world, { style: null, font: null, fallbackIcons });
  } else og.ui.configure(world, { style: must(og.load.style(world, `@styles/${name}`), name), font: null, fallbackIcons });
}

/** Mounts the sheet, its stores the fixtures and the page shown. */
function show(world: World): void {
  const scope = og.ui.scope(world, "sheet");
  const stores = Object.fromEntries(Object.entries({ ...withArt(COMBAT), stage: art.stage ?? 0, feed: FEED, ...LOADOUT, page: "combat" }).map(([k, v]) =>
    [k, Array.isArray(v) ? scope.collection(k, v as { id: string }[], { key: "id" }) : scope.writable(k, v)]));
  must(scope.mount("sheet", "ShooterSheet", stores), "mount");
  const errors = scope.takeErrors();
  if (errors.length) throw new Error(errors.join("; "));
}

export default og.game({
  tick: 60,
  seed: 1,
  start(world) {
    const camera = world.resource(og.Camera);
    if (camera.ok) camera.value.background = { r: 0.05, g: 0.06, b: 0.08 };
    for (const name of ART) art[name] = must(og.load.texture(world, `art/${name}.jpg`), name) as number;
    setStyle(world, "fantasy");
    // The kit's strings in the review's CJK and right-to-left locales, as a
    // game ships them (English comes with the kit).
    for (const locale of ["ja", "zh-CN", "ar"]) must(og.i18n.load(world, `strings/${locale}.ogt`), locale);
    for (const path of DOCUMENTS) must(og.load.document(world, path), path);
  },
  ui: show,
  commands: {
    style: (world, args) => setStyle(world, STYLES.includes(args[0] as never) ? args[0]! : "fantasy"),
    scheme: (world, args) => og.ui.configure(world, { scheme: args[0] === "light" ? "light" : "dark" }),
    page: (world, args) => {
      og.ui.scope(world, "sheet").writable("page", "combat").set(PAGES.includes(args[0] as never) ? args[0]! : "combat");
    },
    hit: (world, args) => {
      const scope = og.ui.scope(world, "sheet");
      scope.writable("hit", COMBAT.hit).set(HITS.includes(args[0] as never) ? args[0]! : "hit");
      scope.writable("hitAt", COMBAT.hitAt).update((at) => at + 1);
    },
    damage: (world, args) => {
      const [id = "d", angle = "0", strength = "1", kind = "normal"] = args;
      og.ui.scope(world, "sheet").collection("hits", COMBAT.hits, { key: "id" })
        .insert({ id, angle: Number(angle), strength: Number(strength), kind: KINDS.includes(kind as never) ? kind : "normal" });
    },
    heal: (world, args) => {
      og.ui.scope(world, "sheet").collection("hits", COMBAT.hits, { key: "id" }).remove(args[0] ?? "");
    },
    reduced: (world, args) => og.ui.configure(world, { reducedMotion: args[0] !== "off" }),
  },
  commandHelp: {
    style: { usage: `<${STYLES.join("|")}>`, doc: "The style the pages are dressed in." },
    scheme: { usage: "<light|dark>", doc: "The colour scheme the style is drawn in." },
    page: { usage: `<${PAGES.join("|")}>`, doc: "A review page of the shooter kit." },
    hit: { usage: `<${HITS.join("|")}>`, doc: "Set the crosshair's hit mark and replay it." },
    damage: { usage: `<id>:<angle rad>:<strength 0-1>:<${KINDS.join("|")}>`, doc: "Add an incoming hit to the damage direction." },
    heal: { usage: "<id>", doc: "Remove an incoming hit." },
    reduced: { usage: "<on|off>", doc: "Reduced motion on or off." },
  },
});
