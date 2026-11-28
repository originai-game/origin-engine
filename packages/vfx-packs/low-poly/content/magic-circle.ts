import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Magic Circle: on the ground.
export default file.vfx({
  "duration": 3,
  "events": [
    {
      "name": "impact",
      "time": 0.3
    }
  ],
  "elements": [
    {
      "life": 3,
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
              0
            ],
            [
              0.1,
              1
            ],
            [
              0.9,
              1
            ],
            [
              1,
              0
            ]
          ]
        },
        "spin": [
          0,
          0,
          40
        ],
        "color": [
          0.601,
          0.341,
          1.3,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "life": 3,
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
        "effect": ref("base/vfx/low-poly/solid"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0
            ],
            [
              0.12,
              0.72
            ],
            [
              0.9,
              0.72
            ],
            [
              1,
              0
            ]
          ]
        },
        "spin": [
          0,
          0,
          -60
        ],
        "color": [
          1.3,
          0.207,
          0.509,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "life": 3,
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
            "inner": 0.7,
            "segments": 48
          }
        },
        "effect": ref("base/vfx/low-poly/solid"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0
            ],
            [
              0.14,
              0.45
            ],
            [
              0.9,
              0.45
            ],
            [
              1,
              0
            ]
          ]
        },
        "spin": [
          0,
          0,
          90
        ],
        "color": [
          0.601,
          0.341,
          1.3,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "at": 0.2,
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/circle-stars")
      }
    },
    {
      "life": 3,
      "offset": [
        0,
        0.3,
        0
      ],
      "node": {
        "kind": "light",
        "color": [
          0.462,
          0.262,
          1
        ],
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              0
            ],
            [
              0.1,
              1.6
            ],
            [
              0.9,
              1.6
            ],
            [
              1,
              0
            ]
          ]
        },
        "distance": 3.5
      }
    }
  ]
});
