import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Shield: centred on the figure.
export default file.vfx({
  "duration": 2,
  "looping": true,
  "elements": [
    {
      "life": 2,
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "sphere"
        },
        "effect": ref("base/vfx/low-poly/bubble"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              1.1
            ],
            [
              0.5,
              1.14
            ],
            [
              1,
              1.1
            ]
          ]
        },
        "color": [
          0.262,
          0.578,
          1,
          0.5
        ],
        "blend": "alpha"
      }
    },
    {
      "life": 2,
      "offset": [
        0,
        -0.5,
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
            "inner": 0.9,
            "segments": 48
          }
        },
        "effect": ref("base/vfx/low-poly/solid"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.56
            ],
            [
              0.5,
              0.6
            ],
            [
              1,
              0.56
            ]
          ]
        },
        "spin": [
          0,
          0,
          60
        ],
        "color": [
          1.158,
          1.116,
          1.046,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/shield-glints")
      }
    }
  ]
});
