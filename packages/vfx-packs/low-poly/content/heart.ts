import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Heart Pickup: where the heart was.
export default file.vfx({
  "duration": 1.2,
  "events": [
    {
      "name": "impact",
      "time": 0
    }
  ],
  "elements": [
    {
      "life": 0.6,
      "velocity": [
        0,
        1.2,
        0
      ],
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": ref("base/vfx/low-poly/models/heart")
        },
        "effect": ref("base/vfx/low-poly/solid"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.3
            ],
            [
              0.2,
              0.42
            ],
            [
              0.4,
              0.34
            ],
            [
              1,
              0
            ]
          ]
        },
        "color": [
          0.807,
          0.051,
          0.028,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "at": 0.4,
      "offset": [
        0,
        0.6,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/heart-little")
      }
    }
  ]
});
