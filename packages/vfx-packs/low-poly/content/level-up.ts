import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Level Up: on the ground at the figure's feet.
export default file.vfx({
  "duration": 2.2,
  "events": [
    {
      "name": "impact",
      "time": 0
    }
  ],
  "elements": [
    {
      "life": 0.9,
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
              0.2
            ],
            [
              0.3,
              0.9
            ],
            [
              1,
              1
            ]
          ]
        },
        "color": [
          1.3,
          0.838,
          0.055,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "at": 0.2,
      "life": 0.8,
      "offset": [
        0,
        0.7,
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
              0.2
            ],
            [
              0.3,
              0.75
            ],
            [
              1,
              0.8
            ]
          ]
        },
        "color": [
          1.3,
          1.154,
          0.33,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "at": 0.4,
      "life": 0.7,
      "offset": [
        0,
        1.4,
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
              0.15
            ],
            [
              0.3,
              0.6
            ],
            [
              1,
              0.65
            ]
          ]
        },
        "color": [
          1.254,
          1.209,
          1.133,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/level-stars")
      }
    },
    {
      "at": 0.9,
      "offset": [
        0,
        1.6,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/level-confetti")
      }
    },
    {
      "life": 1.6,
      "offset": [
        0,
        1,
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
              0.2,
              3
            ],
            [
              1,
              0
            ]
          ]
        },
        "distance": 4
      }
    }
  ]
});
