import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Fountain: at the spout, +y up.
export default file.vfx({
  "duration": 1,
  "looping": true,
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d/emitters/fountain-jet")
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
            "segments": 48
          }
        },
        "effect": ref("base/vfx/toy-3d/solid"),
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
