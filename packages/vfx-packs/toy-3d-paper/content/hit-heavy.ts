import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Heavy Hit: at the point struck.
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
        "asset": ref("base/vfx/toy-3d-paper/emitters/heavy-stars")
      }
    },
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/heavy-bits")
      }
    },
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/heavy-puffs")
      }
    },
    {
      "life": 0.22,
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
            "inner": 0.7,
            "segments": 20
          }
        },
        "effect": ref("base/vfx/toy-3d-paper/fade"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.08
            ],
            [
              1,
              0.9
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
    },
    {
      "at": 0.04,
      "life": 0.25,
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
            "inner": 0.85,
            "segments": 20
          }
        },
        "effect": ref("base/vfx/toy-3d-paper/fade"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.1
            ],
            [
              1,
              1.2
            ]
          ]
        },
        "color": [
          0.75,
          0.522,
          0.115,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.3
      }
    }
  ]
});
