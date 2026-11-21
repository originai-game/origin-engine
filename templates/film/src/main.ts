import { defineResource, defineSystem, Entity, og } from "@origingame/engine";
import type { FilmView as View, TimelineCaption, World } from "@origingame/engine";
import { mount, useStore } from "@origingame/engine/ui";
import { FilmChoices, Subtitles } from "@origingame/engine/ui/components";
import { FilmView } from "../assets/ui/film-view.tsx";

// An interactive film (og api show film): the story in assets/story/film.yarn,
// a segment per film node in assets/segments/, its footage in assets/clips/.
// The picture fills the screen (assets/ui/film-view.tsx), the built-in
// Subtitles shows the segment's captions and FilmChoices its window.

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }, what: string): T {
  if (!r.ok) throw new Error(`${what}: ${r.reason}`);
  return r.value;
}

/** The film og.film.start made. */
const FilmRef = defineResource("FilmRef", { film: Entity() });

/** What the screens show, written each tick from og.film.view and og.timeline.captions (film §4): world state, so a restore brings it back. */
function stores(world: World) {
  const scope = og.ui.scope(world, "film");
  return {
    view: scope.writable<View | null>("view", null),
    captions: scope.writable<readonly TimelineCaption[]>("captions", []),
  };
}

const presentFilm = defineSystem("PresentFilm", { reads: [FilmRef] }, (_q, _time, world) => {
  const ref = world.resource(FilmRef);
  if (!ref.ok) return;
  const s = stores(world);
  s.view.set(og.film.view(world, ref.value.film));
  s.captions.set(og.timeline.captions(world));
});

function film(world: World) {
  return must(world.resource(FilmRef), "FilmRef").film;
}

export default og.game({
  resources: [FilmRef],
  start(world) {
    must(og.story.load(world, "story/film.yarn"), "story");
    must(og.film.load(world, "film"), "film");
    // Paused: on the web, audio starts only after a gesture, so a title
    // screen's Start calls og.film.play (film §4); `film:play` in a script.
    must(world.resource(FilmRef), "FilmRef").film = must(og.film.start(world, { dialogue: "film", paused: true }), "start");
  },
  ui(world) {
    const s = stores(world);
    mount(world, () => {
      const view = useStore(s.view);
      return FilmView({
        get picture() { return view()?.texture ?? 0; },
        get previous() { return view()?.previous ?? 0; },
        get mix() { return view()?.mix ?? 1; },
      });
    }, {});
    mount(world, () => {
      const captions = useStore(s.captions);
      return Subtitles({ get captions() { return captions(); } });
    }, {});
    // A choice row or hotspot pressed: its index is og.film.choose's.
    mount(world, () => {
      const view = useStore(s.view);
      return FilmChoices({ get view() { return view(); }, onChoose: (e, index) => void og.film.choose(e.world, film(e.world), index) });
    }, {});
  },
  systems: [presentFilm],
  // What the probe's state() and a playtest's `state:` checks read (film §8).
  facts(world) {
    const ref = world.resource(FilmRef);
    const v = ref.ok ? og.film.view(world, ref.value.film) : null;
    return { film: v ? { node: v.node, ending: v.ending, ended: v.ended, open: v.window !== null, pending: v.window?.pending ?? -1 } : null };
  },
});
