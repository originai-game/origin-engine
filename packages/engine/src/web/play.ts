// The play page `og build` ships (dist/index.html loads dist/play.js): the
// player shell of shell/page.ts on the page's canvas, the engine Worker
// (dist/game.js), the cooked bundle next to the page, the Basis transcoder
// under basis/ and the audio worklet (dist/audio-worklet.js).
//
// - A `replay-seed` query parameter starts a deterministic replay from that
//   uint32 seed; `webgl=1` takes the WebGL2 fallback; `autoplay=0` leaves the
//   clock to the probe (no gate, no play), as test harnesses drive it.
// - The probe is published on `globalThis.__runtime_probe__`: every member
//   answers a Promise (the call crosses the request ring to the Worker, or
//   postMessage without cross-origin isolation), and the platform's og-probe
//   wire is served over it (shell/wire.ts).

import { play } from "../../shell/page.ts";

export { play };

void play().catch(() => {
  /* the error card is already on screen */
});
