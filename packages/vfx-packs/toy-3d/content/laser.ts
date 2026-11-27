import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Toy Laser: from the emitter along +y, 4 m.
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
      "life": 0.45,
      "node": {
        "kind": "beam",
        "to": [
          0,
          4,
          0
        ],
        "width": 0.06,
        "effect": ref("base/vfx/toy-3d/solid"),
        "color": [
          1.775,
          0.113,
          0.063,
          1
        ]
      }
    },
    {
      "life": 0.45,
      "node": {
        "kind": "beam",
        "to": [
          0,
          4,
          0
        ],
        "width": 0.025,
        "effect": ref("base/vfx/toy-3d/solid"),
        "color": [
          1.929,
          1.86,
          1.743,
          1
        ]
      }
    },
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d/emitters/laser-bits")
      }
    },
    {
      "offset": [
        0,
        4,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d/emitters/laser-hit")
      }
    },
    {
      "life": 0.45,
      "offset": [
        0,
        0.2,
        0
      ],
      "node": {
        "kind": "light",
        "color": [
          0.807,
          0.051,
          0.028
        ],
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              1.5
            ],
            [
              1,
              0
            ]
          ]
        },
        "distance": 2.5
      }
    }
  ]
});
