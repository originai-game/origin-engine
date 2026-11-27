import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Small Hit: at the point struck.
export default file.vfx({
  "duration": 0.6,
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
        "asset": ref("base/vfx/toy-3d-paper/emitters/small-stars")
      }
    },
    {
      "life": 0.14,
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
              0.04
            ],
            [
              1,
              0.3
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
