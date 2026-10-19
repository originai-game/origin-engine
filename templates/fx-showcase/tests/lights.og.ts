// The lights station: a night plaza under 256 point and spot lights
// (Forward+). Nearly the whole plaza is lit, by lights of many hues: with
// the old eight-light loop only a strip far away was (0.1 % of these
// pixels above the threshold, where clustered lights give about 90 %).
import { stationPlaytest } from "./station.ts";

export default stationPlaytest("lights", 1, [], ({ t, frame }) => {
  let lit = 0;
  let n = 0;
  const hues = new Set<number>();
  for (let y = 300; y < 700; y++) {
    for (let x = 100; x < 1180; x++) {
      const i = (y * frame.width + x) * 4;
      const [r, g, b] = [frame.rgba[i]!, frame.rgba[i + 1]!, frame.rgba[i + 2]!];
      n++;
      if (0.2126 * r + 0.7152 * g + 0.0722 * b > 40) lit++;
      const max = Math.max(r, g, b);
      const min = Math.min(r, g, b);
      if (max > 80 && max - min > 40) {
        // A coarse hue: which channel leads, and which follows.
        hues.add((r === max ? 0 : g === max ? 3 : 6) + (r === min ? 0 : g === min ? 1 : 2));
      }
    }
  }
  t.ok(lit / n > 0.6, `the plaza is lit (${((100 * lit) / n).toFixed(1)} % of it)`);
  t.ok(hues.size >= 5, `by lights of many hues (${hues.size} coarse hues)`);
});
