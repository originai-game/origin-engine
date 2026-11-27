# Checked effect declarations

Particles, effect materials and composed effects each have one author entry: a default export from `content/**/*.ts`, importing `file` and `ref` from `@origingame/engine/author`. Their complete checked inputs are `og api show file:particles`, `file:effect` and `file:vfx`. The cook evaluates TS data into native fields and resource tables. Runtime loading only binds those resources; it never parses author text.

## 1. Emitters

Use `file.particles` for a lasting source on an og.ParticleSystem, or for the emitter node of a composed effect. Share pure data or helper functions through ordinary TS imports. Texture, array, model, effect, field and sub-emitter references use canonical content IDs, not sibling filenames. Built-in engine UI declarations retain their reserved `@ui/` and `@styles/` identities.

An array needs sprite rendering. A field reference consumes the same checked lattice or image as other native consumers; a lattice must be cubic with three channels. Sub-emitter references are checked for the supported depth and cycles. CPU and GPU simulation consume the same native emitter fields.

For a lit mesh, `translucency` sets the share of far-side light transmitted through its colour, from 0 (the default) to 1. Thin paper, leaves and confetti stay bright when turned from the sun. Non-mesh rendering refuses a value above zero.

## 2. Effect materials

Use `file.effect` for a particle, mesh, trail or beam's material. A material shader is another checked declaration, referenced by canonical ID. Named parameters are checked against that shader's actual metadata at cook; the runtime binds its resource handle without reevaluating source.

## 3. Composed effects

Use `file.vfx` for one timeline of emitter, mesh, trail, beam, light, decal, glow, shake, sound and flipbook nodes. Timings are in seconds and land on `startTick + round(time × tickRate)`. Events and seeded samples use the authored element order. A `flipbook` node passes its clip events through as og.VfxEvent, including a pixel effect's impact.

Use `effects §2` for a composition and `effects §5` for composing source-volume textures. Load the canonical `<module>/<content-relative-stem>` with og.load.emitter, og.load.effect or og.load.vfx. Preview composed effects by that same ID. Pixel recipes and sheets are `file.vfx2d` and `file.flipbook` (`pixel-vfx`).

## 4. Source textures

A checked `file["effect-texture"]` declaration generates a scattering sheet, its thermal `#emission` sheet, chronological `#array` and `#emission#array`, and their real `#layerN` outputs. Refer only to outputs the producer reports. Ordinary imports, explicit input bytes and checked data all participate in cook invalidation. There is no EffectSource wrapper, generated text bridge or OGT author fallback; re-cook with matching hosts after upgrading.
