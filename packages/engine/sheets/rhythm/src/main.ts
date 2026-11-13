// The rhythm components' review sheet (docs/ui.md §6.2): the songs page (the
// song list with the selected song's details and its charts), the play page
// (the judgement and score HUD over bright and dark lanes beside a long
// results screen) and the states page (every member in every fixture state
// it declares), dressed by a style pack or the default style. The pages use
// the built-in components by name, as a game does, with fixture stores.
//
// Commands: style:<fantasy|default>, page:<songs|play|states>; for motion
// clips (`og render --keyframes`), hit:<perfect|great|good|bad|miss>:<early|late|-> judges
// the next hit and replays the pop, and reduced:<on|off> sets reduced motion.

import { og, type World } from "@origingame/engine";
import { FIXTURES } from "./fixtures.ts";

const PAGES = ["songs", "play", "states"] as const;
const STYLES = ["fantasy", "default"] as const;
const JUDGEMENTS = ["perfect", "great", "good", "bad", "miss"] as const;

const DOCUMENTS = ["ui/songs.svelte", "ui/play.svelte", "ui/states.svelte", "ui/sheet.svelte"];

const NAMES: Record<string, string> = {
  songSelect: "RhythmSongSelect", difficulty: "RhythmDifficulty", judgement: "RhythmJudgement",
  scoreHud: "RhythmScoreHud", results: "RhythmResults",
};

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }, what: string): T {
  if (!r.ok) throw new Error(`${what}: ${r.reason}`);
  return r.value;
}

/** The shared ink icon sheet's cells (templates/styles/shared/ink-icons.png). */
const INK_NAMES = [
  "heart", "star", "coin", "lock", "pause", "gear", "sword", "shield",
  "potion", "gem", "key", "map", "bolt", "clock", "trophy", "home",
  "plus", "close", "check", "play", "back", "bag", "bomb", "crown",
  "flag", "apple", "leaf", "fire", "drop", "skull", "music", "person",
];

/** The style and the fallback icons in one configure, so neither resets the other. */
function setStyle(world: World, name: string): void {
  const ink = must(og.load.texture(world, "icons/ink-icons.png"), "ink icons");
  const style = name === "default" ? null : must(og.load.style(world, `@styles/${name}`), name);
  og.ui.configure(world, { style, fallbackIcons: { texture: ink, columns: 8, rows: 4, names: INK_NAMES } });
}

/** Each fixture as a store of its scalars (`<member>_<state>`) and a keyed collection per list (`<member>_<state>_<prop>`). */
function stores(world: World): Record<string, unknown> {
  const scope = og.ui.scope(world, "sheet");
  const out: Record<string, unknown> = { page: scope.writable("page", "songs") };
  const captions: Record<string, string> = {};
  for (const [member, states] of Object.entries(FIXTURES)) {
    for (const [state, fixture] of Object.entries(states)) {
      const name = `${member}_${state}`;
      captions[name] = `${NAMES[member]} · ${state}`;
      out[name] = scope.writable(name, fixture);
      for (const [prop, value] of Object.entries(fixture)) {
        if (Array.isArray(value)) out[`${name}_${prop}`] = scope.collection(`${name}_${prop}`, value as { id: string }[], { key: "id" });
      }
    }
  }
  out.captions = scope.writable("captions", captions);
  return out;
}

function show(world: World): void {
  const scope = og.ui.scope(world, "sheet");
  must(scope.mount("sheet", "RhythmSheet", stores(world)), "mount");
  const errors = scope.takeErrors();
  if (errors.length) throw new Error(errors.join("; "));
}

export default og.game({
  tick: 60,
  seed: 1,
  start(world) {
    const camera = world.resource(og.Camera);
    if (camera.ok) camera.value.background = { r: 0.05, g: 0.05, b: 0.06 };
    setStyle(world, "fantasy");
    for (const path of DOCUMENTS) must(og.load.document(world, path), path);
  },
  ui: show,
  commands: {
    style: (world, args) => setStyle(world, STYLES.includes(args[0] as never) ? args[0]! : "fantasy"),
    page: (world, args) => {
      og.ui.scope(world, "sheet").writable("page", "songs").set(PAGES.includes(args[0] as never) ? args[0]! : "songs");
    },
    hit: (world, args) => {
      const judgement = JUDGEMENTS.includes(args[0] as never) ? args[0]! : "perfect";
      const timing = args[1] === "early" || args[1] === "late" ? args[1] : "";
      const store = og.ui.scope(world, "sheet").writable("judgement_default", FIXTURES.judgement!.default!);
      store.update((j) => ({ ...j, judgement, timing, broken: judgement === "miss", combo: judgement === "miss" ? 0 : (j.combo as number) + 1, hit: (j.hit as number) + 1 }));
    },
    reduced: (world, args) => og.ui.configure(world, { reducedMotion: args[0] !== "off" }),
  },
  commandHelp: {
    style: { usage: `<${STYLES.join("|")}>`, doc: "The style the pages are dressed in." },
    page: { usage: `<${PAGES.join("|")}>`, doc: "A review page of the rhythm kit." },
    hit: { usage: `<${JUDGEMENTS.join("|")}>:<early|late|->`, doc: "Judge the next hit and replay the judgement's pop (the sheet stands in for the game)." },
    reduced: { usage: "<on|off>", doc: "Reduced motion on or off." },
  },
});
