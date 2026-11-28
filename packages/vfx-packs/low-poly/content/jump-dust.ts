import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Jump Dust: on the ground under the jump.
export default file.vfx({
  "duration": 0.8,
  "events": [
    {
      "name": "impact",
      "time": 0
    }
  ],
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/jump-puffs")
      }
    }
  ]
});
