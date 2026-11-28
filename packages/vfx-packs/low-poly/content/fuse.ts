import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Fuse: at the fuse's end.
export default file.vfx({
  "duration": 0.5,
  "looping": true,
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d/emitters/fuse-stars")
      }
    },
    {
      "life": 0.5,
      "offset": [
        0,
        0.05,
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
              0.35
            ],
            [
              0.5,
              0.5
            ],
            [
              1,
              0.35
            ]
          ]
        },
        "distance": 1.2
      }
    }
  ]
});
