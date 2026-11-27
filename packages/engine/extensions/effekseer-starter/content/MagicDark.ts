import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 2.049999952316284,
  "looping": false,
  "events": [
    {
      "name": "impact",
      "time": 0
    }
  ],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        1.5,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/effekseer-starter/MagicDark/n0.emitter")
      }
    },
    {
      "at": 0,
      "life": 1.666700005531311,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        -90,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "mesh",
        "blend": "additive",
        "color": [
          0.5860000252723694,
          0.14499999582767487,
          1,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/MagicDark/n1-0.effect"),
        "faceCamera": false,
        "scale": 3,
        "shape": {
          "kind": "quad"
        },
        "spin": [
          0,
          0,
          120
        ]
      }
    },
    {
      "at": 0.0333000011742115,
      "life": 0,
      "offset": [
        0,
        1,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/effekseer-starter/MagicDark/n2.emitter")
      }
    },
    {
      "at": 0,
      "life": 0.30000001192092896,
      "offset": [
        0,
        0.11110000312328339,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "light",
        "color": [
          0.616599977016449,
          0.20829999446868896,
          1
        ],
        "distance": 12,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              8
            ],
            [
              1,
              0
            ]
          ]
        }
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "shake",
        "trauma": 0.4000000059604645
      }
    }
  ]
});
