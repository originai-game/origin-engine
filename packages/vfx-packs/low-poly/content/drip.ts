import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Drip: the drop falls 1.5 m to the origin.
export default file.vfx({
  "duration": 1.2,
  "looping": true,
  "elements": [
    {
      "offset": [
        0,
        1.5,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d/emitters/drip-drop")
      }
    },
    {
      "at": 0.55,
      "life": 0.6,
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
        "effect": ref("base/vfx/toy-3d/fade"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.04
            ],
            [
              1,
              0.3
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
