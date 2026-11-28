import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Toy Rocket Trail: at the rocket's tail, -y behind it.
export default file.vfx({
  "duration": 1,
  "looping": true,
  "elements": [
    {
      "rotate": [
        180,
        0,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/rocket-flame")
      }
    },
    {
      "offset": [
        0,
        -0.12,
        0
      ],
      "rotate": [
        180,
        0,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/rocket-puffs")
      }
    },
    {
      "life": 1,
      "offset": [
        0,
        -0.1,
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
