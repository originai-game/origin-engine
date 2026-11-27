import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Falling Leaves: 3 m above the area it covers.
export default file.vfx({
  "duration": 3,
  "looping": true,
  "elements": [
    {
      "offset": [
        0,
        3,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/leaves-fall")
      }
    }
  ]
});
