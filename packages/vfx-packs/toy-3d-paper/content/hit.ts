import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Hit: at the point struck.
export default file.vfx({
  "duration": 0.8,
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
        "asset": ref("base/vfx/toy-3d-paper/emitters/hit-stars")
      }
    },
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/hit-bits")
      }
    },
    {
      "life": 0.18,
      "offset": [
        0,
        0,
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
            "inner": 0.6,
            "segments": 20
          }
        },
        "effect": ref("base/vfx/toy-3d-paper/fade"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.05
            ],
            [
              1,
              0.5
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
