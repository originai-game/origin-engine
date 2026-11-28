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
          "asset": ref("base/vfx/toy-3d/models/star")
        },
        "effect": ref("base/vfx/toy-3d/solid"),
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
          0.739,
          0.42,
          1.6,
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
        "effect": ref("base/vfx/toy-3d/solid"),
        "color": [
          0.407,
          0.164,
          1.266,
          1
        ]
      }
    },
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d/emitters/bolt-bits")
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
