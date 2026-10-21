// The car: held on the line through the countdown, the throttle answered on
// the tick after the press, speed from a standing start, the brake, the
// steering (a slide with tyre smoke and a knock on the wall), the jump
// cleared at speed with a landing, a fall into the gap put back past it
// (the lap no longer counts), R.
import { playtest } from "@origingame/engine/playtest";
import { hold, position, press, race, RACING, script, value } from "./lib.ts";

export default playtest({
  name: "the car holds the line in the countdown, answers W at once, accelerates, brakes, slides, jumps the gap, and is put back after falling in",
  commands: script(
    "0.5:race",
    "0.6:get:car.og.Transform:grid.json",
    hold("KeyW", 1.0, 1.0),
    "1.9:get:car.og.Transform:held.json",
    // GO is three seconds after the race starts.
    "3.55:get:Race:go.json",
    "3.6:key_down:KeyW",
    "3.617:get:Race:answer.json",
    "6.6:get:Race:accel.json",
    "6.6:key_up:KeyW",
    hold("KeyS", 6.7, 1.2),
    "7.95:get:Race:braked.json",
    // A slide: full lock at 30 m/s on the sweeper's straight end, into the wall.
    "8.5:warp:150:30",
    "8.55:key_down:KeyW",
    "8.55:key_down:KeyA",
    "9.4:get:Race:sliding.json",
    "9.4:get:smoke-l.og.ParticleSystem:smoke.json",
    "10.2:key_up:KeyA",
    "10.2:key_up:KeyW",
    "10.3:get:Race:slid.json",
    // The jump, at 36 m/s from 60 m before the lip.
    "11:warp:205:36",
    "11.05:key_down:KeyW",
    "12.9:get:Race:air.json",
    "14.4:key_up:KeyW",
    "14.4:get:Race:landed.json",
    // Rolling off the lip at 8 m/s: into the gap, and put back past it after a second.
    "15:warp:229:8",
    "17.2:get:Race:fell.json",
    "19:get:Race:back.json",
    press("KeyR", 19.5),
    "19.8:get:Race:reset.json",
    // The pad: the right trigger is the throttle, the left stick steers.
    "20:warp:20:0",
    "20.2:gamepad:-:0,0:0,1",
    "21.7:get:Race:pad-throttle.json",
    "21.7:gamepad:-:-0.9,0:0,1",
    "22.2:get:Race:pad-steer.json",
    "22.2:get:car.Racer:pad-racer.json",
    "22.3:gamepad:-",
    "22.4:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    const [g, h] = [position(run, "grid.json"), position(run, "held.json")];
    t.ok(Math.hypot(g[0]! - h[0]!, g[2]! - h[2]!) < 0.05, "W in the countdown does not move the car off the line");
    t.equal(race(run, "go.json").phase, RACING, "the race is on three seconds after it starts");
    t.equal(value<{ inThrottle: number }>(run, "answer.json").inThrottle, 1, "the tick after W is pressed, the throttle is down");
    const accel = race(run, "accel.json");
    t.ok(accel.kmh > 70 && accel.kmh < 130, `three seconds of throttle from a standing start: ${accel.kmh.toFixed(1)} km/h (70 to 130)`);
    t.ok(accel.gear >= 3, `the gearbox shifted up (gear ${accel.gear})`);
    t.ok(race(run, "braked.json").kmh < 25, `S brakes it to ${race(run, "braked.json").kmh.toFixed(1)} km/h`);
    const slide = race(run, "sliding.json");
    t.ok(slide.skidTicks > 10, `full lock at speed slides the tail: ${slide.skidTicks} ticks of slide`);
    t.ok(value<{ startTick: number }>(run, "smoke.json").startTick > 8.5 * 60, "the rear tyres smoked in the slide (the emitter started after the warp)");
    t.ok(race(run, "slid.json").bumps >= 1, `the slide ends in the wall: ${race(run, "slid.json").bumps} knock(s)`);
    t.ok(race(run, "air.json").airborne, "off the lip the car is in the air");
    const landed = race(run, "landed.json");
    t.ok(landed.landings >= 1 && landed.jumps >= 1, `it lands the jump (${landed.jumps} jump, ${landed.landings} landing)`);
    t.ok(landed.longestAir > 40, `a real flight: ${landed.longestAir} ticks in the air`);
    t.equal(landed.resets, 0, "the jump at speed needs no reset");
    t.ok(race(run, "fell.json").offTicks > 0, "rolled off the lip slowly, the car is down in the gap");
    const back = race(run, "back.json");
    t.equal(back.resets, 1, "a second in the gap puts the car back on the road");
    t.equal(back.valid, false, "a lap with a reset does not count");
    // Put back past the gap: three samples past the landing ramp's top (src/race.ts).
    t.ok(back.at - 20 >= 241 && back.at - 20 <= 250, `put back past the gap (sample ${back.at - 20} from the line)`);
    t.equal(race(run, "reset.json").resets, 2, "R puts the car back too");
    t.ok(race(run, "pad-throttle.json").kmh > 25, `the pad's right trigger drives (${race(run, "pad-throttle.json").kmh.toFixed(1)} km/h after 1.5 s)`);
    t.ok(race(run, "pad-steer.json").inSteer < -0.8, `the left stick steers left (asked ${race(run, "pad-steer.json").inSteer})`);
    t.ok(value<{ steer: number }>(run, "pad-racer.json").steer < -0.5, "and the wheel follows");
  },
});
