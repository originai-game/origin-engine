# motion-feet

og.motion's drawn feet on two game rigs (the strider, Mixamo names with a
character controller; the stout, Unreal names, without one) over one course
(`src/main.ts`): a walk, a run that brakes into a running jump, a walk on,
a jump up onto a 0.4 m ledge, a crouch, a turn in place and a wave; the
stout walks, jumps in place and forward, turns and cheers.

What it checks:

- `tests/feet.og.ts` (`og test tests/fixtures/motion-feet`): `dump_motion`'s
  `footing`, which measures the feet from where they are drawn, holds the
  thresholds of `og api show animation` §2.1 for both agents. It needs a
  renderer and the motion model (`packages/engine/motion/model.safetensors`,
  `bun packages/engine/scripts/motion-model.ts ensure`); without a renderer
  the check is unchecked.
- `packages/engine/tests/web/motion-ticks.ts` (the `browser` lane): the
  course in headless Chrome on WebGPU on the GPU backend, once as the GPU
  answers and once with every og.motion readback 600 ms late: no frame is
  held for og.motion, the late run's Presenter falls behind the world, and
  both runs have the same world hash on every tick and the same probe
  captures. `packages/engine/tests/presentation.test.ts` runs the course
  natively three times (results asked for 600 ms late with
  `OG_MOTION_READBACK_DELAY_MS`, and under load) for the same screenshots
  and feet, and checks display frames after a fork.

Budget: under 10 s warm (measured 5–7 s on Metal at 640x360, with its cook).
