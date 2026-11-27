import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Ripple: on the water's surface.
export default file.vfx({
  "duration": 1.8,
  "looping": true,
  "elements": [
    {
      "life": 1.2,
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
        "effect": ref("base/vfx/toy-3d/fade"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.1
            ],
            [
              1,
              0.9
            ]
          ]
        },
        "color": [
          0.791,
          0.922,
          1,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "at": 0.6,
      "life": 1.2,
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
        "effect": ref("base/vfx/toy-3d/fade"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.1
            ],
            [
              1,
              0.9
            ]
          ]
        },
        "color": [
          0.791,
          0.922,
          1,
          1
        ],
        "blend": "alpha"
      }
    }
  ]
});
