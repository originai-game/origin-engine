// The weather station: rain (og.Weather) over a yard. Streaks move between
// two frames half a second apart (many pixels change sharply while the
// yard stands still), and the preset's fog greys the distance.
import { changed, mean, stationPlaytest } from "./station.ts";

export default stationPlaytest("weather", 4.5, [], ({ t, frame, before }) => {
  const n = changed(before, frame);
  t.ok(n > 60, `the rain moves (${n} pixels changed sharply)`);
  const far = mean(frame, 500, 300, 780, 360);
  const spread = Math.max(...far) - Math.min(...far);
  t.ok(spread < 40, `the distance is greyed by the rain's fog (spread ${spread.toFixed(1)})`);
});
