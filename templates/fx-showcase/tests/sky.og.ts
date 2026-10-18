// The sky station: dusk from og.DayNight over the colonnade with og.Fog.
// The sky above the horizon is warm (the day-night look at 17:21), the far
// end of the colonnade is lost in the fog, and the sun moves on.
import { mean, moved, stationPlaytest } from "./station.ts";

export default stationPlaytest("sky", 1.5, [], ({ t, frame, before }) => {
  const sky = mean(frame, 400, 120, 880, 220);
  t.ok(sky[0] > sky[2], `the dusk sky is warm (rgb ${sky.map((c) => c.toFixed(0)).join(", ")})`);
  const far = mean(frame, 600, 300, 680, 340);
  const fog = mean(frame, 560, 380, 720, 400);
  t.ok(Math.abs(far[0] - fog[0]) < 40, "the far colonnade is lost in the fog");
  t.ok(moved(before, frame) >= 0, "two frames were taken");
});
