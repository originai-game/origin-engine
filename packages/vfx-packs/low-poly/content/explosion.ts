import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Explosion: on the ground.
export default file.vfx({
  "duration": 2,
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
        "asset": ref("base/vfx/low-poly/emitters/explosion-core")
      }
    },
    {
      "at": 0.04,
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/explosion-puffs")
      }
    },
    {
      "at": 0.08,
      "offset": [
        0,
        0.2,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/explosion-soot")
      }
    },
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/explosion-chips")
      }
    },
    {
      "life": 0.35,
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
              0.3
            ],
            [
              0.4,
              1.9
            ],
            [
              1,
              2.4
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
      "life": 0.4,
      "offset": [
        0,
        0.6,
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
              8
            ],
            [
              1,
              0
            ]
          ]
        },
        "distance": 6
      }
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      }
    }
  ]
});
