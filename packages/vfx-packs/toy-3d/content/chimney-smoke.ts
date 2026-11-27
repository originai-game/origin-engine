import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Chimney Smoke: at the chimney's top.
export default file.vfx({
  "duration": 2,
  "looping": true,
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d/emitters/chimney-puffs")
      }
    }
  ]
});
