import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Land Dust: on the ground under the landing.
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
        "asset": ref("base/vfx/toy-3d-paper/emitters/land-puffs")
      }
    },
    {
      "life": 0.25,
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
            "inner": 0.86,
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
              0.9
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
    }
  ]
});
