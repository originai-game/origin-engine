import { file } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Solid colour fading out over the last half of its life: rings and ripples.
export default file.effect({
  "blend": "alpha",
  "doubleSided": true,
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0,
        1
      ],
      [
        0.5,
        0.9
      ],
      [
        1,
        0
      ]
    ]
  }
});
