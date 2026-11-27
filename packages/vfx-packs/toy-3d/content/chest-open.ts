import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Chest Open: in the chest's mouth.
export default file.vfx({
  "duration": 1.8,
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
        "asset": ref("base/vfx/toy-3d/emitters/chest-coins")
      }
    },
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d/emitters/chest-stars")
      }
    },
    {
      "life": 1.2,
      "offset": [
        0,
        0.4,
        0
      ],
      "node": {
        "kind": "light",
        "color": [
          1,
          0.644,
          0.042
        ],
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              3
            ],
            [
              1,
              0
            ]
          ]
        },
        "distance": 3
      }
    }
  ]
});
