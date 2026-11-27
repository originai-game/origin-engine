import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Boat Wake: at the stern on the water, moved by the game.
export default file.vfx({
  "duration": 0.5,
  "looping": true,
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d/emitters/wake-foam")
      }
    },
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d/emitters/wake-drops")
      }
    }
  ]
});
