import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Buff: on the ground at the figure's feet.
export default file.vfx({
  "duration": 1.4,
  "events": [
    {
      "name": "impact",
      "time": 0
    }
  ],
  "elements": [
    {
      "life": 0.6,
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
            "inner": 0.82,
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
              0.5,
              0.6
            ],
            [
              1,
              0.65
            ]
          ]
        },
        "color": [
          0.719,
          0.297,
          0.068,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/buff-arrows")
      }
    }
  ]
});
