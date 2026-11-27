import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Confetti Rain: 3 m above the area it covers.
export default file.vfx({
  "duration": 2,
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
        "asset": ref("base/vfx/toy-3d-paper/emitters/confetti-rain-0")
      }
    },
    {
      "offset": [
        0,
        3,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/confetti-rain-1")
      }
    },
    {
      "offset": [
        0,
        3,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/confetti-rain-2")
      }
    },
    {
      "offset": [
        0,
        3,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/confetti-rain-3")
      }
    },
    {
      "offset": [
        0,
        3,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/confetti-rain-4")
      }
    }
  ]
});
