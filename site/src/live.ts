// The live scenes the page runs, as publish builds served from public/live/
// (scripts/live.ts builds them with `og build --publish`). null until the
// scene exists: the slot then shows its recording or its captured frame.
//
// The hero is written for examples/site-hero: one publish build with one
// world per scene, which the page rotates through (Hero.astro). The page
// tells the game which scene to preload and show with a Player protocol 2
// `platform` notice of kind `host` ({ name: "hero.preload" | "hero.show",
// scene }), and the game answers through og.platform.telemetry
// (`game.custom.hero.loaded | hero.ready | hero.fail`, attrs { scene }).
// Each scene enters at its own t=0, so its poster is its first live frame
// and the page's dissolve through the poster hides the switch.
//
// Until site-hero boots, examples/site-ocean stands in as a single scene,
// labelled interim on the page and blocking a deploy (scripts/check.ts).
// Swapping is `{ path: "/live/hero/", scene: "site-hero" }`.
import type { SlotName } from "./media";

export const HERO_SCENE = "site-hero";

export const live: Record<"hero" | "motion", { path: string | null; scene: string }> = {
  hero: { path: "/live/ocean/", scene: "site-ocean" },
  motion: { path: null, scene: "site-motion" },
};

/** The hero slot's live build is a stand-in for the one the page is written for. */
export const heroInterim = live.hero.scene !== HERO_SCENE;

export interface HeroScene {
  /** The scene's id in site-hero's protocol (kebab-case). */
  id: string;
  name: { en: string; zh: string };
  /** Its t=0 capture: the poster, the reduced-motion still and the dissolve's target. */
  poster: SlotName;
  /** Its loop, recorded on studio-win: the fallback without WebGPU. */
  film: SlotName;
}

/** site-hero's scenes in the order the hero shows them (the art director picks the third and fourth). */
export const heroScenes: HeroScene[] = [
  { id: "skyreach-meadow", name: { en: "Skyreach Meadow", zh: "云端草甸" }, poster: "hero-skyreach-meadow-poster", film: "hero-skyreach-meadow-film" },
  { id: "lantern-shrine", name: { en: "Lantern Shrine", zh: "灯笼神社" }, poster: "hero-lantern-shrine-poster", film: "hero-lantern-shrine-film" },
];

/** The stand-in, while site-hero is in production. */
const ocean: HeroScene = { id: "ocean", name: { en: "Open ocean", zh: "开阔海面" }, poster: "hero-ocean-poster", film: "hero-ocean-film" };

/** What the hero actually runs today. */
export const heroRunning: HeroScene[] = heroInterim ? [ocean] : heroScenes;

/** How long the hero holds each scene before it dissolves to the next. */
export const HERO_DWELL_MS = 24_000;
