import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Magic Bolt: on the bolt, moved by the game.
export default file.vfx({
  "duration": 0.6,
  "looping": true,
  "elements": [
    {
      "life": 0.6,
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": ref("base/vfx/toy-3d-paper/models/starFlat")
        },
        "effect": ref("base/vfx/toy-3d-paper/solid"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.24
            ],
            [
              1,
              0.24
            ]
          ]
        },
        "spin": [
          0,
          0,
          600
        ],
        "color": [
          0.574,
          0.393,
          0.947,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "life": 0.6,
      "node": {
        "kind": "trail",
        "lifetime": 0.24,
        "width": [
          0.14,
          0
        ],
        "effect": ref("base/vfx/toy-3d-paper/solid"),
        "color": [
          0.349,
          0.2,
          0.715,
          1
        ]
      }
    },
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/bolt-bits")
      }
    },
    {
      "life": 0.6,
      "offset": [
        0,
        0,
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
              1
            ],
            [
              1,
              1
            ]
          ]
        },
        "distance": 2
      }
    }
  ]
});
