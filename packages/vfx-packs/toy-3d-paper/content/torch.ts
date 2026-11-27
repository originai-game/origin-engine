import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Torch: on the torch head.
export default file.vfx({
  "duration": 1,
  "looping": true,
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/torch-flames")
      }
    },
    {
      "life": 1,
      "offset": [
        0,
        0.15,
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
              1.2
            ],
            [
              0.5,
              1.5
            ],
            [
              1,
              1.2
            ]
          ]
        },
        "distance": 3
      }
    }
  ]
});
