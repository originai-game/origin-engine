import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Run Dust: at the foot, moved with the figure.
export default file.vfx({
  "duration": 0.4,
  "looping": true,
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d/emitters/run-puffs")
      }
    }
  ]
});
