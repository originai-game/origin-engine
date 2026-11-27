import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Portal: its foot on the ground, facing +z.
export default file.vfx({
  "duration": 2,
  "looping": true,
  "elements": [
    {
      "life": 2,
      "offset": [
        0,
        0.85,
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
            "inner": 0.82,
            "segments": 20
          }
        },
        "effect": ref("base/vfx/toy-3d-paper/solid"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.8
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
          90
        ],
        "color": [
          0.305,
          0.175,
          0.626,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "life": 2,
      "offset": [
        0,
        0.85,
        -0.01
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
            "inner": 0,
            "segments": 20
          }
        },
        "effect": ref("base/vfx/toy-3d-paper/solid"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.62
            ],
            [
              1,
              0.62
            ]
          ]
        },
        "color": [
          0.323,
          0.221,
          0.533,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "offset": [
        0,
        0.85,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/portal-stars")
      }
    },
    {
      "life": 2,
      "offset": [
        0,
        0.85,
        0.3
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
              1.5
            ],
            [
              1,
              1.5
            ]
          ]
        },
        "distance": 3
      }
    }
  ]
});
