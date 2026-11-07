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

Budget: under 10 s warm (measured 5–7 s on Metal at 640x360, with its cook).
