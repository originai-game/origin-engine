import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Stomp: on the ground under the foot.
export default file.vfx({
  "duration": 1,
  "events": [
    {
      "name": "impact",
      "time": 0
    }
  ],
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/stomp-puffs")
      }
    },
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/stomp-stars")
      }
    },
    {
      "life": 0.3,
      "offset": [
        0,
        0.02,
        0
      ],
      "rotate": [
        -90,
        0,
        0
      ],
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.84,
            "segments": 20
          }
        },
        "effect": ref("base/vfx/toy-3d-paper/fade"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.2
            ],
            [
              1,
              1.3
            ]
          ]
        },
        "color": [
          0.849,
          0.782,
          0.673,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.25
      }
    }
  ]
});
