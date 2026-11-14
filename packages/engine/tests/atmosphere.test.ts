// The day-night cycle through the TypeScript API on the native host: the
// clock is a function of the tick, and the sun og.DayNight drives is what
// `get` answers.
import { beforeAll, expect, test } from "bun:test";
import { og, defineResource, defineSystem, f32 } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type CreateOptions, type Engine, type GameToken } from "../src/runtime.ts";

beforeAll(() => {
  loadNativeHost();
});

function start(game: unknown, options: CreateOptions = {}): Engine {
  const r = createEngine(game as GameToken, options);
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

test("the day-night clock is the start plus the simulated time times the speed", () => {
  const Clock = defineResource("DayNightClock", { hours: f32() });
  const Read = defineSystem("DayNightReadClock", { reads: [og.DayNight], writes: [Clock] }, (_q, _t, world) => {
    must(world.resource(Clock)).hours = og.atmosphere.timeOfDay(world);
  });
  const engine = start(
    og.game({
      resources: [Clock],
      systems: [Read],
      start: (world: World) => {
        const d = must(world.resource(og.DayNight));
        d.enabled = true;
        d.start = 23;
        d.speed = 1;
      },
    }),
  );
  expect(engine.advance(120).ok).toBeTrue();
  // 119 ticks of 1/60 s at one game hour per second, past midnight.
  expect(og.atmosphere.timeOfDay(engine.world)).toBeCloseTo((23 + 120 / 60) % 24, 4);
  expect(must(engine.world.resource(Clock)).hours).toBeCloseTo((23 + 119 / 60) % 24, 4);
});

test("get answers the sun og.DayNight drives as it is drawn, and og.Time whole", () => {
  const engine = start(
    og.game({
      start: (world: World) => {
        const d = must(world.resource(og.DayNight));
        d.enabled = true;
        d.start = 12;
        d.speed = 0;
        for (const name of ["sun", "lamp"]) {
          const e = must(world.spawn({ name }));
          world.insert(e, og.DirectionalLight, { color: { r: 1, g: 1, b: 1 }, intensity: 2.4 });
        }
      },
    }),
  );
  const read = (script: string) => {
    expect(engine.probe.commands(script)).not.toBeNull();
    expect(engine.advance(1).ok).toBeTrue();
    return engine.takeOutputs().map((o) => (JSON.parse(o.data) as { value: Record<string, unknown> }).value);
  };
  const [sun, day, lamp, time] = read("0:get:sun.og.DirectionalLight;0:get:og.DayNight;0:get:lamp.og.DirectionalLight;0:get:og.Time");
  // Noon: the sun is overhead-ish, warm white and bright; the stored 2.4 is not what is drawn.
  expect(sun!.drivenBy).toBe("og.DayNight");
  expect(sun!.intensity).not.toBeCloseTo(2.4, 3);
  expect((sun!.toLight as number[])[1]).toBeGreaterThan(0.5);
  expect(day!.hours).toBeCloseTo(12, 4);
  expect(day!.lightIntensity).toBeCloseTo(sun!.intensity as number, 5);
  expect(day!.toLight).toEqual(sun!.toLight);
  // Only the first directional light is driven.
  expect("drivenBy" in lamp!).toBeFalse();
  expect(lamp!.intensity).toBeCloseTo(2.4, 5);
  // og.Time as a system reads it: the clock, its rate and the time scale.
  expect([time!.rate, time!.scale, time!.hitStopTicks]).toEqual([60, 1, 0]);
  expect(time!.scaledDt).toBeCloseTo(1 / 60, 6);
  // Midnight: the moon, cold and dim, opposite the sun.
  // (a script's times count from when it is scheduled)
  expect(engine.advance(59).ok).toBeTrue();
  const [night] = read("0:set:og.DayNight.start=0;0:get:sun.og.DirectionalLight");
  expect((night!.toLight as number[])[1]).toBeGreaterThan(0);
  expect(night!.intensity as number).toBeLessThan(0.5);
  expect((night!.color as number[])[2]).toBeGreaterThan((night!.color as number[])[0]!);
});
