import { defineResource, defineSystem, Entity, og } from "@origingame/engine";
import type { UiCollection, World } from "@origingame/engine";

// An interactive film (og api show film): the story in assets/story/film.yarn,
// a segment per film node in assets/segments/, its footage in assets/clips/.
// The picture fills the screen (assets/ui/film-view.svelte), the built-in
// subtitles screen shows the segment's captions and film-choices its window.

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }, what: string): T {
  if (!r.ok) throw new Error(`${what}: ${r.reason}`);
  return r.value;
}

/** The film og.film.start made. */
const FilmRef = defineResource("FilmRef", { film: Entity() });

type ChoiceRow = { id: string; text: string; enabled: boolean; chosen: boolean; progress: number };
type HotspotRow = { id: string; x: number; y: number; width: number; height: number; label: string };
type LineRow = { id: string; speaker: string; text: string };

/** The film's presentation stores, written each tick from og.film.view (film §4). */
function stores(world: World) {
  const scope = og.ui.scope(world, "film");
  return {
    scope,
    picture: scope.writable("picture", 0),
    open: scope.writable("open", false),
    left: scope.writable("left", 0),
    locked: scope.writable("locked", false),
    choices: scope.collection("choices", [] as ChoiceRow[], { key: "id" }),
    hotspots: scope.collection("hotspots", [] as HotspotRow[], { key: "id" }),
    lines: scope.collection("lines", [] as LineRow[], { key: "id" }),
  };
}

/** Make a keyed collection hold exactly these rows, in order. */
function sync<T extends { id: string }>(c: UiCollection<T>, rows: readonly T[]): void {
  const want = new Set(rows.map((r) => r.id));
  for (const key of c.keys()) if (!want.has(key)) c.remove(key);
  const have = new Set(c.keys());
  for (const row of rows) {
    if (have.has(row.id)) c.patch(row.id, row);
    else c.insert(row);
  }
}

const presentFilm = defineSystem("PresentFilm", { reads: [FilmRef] }, (_q, _time, world) => {
  const ref = world.resource(FilmRef);
  if (!ref.ok) return;
  const film = ref.value.film;
  // A choice row or a hotspot picked: its key is the choice's index.
  for (const e of og.ui.docEvents(world)) {
    if (e.mount === "film-choices" && e.kind === "emit" && (e.name === "choose" || e.name === "pick")) og.film.choose(world, film, Number(e.rowKey));
  }
  const view = og.film.view(world, film);
  if (!view) return;
  const s = stores(world);
  s.picture.set(view.texture);
  const win = view.window;
  s.open.set(win !== null);
  s.left.set(win && win.ticksTotal > 0 ? win.ticksLeft / win.ticksTotal : 0);
  s.locked.set(win?.locked ?? false);
  sync(
    s.choices,
    win
      ? view.choices.map((c) => ({
          id: String(c.index),
          text: c.text,
          enabled: c.enabled,
          chosen: c.index === win.pending,
          progress: view.qtes.find((q) => q.choice === c.index)?.progress ?? -1,
        }))
      : [],
  );
  sync(
    s.hotspots,
    view.hotspots.map((h) => ({ id: String(h.choice), x: h.x, y: h.y, width: h.width, height: h.height, label: view.choices.find((c) => c.index === h.choice)?.text ?? "" })),
  );
});

export default og.game({
  resources: [FilmRef],
  start(world) {
    must(og.story.load(world, "story/film.yarn"), "story");
    must(og.film.load(world, "film"), "film");
    // Paused: on the web, audio starts only after a gesture, so a title
    // screen's Start calls og.film.play (film §4); `film:play` in a script.
    must(world.resource(FilmRef), "FilmRef").film = must(og.film.start(world, { dialogue: "film", paused: true }), "start");
    for (const doc of ["ui/film-view.svelte", "@ui/screens/subtitles.svelte", "@ui/screens/film-choices.svelte"]) must(og.load.document(world, doc), doc);
  },
  ui(world) {
    const s = stores(world);
    must(s.scope.mount("film-view", "film-view", { picture: s.picture }), "film-view");
    must(s.scope.mount("subtitles", "subtitles", { lines: s.lines }), "subtitles");
    must(s.scope.mount("film-choices", "film-choices", { open: s.open, left: s.left, locked: s.locked, choices: s.choices, hotspots: s.hotspots }), "film-choices");
  },
  systems: [presentFilm],
  // What the probe's state() and a playtest's `state:` checks read (film §8).
  facts(world) {
    const ref = world.resource(FilmRef);
    const view = ref.ok ? og.film.view(world, ref.value.film) : null;
    return { film: view ? { node: view.node, ending: view.ending, ended: view.ended, open: view.window !== null, pending: view.window?.pending ?? -1 } : null };
  },
});
