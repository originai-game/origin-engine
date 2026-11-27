import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Fireflies: over the area, 0.2 to 1.5 m up.
export default file.vfx({
  "duration": 3,
  "looping": true,
  "elements": [
    {
      "offset": [
        0,
        0.85,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d/emitters/fireflies-drift")
      }
    }
  ]
});
