// The three.js side of threejs-compare.ts: `globalThis.__bench_run__(warm,
// frames)` plays `warm` frames, then times `frames` more and answers the
// interval between consecutive requestAnimationFrame callbacks (ms; vsync and
// the frame-rate limit are off, so a frame starts as soon as the last one is
// done), with the simulation advanced exactly 1/60 s per frame, as the
// engine's `frame` call does.
export function bench(step) {
  globalThis.__bench_run__ = (warm, frames) =>
    new Promise((resolve, reject) => {
      const each = [];
      let n = 0, last = 0, t0 = 0;
      const tick = (now) => {
        try {
          if (n === warm) t0 = now;
          else if (n > warm) each.push(now - last);
          last = now;
          if (n === warm + frames) return resolve({ each, total: now - t0 });
          step(n / 60);
          n++;
          requestAnimationFrame(tick);
        } catch (e) {
          reject(e);
        }
      };
      requestAnimationFrame(tick);
    });
  // One more frame outside the timed run (the harness's screenshot keeps them coming).
  let t = 0;
  globalThis.__bench_step__ = () => step((t += 1 / 60));
  globalThis.__bench_ready__ = true;
}
export function fail(e) {
  globalThis.__bench_error__ = e instanceof Error ? `${e.message}\n${e.stack}` : String(e);
}
