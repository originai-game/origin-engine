// The boot rule for the lanes variant (docs/web-export.md §Lanes), shared by
// the shell, which starts the variant it picks, and the exported game's
// service worker (shell/sw.ts), which precaches the variant this rule will
// pick, so the two never disagree.

/** The most Workers lanes take beside the engine Worker. */
export const LANES_MAX_THREADS = 8;

/**
 * Lanes or not, decided at boot: the lanes variant needs shared memory
 * (`crossOriginIsolated`) and more than two logical cores. The pool takes
 * the cores but one, at most LANES_MAX_THREADS: the core left is the page's,
 * and the engine Worker needs none of its own while a lane runs, since it
 * waits for the pool (rayon blocks the calling thread until the lane ends).
 */
export function laneChoice(offered: boolean, isolated: boolean, cores: number): { threads: number } | { why: string } {
  if (!offered) return { why: "the build has no lanes variant" };
  if (!isolated) return { why: "the page is not cross-origin isolated (COOP/COEP)" };
  if (!(cores > 2)) return { why: `${cores || "unknown"} logical cores: single-threaded` };
  return { threads: Math.min(cores - 1, LANES_MAX_THREADS) };
}
