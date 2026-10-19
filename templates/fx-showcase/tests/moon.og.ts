// The moon station: og.DayNight at 21:00 with moonSize and moonPhase. The
// waxing gibbous moon is one bright, round disc in the upper sky, lit more
// on its right; stars fill the rest of the sky and none shows through it.
import { stationPlaytest } from "./station.ts";

export default stationPlaytest("moon", 1, [], ({ t, frame }) => {
  const luma = (x: number, y: number) => {
    const i = (y * frame.width + x) * 4;
    return 0.2126 * frame.rgba[i]! + 0.7152 * frame.rgba[i + 1]! + 0.0722 * frame.rgba[i + 2]!;
  };
  // The disc: the bright pixels of the upper half right of the HUD panel.
  let [x0, y0, x1, y1, n] = [frame.width, frame.height, 0, 0, 0];
  for (let y = 0; y < frame.height / 2; y++) {
    for (let x = 340; x < frame.width - 2; x++) {
      // Bright and with bright neighbours: the disc, not a one-pixel star.
      if (luma(x, y) > 200 && luma(x - 2, y) > 200 && luma(x + 2, y) > 200) {
        n++;
        [x0, y0, x1, y1] = [Math.min(x0, x), Math.min(y0, y), Math.max(x1, x), Math.max(y1, y)];
      }
    }
  }
  const w = x1 - x0 + 1;
  const h = y1 - y0 + 1;
  t.ok(n > 400 && n < 8000, `one moon disc (${n} bright pixels)`);
  t.ok(w / h > 0.6 && w / h < 1.4 && n > 0.5 * w * h, `round and filled (${w} x ${h})`);
  const cx = Math.round((x0 + x1) / 2);
  const cy = Math.round((y0 + y1) / 2);
  let [left, right] = [0, 0];
  for (let y = cy - (h >> 2); y < cy + (h >> 2); y++) {
    for (let x = x0; x < cx; x++) left += luma(x, y);
    for (let x = cx; x <= x1; x++) right += luma(x, y);
  }
  t.ok(right >= left, `waxing: the right half is the brighter (${right.toFixed(0)} against ${left.toFixed(0)})`);
  let stars = 0;
  for (let y = 0; y < frame.height / 2; y += 1) {
    for (let x = 340; x < frame.width - 2; x++) {
      const l = luma(x, y);
      if (l > 70 && luma(x - 2, y) < 60 && luma(x + 2, y) < 60 && (x < x0 - 40 || x > x1 + 40 || y < y0 - 40 || y > y1 + 40)) stars++;
    }
  }
  t.ok(stars > 40, `stars in the night sky (${stars} star pixels)`);
});
