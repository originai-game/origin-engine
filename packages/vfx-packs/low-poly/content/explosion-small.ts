import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Small Explosion: on the ground.
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
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/small-core")
      }
    },
    {
      "at": 0.03,
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/small-puffs")
      }
    },
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/small-chips")
      }
    },
    {
      "life": 0.28,
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
            "segments": 48
          }
        },
        "effect": ref("base/vfx/low-poly/fade"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.2
            ],
            [
              0.4,
              1.1
            ],
            [
              1,
              1.4
            ]
          ]
        },
        "color": [
          0.896,
          0.807,
          0.644,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "life": 0.3,
      "offset": [
        0,
        0.4,
        0
      ],
      "node": {
        "kind": "light",
        "color": [
          1,
          0.254,
          0.014
        ],
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              4
            ],
            [
              1,
              0
            ]
          ]
        },
        "distance": 4
      }
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.2
      }
    }
  ]
});
