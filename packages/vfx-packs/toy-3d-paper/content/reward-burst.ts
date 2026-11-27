import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Reward Burst: where the reward is shown.
export default file.vfx({
  "duration": 1.6,
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
        "asset": ref("base/vfx/toy-3d-paper/emitters/reward-coins")
      }
    },
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/reward-stars")
      }
    },
    {
      "life": 0.3,
      "offset": [
        0,
        0.3,
        0
      ],
      "rotate": [
        0,
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
              0.15
            ],
            [
              1,
              1.2
            ]
          ]
        },
        "color": [
          0.868,
          0.804,
          0.712,
          1
        ],
        "blend": "alpha"
      }
    }
  ]
});
