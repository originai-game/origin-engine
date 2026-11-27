import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Heal: on the ground at the figure's feet.
export default file.vfx({
  "duration": 1.8,
  "events": [
    {
      "name": "impact",
      "time": 0
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
            "inner": 0.82,
            "segments": 48
          }
        },
        "effect": ref("base/vfx/toy-3d/fade"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.2
            ],
            [
              0.4,
              0.6
            ],
            [
              1,
              0.65
            ]
          ]
        },
        "color": [
          0.072,
          0.552,
          0.102,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d/emitters/heal-hearts")
      }
    },
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d/emitters/heal-plus")
      }
    },
    {
      "life": 1.2,
      "offset": [
        0,
        0.6,
        0
      ],
      "node": {
        "kind": "light",
        "color": [
          0.072,
          0.552,
          0.102
        ],
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              0
            ],
            [
              0.2,
              1.5
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
