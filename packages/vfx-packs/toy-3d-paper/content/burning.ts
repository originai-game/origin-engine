import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Burning: on the figure, up to 1 m.
export default file.vfx({
  "duration": 1,
  "looping": true,
  "elements": [
    {
      "offset": [
        0,
        0.5,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/burning-flames")
      }
    },
    {
      "offset": [
        0,
        1,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/burning-puffs")
      }
    },
    {
      "life": 1,
      "offset": [
        0,
        0.6,
        0
      ],
      "node": {
        "kind": "light",
        "color": [
          1,
          0.254,
          0.014
        ],
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              1.4
            ],
            [
              0.5,
              1.8
            ],
            [
              1,
              1.4
            ]
          ]
        },
        "distance": 2.5
      }
    }
  ]
});
