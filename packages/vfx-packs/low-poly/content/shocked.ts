import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Shocked: on the figure.
export default file.vfx({
  "duration": 0.6,
  "looping": true,
  "elements": [
    {
      "life": 0.08,
      "offset": [
        -0.2,
        0.4,
        0
      ],
      "node": {
        "kind": "beam",
        "to": [
          0.4,
          0.5,
          0.1
        ],
        "width": 0.07,
        "effect": ref("base/vfx/low-poly/solid"),
        "color": [
          2.4,
          2.131,
          0.61,
          1
        ]
      }
    },
    {
      "at": 0.15,
      "life": 0.08,
      "offset": [
        0.2,
        1.1,
        0
      ],
      "node": {
        "kind": "beam",
        "to": [
          -0.35,
          -0.5,
          0.1
        ],
        "width": 0.07,
        "effect": ref("base/vfx/low-poly/solid"),
        "color": [
          2.4,
          2.131,
          0.61,
          1
        ]
      }
    },
    {
      "at": 0.32,
      "life": 0.08,
      "offset": [
        0,
        0.2,
        0.15
      ],
      "node": {
        "kind": "beam",
        "to": [
          0.1,
          0.9,
          -0.25
        ],
        "width": 0.07,
        "effect": ref("base/vfx/low-poly/solid"),
        "color": [
          2.315,
          2.232,
          2.091,
          1
        ]
      }
    },
    {
      "at": 0.46,
      "life": 0.08,
      "offset": [
        -0.15,
        1.2,
        0
      ],
      "node": {
        "kind": "beam",
        "to": [
          0.3,
          -0.4,
          0
        ],
        "width": 0.07,
        "effect": ref("base/vfx/low-poly/solid"),
        "color": [
          2.4,
          2.131,
          0.61,
          1
        ]
      }
    },
    {
      "offset": [
        0,
        0.6,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/shock-stars")
      }
    },
    {
      "life": 0.6,
      "offset": [
        0,
        0.7,
        0
      ],
      "node": {
        "kind": "light",
        "color": [
          1,
          0.888,
          0.254
        ],
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              1.6
            ],
            [
              0.12,
              0.2
            ],
            [
              0.25,
              1.6
            ],
            [
              0.4,
              0.2
            ],
            [
              0.77,
              1.6
            ],
            [
              1,
              0.2
            ]
          ]
        },
        "distance": 3
      }
    }
  ]
});
