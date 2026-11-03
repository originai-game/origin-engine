# Site: ocean

The live hero of the introduction site (engine.origingame.ai), rendered
by the engine in the visitor's browser: an open sea at golden hour under
`og.PhysicalSky` and volumetric `og.Clouds`, a wind-driven Gerstner sea
(`og.Water` with a `wind`), a two-masted schooner floated on it by
buoyancy (`src/ship.ts`: ten hull points sampled with
`og.physics.waterHeight` inside the tick, the same sea the renderer
draws), and a headland with a lighthouse on the far-left horizon
(`src/land.ts`). Every model is built from code.

It plays itself: a seamless 90 s loop (`src/loop.ts`) from golden hour
(the poster frame) into a storm with rain and lightning (`og.Weather`,
`og.WeatherState`), a moonlit night with the ship's lamps and the
lighthouse's turning beam, dawn, noon, and back to golden hour. The
camera rides about 2 m over the swell on a fixed path; every value is a
function of the loop's clock, so a frame at a given moment is always the
same. There is no UI and no input: the site frames the publish build in a
non-interactive iframe (`gate none`, `fit fill`, `isolation off`).

```sh
og run examples/site-ocean --size 1920x1080 --commands "4:screenshot:dusk.png;4.1:quit"
og run examples/site-ocean --commands "0:loop:30;3:screenshot:storm.png;3.1:quit"   # jump the loop
og build examples/site-ocean --publish
```

Commands (dev builds): `loop:<seconds>` jumps the loop (the ship moves
with it), `rate:<r>` plays it at r times (0 holds the moment while the sea
and clouds keep moving).
