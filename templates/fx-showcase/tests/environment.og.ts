// The environment station: spheres on a plinth lit by a cooked HDR
// environment map (og.load.environment, og.Environment), the map behind
// them, AgX and adapting auto exposure. The mirror sphere reflects the
// studio sharply (its horizon is an edge), the chalk sphere's shading is a
// smooth gradient; the background is the studio's sky above its floor;
// nothing moves, so the adapted exposure has settled: the frame is the one
// half a second before but for TAA's last flicker.
import { mean, moved, stationPlaytest, type Station } from "./station.ts";

type Img = Station["frame"];
const luma = (f: Img, x: number, y: number) => {
  const i = (y * f.width + x) * 4;
  return 0.2126 * f.rgba[i]! + 0.7152 * f.rgba[i + 1]! + 0.0722 * f.rgba[i + 2]!;
};
/** The largest step of luma down a column: an edge in the reflection. */
const sharpest = (f: Img, x: number, y0: number, y1: number) => {
  let best = 0;
  for (let y = y0; y < y1; y++) best = Math.max(best, Math.abs(luma(f, x, y + 1) - luma(f, x, y)));
  return best;
};

/** A column down the middle of the mirror sphere and of the chalk sphere. */
const MIRROR = { x: 308, y0: 295, y1: 405 };
const CHALK = { x: 988, y0: 300, y1: 410 };

export default stationPlaytest("environment", 1, [], ({ t, frame, before }) => {
  const mirror = sharpest(frame, MIRROR.x, MIRROR.y0, MIRROR.y1);
  const chalk = sharpest(frame, CHALK.x, CHALK.y0, CHALK.y1);
  t.ok(mirror > 3 * chalk, `the mirror reflects the studio sharply (a step of ${mirror.toFixed(1)} against the chalk's ${chalk.toFixed(1)})`);
  const sky = mean(frame, 0, 0, 1280, 60);
  const floor = mean(frame, 0, 660, 1280, 720);
  t.ok(sky[0] + sky[1] + sky[2] > floor[0] + floor[1] + floor[2] + 120, `the studio's sky is behind, brighter than its floor (${sky.map((c) => c.toFixed(0))} over ${floor.map((c) => c.toFixed(0))})`);
  const still = moved(before, frame);
  t.ok(still < 0.5, `the exposure has settled: next to nothing changed in half a second (${still.toFixed(3)} per channel)`);
});

