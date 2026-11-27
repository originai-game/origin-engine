import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Frozen: on the figure.
export default file.vfx({
  "duration": 1.5,
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
        "asset": ref("base/vfx/toy-3d-paper/emitters/frozen-crystals")
      }
    },
    {
      "offset": [
        0,
        1,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/frozen-flakes")
      }
    }
  ]
});
