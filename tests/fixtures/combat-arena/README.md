# Combat arena

og.systems.combat on the real stack: a fighter with a blade (an
og.HitVolume in front of it) and a dummy (og.HurtVolume, og.Health 40,
og.Hurt with 10 invulnerable ticks and a knockback). Three moves on
entities of their own (light 1, light 2, heavy) and four og.AttackLink
entities: a starter into light 1, light 1 into light 2, light 2 into the
heavy, each on the `attack` action (J, a 4-tick buffer) in ticks [4, 14).

Pressing J three times chains the combo. Light 2's hit lands inside the
dummy's i-frames and takes nothing, the heavy kills it and og.Died is sent
once. A fourth light strikes the dead dummy for nothing. The game's own
`Dodge` (K) cuts the attack only while og.Attack.cancelable holds: refused
in the active frames, taken in recovery. Every number is in `src/main.ts`.

Readers: its `tests/combo.og.ts` (the combo, two runs on one hash) and
`packages/engine/tests/combat.test.ts` (tick-exact windows and buffer, a
hit read after its move was cut, the cancel window, a hit stop, a restore
mid-swing).

Inputs: og-runtime, og-play, og-input, og-physics, packages/engine.
