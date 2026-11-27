import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Pellet Trail: on the pellet, moved by the game.
export default file.vfx({
  "duration": 0.5,
  "looping": true,
  "elements": [
    {
      "life": 0.5,
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": ref("base/vfx/toy-3d-paper/models/coinFlat")
        },
        "effect": ref("base/vfx/toy-3d-paper/solid"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.14
            ],
            [
              1,
              0.14
            ]
          ]
        },
        "color": [
          0.75,
          0.522,
          0.115,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "life": 0.5,
      "node": {
        "kind": "trail",
        "lifetime": 0.16,
        "width": [
          0.1,
          0
        ],
        "effect": ref("base/vfx/toy-3d-paper/solid"),
        "color": [
          0.868,
          0.804,
          0.712,
          1
        ]
      }
    },
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/bullet-puffs")
      }
    }
  ]
});
