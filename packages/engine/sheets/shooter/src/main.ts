// The shooter components' review sheet (docs/ui.md §6.2): the normal page (combat
// HUD over a bright sky and a dark ground) and the stress page (a loadout
// with long names, locked and equipped slots, and a dense killfeed), in a
// style pack. The pages use the built-in components by name, as a game
// does, with fixture stores; loading a page loads them.
//
// Commands: style:<clean>, page:<combat|loadout>; for motion clips (`og
// render --keyframes`), hit:<kind> replays the hit mark, damage:<id>:<angle>
// :<strength>:<kind> adds an incoming hit, heal:<id> removes one, and
// reduced:<on|off> sets reduced motion.

import { og, type World } from "@origingame/engine";
import { COMBAT, FEED, LOADOUT } from "./fixtures.ts";

const PAGES = ["combat", "loadout"] as const;
const HITS = ["none", "hit", "headshot", "kill", "shieldBreak"] as const;
const KINDS = ["normal", "shield", "critical"] as const;
const STYLES = ["clean"] as const;

const DOCUMENTS = [
  "ui/combat.svelte",
  "ui/loadout.svelte",
  "ui/sheet.svelte",
];

function setStyle(world: World, name: string): void {
  const loaded = og.load.style(world, `@styles/${name}/style.ogt`);
  if (!loaded.ok) throw new Error(loaded.reason);
  og.ui.configure(world, { style: loaded.value });
}

/** Mounts the sheet, its stores the fixtures and the page shown. */
function show(world: World): void {
  const scope = og.ui.scope(world, "sheet");
  const stores = Object.fromEntries(Object.entries({ ...COMBAT, feed: FEED, ...LOADOUT, page: "combat" }).map(([k, v]) =>
    [k, Array.isArray(v) ? scope.collection(k, v as { id: string }[], { key: "id" }) : scope.writable(k, v)]));
  const mounted = scope.mount("sheet", "ShooterSheet", stores);
  if (!mounted.ok) throw new Error(mounted.reason);
  const errors = scope.takeErrors();
  if (errors.length) throw new Error(errors.join("; "));
}

export default og.game({
  tick: 60,
  seed: 1,
  start(world) {
    const camera = world.resource(og.Camera);
    if (camera.ok) camera.value.background = { r: 0.05, g: 0.06, b: 0.08 };
    setStyle(world, "clean");
    // The kit's strings in the review's CJK and right-to-left locales, as a
    // game ships them (English comes with the kit).
    for (const locale of ["ja", "zh-CN", "ar"]) {
      const loaded = og.i18n.load(world, `strings/${locale}.ogt`);
      if (!loaded.ok) throw new Error(loaded.reason);
    }
    for (const path of DOCUMENTS) {
      const loaded = og.load.document(world, path);
      if (!loaded.ok) throw new Error(`${path}: ${loaded.reason}`);
    }
  },
  ui: show,
  commands: {
    style: (world, args) => setStyle(world, STYLES.includes(args[0] as never) ? args[0]! : "clean"),
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
    style: { usage: `<${STYLES.join("|")}>`, doc: "The style pack the pages are dressed in." },
    page: { usage: `<${PAGES.join("|")}>`, doc: "A review page of the shooter kit." },
    hit: { usage: `<${HITS.join("|")}>`, doc: "Set the crosshair's hit mark and replay it." },
    damage: { usage: `<id>:<angle rad>:<strength 0-1>:<${KINDS.join("|")}>`, doc: "Add an incoming hit to the damage direction." },
    heal: { usage: "<id>", doc: "Remove an incoming hit." },
    reduced: { usage: "<on|off>", doc: "Reduced motion on or off." },
  },
});
