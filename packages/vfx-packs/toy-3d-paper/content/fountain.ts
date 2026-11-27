import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Fountain: at the spout, +y up.
export default file.vfx({
  "duration": 1,
  "looping": true,
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/fountain-jet")
      }
    },
    {
      "life": 1,
      "offset": [
        0,
        0.01,
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
            "inner": 0.85,
            "segments": 20
          }
        },
        "effect": ref("base/vfx/toy-3d-paper/solid"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.5
            ],
            [
              0.5,
              0.6
            ],
            [
              1,
              0.5
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
