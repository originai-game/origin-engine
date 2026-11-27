import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 1.6299999952316284,
  "looping": false,
  "events": [
    {
      "name": "impact",
      "time": 0.583299994468689
    }
  ],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        3,
        0
      ],
      "rotate": [
        0,
        0,
        180
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
        "asset": ref("base/vfx/effekseer-starter/HealPotion2/n0.emitter")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        3,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        -5.25,
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
        "asset": ref("base/vfx/effekseer-starter/HealPotion2/n0-0.emitter")
      }
    },
    {
      "at": 0.666700005531311,
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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/effekseer-starter/HealPotion2/n1.emitter")
      }
    },
    {
      "at": 0.583299994468689,
      "life": 0.33329999446868896,
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
          0.7968999743461609,
          1,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/HealPotion2/n2.effect"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0
            ],
            [
              0.2655999958515167,
              0.9151999950408936
            ],
            [
              0.5,
              2.149399995803833
            ],
            [
              0.75,
              3.949700117111206
            ],
            [
              1,
              6.249800205230713
            ]
          ]
        },
        "shape": {
          "kind": "quad"
        },
        "spin": [
          0,
          0,
          0
        ]
      }
    },
    {
      "at": 0.666700005531311,
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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/effekseer-starter/HealPotion2/n3.emitter")
      }
    }
  ]
});
