// The live scenes the page runs, as publish builds served from public/live/
// (scripts/live.ts builds them with `og build --publish`). null until the
// scene exists: the hero then shows its recording or its captured frame.
export const live: { ocean: string | null; motion: string | null } = {
  ocean: "/live/ocean/",
  motion: null,
};
