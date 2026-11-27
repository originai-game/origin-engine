import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Small Splash: on the water's surface.
export default file.vfx({
  "duration": 1,
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
        "asset": ref("base/vfx/toy-3d-paper/emitters/small-crown")
      }
    },
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
            "inner": 0.86,
            "segments": 20
          }
        },
        "effect": ref("base/vfx/toy-3d-paper/fade"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.1
            ],
            [
              1,
              0.8
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
