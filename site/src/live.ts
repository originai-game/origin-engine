// The live scenes the page runs, as publish builds served from public/live/
// (scripts/live.ts builds them with `og build --publish`). null until the
// scene exists: the hero then shows its recording or its captured frame.
//
// `scene` names the project each slot runs. The hero is written for
// examples/site-hero ("Skyreach Meadow": its type sits on the scene's own
// dark left sky, with no page scrim); until that scene boots,
// examples/site-ocean stands in, labelled interim on the page and blocking
// a deploy in scripts/check.ts. Swapping is `{ path: "/live/hero/", scene:
// "site-hero" }` plus the hero's poster and film slots in media.ts.
export const HERO_SCENE = "site-hero";

export const live: Record<"hero" | "motion", { path: string | null; scene: string }> = {
  hero: { path: "/live/ocean/", scene: "site-ocean" },
  motion: { path: null, scene: "site-motion" },
};

/** The slot's live scene is a stand-in for the one the page is written for. */
export const heroInterim = live.hero.scene !== HERO_SCENE;
