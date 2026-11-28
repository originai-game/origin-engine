import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Charge: where the power gathers.
export default file.vfx({
  "duration": 1.2,
  "looping": true,
  "elements": [
    {
      "life": 1.2,
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": ref("base/vfx/toy-3d/models/puff")
        },
        "effect": ref("base/vfx/toy-3d/solid"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.12
            ],
            [
              0.5,
              0.22
            ],
            [
              1,
              0.12
            ]
          ]
        },
        "color": [
          0.42,
          0.924,
          1.6,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d/emitters/charge-in")
      }
    },
    {
      "life": 1.2,
      "offset": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "light",
        "color": [
          0.262,
          0.578,
          1
        ],
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.8
            ],
            [
              0.5,
              1.6
            ],
            [
              1,
              0.8
            ]
          ]
        },
        "distance": 2.5
      }
    }
  ]
});
