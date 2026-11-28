import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Poisoned: on the figure.
export default file.vfx({
  "duration": 1.2,
  "looping": true,
  "elements": [
    {
      "offset": [
        0,
        0.5,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/poison-bubbles")
      }
    }
  ]
});
