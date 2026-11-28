import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Star Collect: where the star was.
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
      "life": 0.7,
      "velocity": [
        0,
        1.1,
        0
      ],
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": ref("base/vfx/low-poly/models/star")
        },
        "effect": ref("base/vfx/low-poly/solid"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.45
            ],
            [
              0.3,
              0.6
            ],
            [
              0.8,
              0.5
            ],
            [
              1,
              0
            ]
          ]
        },
        "spin": [
          0,
          720,
          0
        ],
        "color": [
          1,
          0.644,
          0.042,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "at": 0.6,
      "offset": [
        0,
        0.75,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/star-ring")
      }
    },
    {
      "at": 0.6,
      "life": 0.3,
      "offset": [
        0,
        0.75,
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
            "inner": 0.85,
            "segments": 48
          }
        },
        "effect": ref("base/vfx/low-poly/fade"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.15
            ],
            [
              1,
              0.9
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
    },
    {
      "life": 0.9,
      "offset": [
        0,
        0.6,
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
              1.2
            ],
            [
              0.7,
              1
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
