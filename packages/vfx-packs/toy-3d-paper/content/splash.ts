import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Splash: on the water's surface.
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
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/splash-crown")
      }
    },
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/splash-column")
      }
    },
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/splash-foam")
      }
    },
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
            "inner": 0.88,
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
              1,
              1.4
            ]
          ]
        },
        "color": [
          0.659,
          0.834,
          0.929,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "at": 0.2,
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
            "inner": 0.88,
            "segments": 20
          }
        },
        "effect": ref("base/vfx/toy-3d-paper/fade"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.15
            ],
            [
              1,
              1
            ]
          ]
        },
        "color": [
          0.659,
          0.834,
          0.929,
          1
        ],
        "blend": "alpha"
      }
    }
  ]
});
