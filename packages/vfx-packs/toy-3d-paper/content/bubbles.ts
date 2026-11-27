import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Bubbles: under water, rising.
export default file.vfx({
  "duration": 1.5,
  "looping": true,
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/bubbles-rise")
      }
    }
  ]
});
