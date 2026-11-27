import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Steam: at the vent, +y the way it blows.
export default file.vfx({
  "duration": 1,
  "looping": true,
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/steam-puffs")
      }
    }
  ]
});
