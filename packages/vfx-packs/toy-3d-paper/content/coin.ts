import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Coin Collect: where the coin was.
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
      "life": 0.45,
      "velocity": [
        0,
        1.6,
        0
      ],
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": ref("base/vfx/toy-3d-paper/models/coinFlat")
        },
        "effect": ref("base/vfx/toy-3d-paper/solid"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.3
            ],
            [
              0.4,
              0.36
            ],
            [
              1,
              0
            ]
          ]
        },
        "spin": [
          0,
          1100,
          0
        ],
        "color": [
          0.662,
          0.393,
          0.059,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "at": 0.35,
      "offset": [
        0,
        0.55,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/coin-stars")
      }
    }
  ]
});
