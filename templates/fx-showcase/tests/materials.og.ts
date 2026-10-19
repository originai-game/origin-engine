// The materials station: every StandardMaterialDesc feature on one board.
// The glossy clear coat holds a sharper, brighter highlight than the rough
// coat over the same rough red base; the normal-mapped sphere's bumps
// break its shading up where a plain sphere is smooth; the grille's holes
// show the ground through it.
import { stationPlaytest, type Station } from "./station.ts";

type Img = Station["frame"];
const luma = (f: Img, x: number, y: number) => {
  const i = (y * f.width + x) * 4;
  return 0.2126 * f.rgba[i]! + 0.7152 * f.rgba[i + 1]! + 0.0722 * f.rgba[i + 2]!;
};
const peak = (f: Img, cx: number, cy: number, r: number) => {
  let best = 0;
  for (let y = cy - r; y < cy + r; y++) for (let x = cx - r; x < cx + r; x++) best = Math.max(best, luma(f, x, y));
  return best;
};
/** Mean absolute difference between horizontal neighbours: how busy the shading is. */
const busy = (f: Img, cx: number, cy: number, r: number) => {
  let sum = 0;
  let n = 0;
  for (let y = cy - r; y < cy + r; y++) {
    for (let x = cx - r; x < cx + r; x++) {
      sum += Math.abs(luma(f, x + 1, y) - luma(f, x, y));
      n++;
    }
  }
  return sum / n;
};

export default stationPlaytest("materials", 1, [], ({ t, frame }) => {
  const glossy = peak(frame, 744, 315, 34);
  const rough = peak(frame, 845, 315, 34);
  t.ok(glossy > rough + 12, `the glossy coat's highlight is the brighter (${glossy.toFixed(0)} against ${rough.toFixed(0)})`);
  const bumps = busy(frame, 536, 315, 26);
  const plain = busy(frame, 435, 413, 26);
  t.ok(bumps > 2 * plain, `the normal map breaks the shading up (${bumps.toFixed(2)} against ${plain.toFixed(2)} on a plain sphere)`);
  // The grille: bright bars and darker ground in its holes.
  let bars = 0;
  let holes = 0;
  for (let y = 205; y < 255; y++) {
    for (let x = 420; x < 480; x++) {
      const i = (y * frame.width + x) * 4;
      const [r, g, b] = [frame.rgba[i]!, frame.rgba[i + 1]!, frame.rgba[i + 2]!];
      if (r > g && g > b + 40) bars++;
      else if (Math.abs(r - b) < 25) holes++;
    }
  }
  t.ok(bars > 200 && holes > 200, `the grille is cut by alphaTest (${bars} bar and ${holes} hole pixels)`);
});
