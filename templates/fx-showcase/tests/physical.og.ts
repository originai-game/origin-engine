// The physical materials station: every MeshPhysicalMaterial lobe (og-tests'
// materials-physical scene as a game). The clear glass shows the plinth's
// bars behind it, the amber glass tints what it shows orange, the sheen
// rim is pink on dark blue, the iridescent metal ball is not grey.
import { mean, stationPlaytest } from "./station.ts";

export default stationPlaytest("physical", 1, [], ({ t, frame }) => {
  // The amber glass (third in front): its volume absorbs blue and green.
  const [ar, ag, ab] = mean(frame, 620, 420, 660, 460);
  t.ok(ar > ag + 40 && ag > ab, `the amber glass tints what is behind it (${ar.toFixed(0)} ${ag.toFixed(0)} ${ab.toFixed(0)})`);
  // The frosted glass (second in front) blurs the red and white bars: no
  // pixel of it is as saturated as the bars beside it.
  let sat = 0;
  for (let y = 395; y < 470; y++) {
    for (let x = 400; x < 490; x++) {
      const i = (y * frame.width + x) * 4;
      const [r, g, b] = [frame.rgba[i]!, frame.rgba[i + 1]!, frame.rgba[i + 2]!];
      if (r - Math.max(g, b) > 120) sat++;
    }
  }
  t.ok(sat < 50, `the frosted glass blurs the bars (${sat} saturated red pixels)`);
  // The iridescent metal ball (fifth at the back) shows film colours.
  let spread = 0;
  for (let y = 285; y < 325; y++) {
    for (let x = 725; x < 770; x++) {
      const i = (y * frame.width + x) * 4;
      const c = [frame.rgba[i]!, frame.rgba[i + 1]!, frame.rgba[i + 2]!];
      spread += Math.max(...c) - Math.min(...c);
    }
  }
  spread /= 40 * 45;
  t.ok(spread > 25, `the film colours the metal (mean channel spread ${spread.toFixed(1)})`);
});
