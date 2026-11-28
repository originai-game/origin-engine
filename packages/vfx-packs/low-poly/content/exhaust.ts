import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Kart Exhaust: at the exhaust pipe, +y out of it.
export default file.vfx({
  "duration": 0.6,
  "looping": true,
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d/emitters/exhaust-puffs")
      }
    }
  ]
});
