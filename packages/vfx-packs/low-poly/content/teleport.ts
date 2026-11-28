import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Teleport: on the ground at the figure's feet.
export default file.vfx({
  "duration": 1.4,
  "events": [
    {
      "name": "impact",
      "time": 0.3
    }
  ],
  "elements": [
    {
      "life": 0.7,
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
            "inner": 0.8,
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
              0.55
            ],
            [
              1,
              0.6
            ]
          ]
        },
        "color": [
          0.341,
          0.751,
          1.3,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/teleport-column")
      }
    },
    {
      "at": 0.3,
      "offset": [
        0,
        0.5,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/teleport-poof")
      }
    },
    {
      "life": 0.8,
      "offset": [
        0,
        0.6,
        0
      ],
      "node": {
        "kind": "light",
        "color": [
          0.262,
          0.578,
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
