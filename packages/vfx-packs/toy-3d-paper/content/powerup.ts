import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Power-up: on the ground at the figure's feet.
export default file.vfx({
  "duration": 1.6,
  "events": [
    {
      "name": "impact",
      "time": 0
    }
  ],
  "elements": [
    {
      "life": 0.8,
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
            "segments": 20
          }
        },
        "effect": ref("base/vfx/toy-3d-paper/fade"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.2
            ],
            [
              0.5,
              0.7
            ],
            [
              1,
              0.8
            ]
          ]
        },
        "spin": [
          0,
          0,
          180
        ],
        "color": [
          0.63,
          0.438,
          0.096,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "at": 0.15,
      "life": 0.7,
      "offset": [
        0,
        0.6,
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
            "segments": 20
          }
        },
        "effect": ref("base/vfx/toy-3d-paper/fade"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.2
            ],
            [
              0.5,
              0.55
            ],
            [
              1,
              0.6
            ]
          ]
        },
        "color": [
          0.604,
          0.25,
          0.057,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "rotate": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/power-stars")
      }
    },
    {
      "life": 1.2,
      "offset": [
        0,
        0.8,
        0
      ],
      "node": {
        "kind": "light",
        "color": [
          1,
          0.644,
          0.042
        ],
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              0
            ],
            [
              0.15,
              2
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
