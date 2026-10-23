// The views station (depth S8): a security monitor, a mirror and a
// split-screen view. The monitor shows the magenta crate on layer 1 that
// the main camera (layer 0) does not draw; the mirror shows the walker
// walking, so it changes between the two frames; the split view draws a
// second camera from above in its rect at the bottom right.
import { mean, stationPlaytest } from "./station.ts";

export default stationPlaytest("views", 1.5, ["vw"], ({ t, field, frame }) => {
  t.equal(field<number>("vw", "VW", "active"), 1, "the station's views are on");
  const magenta = (x0: number, y0: number, x1: number, y1: number) => {
    let n = 0;
    for (let y = y0; y < y1; y++) {
      for (let x = x0; x < x1; x++) {
        const i = (y * frame.width + x) * 4;
        const [r, g, b] = [frame.rgba[i]!, frame.rgba[i + 1]!, frame.rgba[i + 2]!];
        if (r > 150 && b > 150 && g < 90) n++;
      }
    }
    return n;
  };
  const onMonitor = magenta(820, 232, 1025, 345);
  t.ok(onMonitor > 15, `the monitor shows the layer-1 crate (${onMonitor} magenta pixels)`);
  const elsewhere = magenta(0, 0, frame.width, frame.height) - onMonitor;
  t.ok(elsewhere < 10, `the main camera does not draw it (${elsewhere} magenta pixels off the monitor)`);
  // The mirror is lit with the room it reflects, not left black.
  const glass = mean(frame, 130, 280, 360, 440);
  t.ok(glass[0] + glass[1] + glass[2] > 200, `the mirror shows the room (${glass.map((c) => c.toFixed(0)).join(",")})`);
  // The split view: the blue ball seen from above inside its rect, where
  // the main camera sees only floor and the red crate.
  let blue = 0;
  for (let y = 470; y < 690; y++) {
    for (let x = 900; x < 1250; x++) {
      const i = (y * frame.width + x) * 4;
      if (frame.rgba[i + 2]! > frame.rgba[i]! + 60) blue++;
    }
  }
  t.ok(blue > 150, `the split view draws the room from above (${blue} blue pixels of the ball in its rect)`);
});
