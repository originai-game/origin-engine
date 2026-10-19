// The fluids station: eight og.Fluid2D in one frame. In the left tank the
// heavy blue fluid (density 3) released beside the light orange one slid
// under it (one group, mixing 0); in the right tank red poured on yellow
// (one group, mixing 1) blended into one orange, alike on both sides; each
// cup above shows its own colour.
import { mean, stationPlaytest } from "./station.ts";

export default stationPlaytest("fluids", 4, ["anim-physics"], ({ t, frame, field }) => {
  const heavy = field<number>("anim-physics", "AnimPhysics", "heavyY");
  const light = field<number>("anim-physics", "AnimPhysics", "lightY");
  t.ok(heavy + 0.2 < light, `the heavy fluid sank under the light one (heavy ${heavy.toFixed(2)} m, light ${light.toFixed(2)} m)`);
  // World (x, y) to the 1280 × 720 frame: orthographic, 7.4 m tall, centred on (1700, 0.4).
  const k = 720 / 7.4;
  const at = (x: number, y: number) => [Math.round(640 + (x - 1700) * k), Math.round(360 - (y - 0.4) * k)] as const;
  const box = (x: number, y: number) => {
    const [px, py] = at(x, y);
    return mean(frame, px - 12, py - 8, px + 12, py + 8);
  };
  const left = box(1701.7, -1.75);
  const right = box(1703.5, -1.75);
  t.ok(Math.abs(left[1] - right[1]) < 30 && left[0] > left[2] + 60, `red and yellow blended into one orange (left ${left.map((c) => c.toFixed(0))}, right ${right.map((c) => c.toFixed(0))})`);
  const cups: Array<[number, number]> = [[1, 0], [2, 0], [0, 1], [-1, -1]];
  [-2.7, -0.9, 0.9, 2.7].forEach((cx, i) => {
    const c = box(1700 + cx, 2.55);
    const [hi, lo] = cups[i]!;
    if (hi < 0) t.ok(Math.min(...c) > 150, `cup ${i} holds its white fluid (${c.map((v) => v.toFixed(0))})`);
    else t.ok(c[hi]! > c[lo]! + 50, `cup ${i} holds its own colour (${c.map((v) => v.toFixed(0))})`);
  });
});
