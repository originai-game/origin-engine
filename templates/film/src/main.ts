import { og } from "@origingame/engine";

// An interactive film: the story in assets/story/film.yarn, a segment per
// film node in assets/segments/, its footage in assets/clips/ (og api show film).
export default og.game({
  start(world) {
    for (const step of [() => og.story.load(world, "story/film.yarn"), () => og.film.load(world, "film"), () => og.film.start(world, { dialogue: "film", paused: true })]) {
      const r = step();
      if (!r.ok) throw new Error(r.reason);
    }
  },
});
