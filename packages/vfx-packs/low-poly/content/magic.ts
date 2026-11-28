import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Magic Hit: at the point struck.
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
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/magic-stars")
      }
    },
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/magic-puffs")
      }
    },
    {
      "life": 0.3,
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
            "inner": 0.84,
            "segments": 48
          }
        },
        "effect": ref("base/vfx/low-poly/fade"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.1
            ],
            [
              1,
              1.1
            ]
          ]
        },
        "color": [
          0.647,
          0.367,
          1.4,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "life": 0.4,
      "offset": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "light",
        "color": [
          0.462,
          0.262,
          1
        ],
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              3
            ],
            [
              1,
              0
            ]
          ]
        },
        "distance": 3
      }
    }
  ]
});
