import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Campfire: on the ground.
export default file.vfx({
  "duration": 1,
  "looping": true,
  "elements": [
    {
      "offset": [
        0,
        0.05,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/camp-flames")
      }
    },
    {
      "offset": [
        0,
        0.2,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/camp-embers")
      }
    },
    {
      "offset": [
        0,
        0.6,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/camp-puffs")
      }
    },
    {
      "life": 1,
      "offset": [
        0,
        0.4,
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
              2
            ],
            [
              0.3,
              2.6
            ],
            [
              0.6,
              1.8
            ],
            [
              1,
              2
            ]
          ]
        },
        "distance": 4
      }
    }
  ]
});
