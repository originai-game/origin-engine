import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Confetti Pop: on the ground, +y up.
export default file.vfx({
  "duration": 2.6,
  "events": [
    {
      "name": "impact",
      "time": 0
    }
  ],
  "elements": [
    {
      "offset": [
        0,
        0.1,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/confetti-0")
      }
    },
    {
      "offset": [
        0,
        0.1,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/confetti-1")
      }
    },
    {
      "offset": [
        0,
        0.1,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/confetti-2")
      }
    },
    {
      "offset": [
        0,
        0.1,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/confetti-3")
      }
    },
    {
      "offset": [
        0,
        0.1,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/confetti-4")
      }
    },
    {
      "life": 0.25,
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
              0.6
            ]
          ]
        },
        "color": [
          0.965,
          0.93,
          0.871,
          1
        ],
        "blend": "alpha"
      }
    }
  ]
});
