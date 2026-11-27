import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0 "斬撃": ring 5 instances: the first 4 drawn
// effekseer: node 2 "Node": ring 5 instances: the first 4 drawn
// effekseer: node 2 "Node": a ring whose thickness changes over its life: its middle thickness kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 1.0,
            "height": [
              0.0,
              0.5
            ],
            "colors": [
              [
                1.0,
                0.1275,
                0.1275,
                0.0
              ],
              [
                1.0,
                0.1275,
                0.1275,
                0.641
              ],
              [
                1.0,
                0.1275,
                0.1275,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword10/n0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.0625,
              2.0378
            ],
            [
              0.1562,
              1.8962
            ],
            [
              0.5938,
              0.5745
            ],
            [
              0.6875,
              0.4548
            ],
            [
              0.9219,
              0.3745
            ],
            [
              0.9844,
              0.1969
            ],
            [
              1.0,
              0.0
            ]
          ]
        },
        "color": [
          5.0,
          5.0,
          5.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        13.0,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -180.0
      ],
      "velocity": [
        -120.0,
        0.0,
        0.0
      ],
      "acceleration": [
        360.0,
        0.0,
        0.0
      ],
      "life": 0.75
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword10/n1-0.emitter"
        }
      },
      "at": 0.1667,
      "offset": [
        -9.0,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -93.3019
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.8336,
            "band": 0.5,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                0.0278,
                0.0278,
                0.0
              ],
              [
                1.0,
                1.0,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword10/n2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.0781,
              2.7876
            ],
            [
              0.1719,
              3.5209
            ],
            [
              0.2656,
              4.054
            ],
            [
              0.375,
              4.4663
            ],
            [
              0.5625,
              4.7916
            ],
            [
              0.875,
              4.8903
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          5.0,
          5.0,
          5.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.2,
      "offset": [
        -9.0,
        0.0,
        0.0
      ],
      "rotate": [
        180.0,
        -86.9999,
        180.0
      ],
      "life": 0.4167
    }
  ],
  "duration": 0.8367,
  "events": [
    {
      "name": "impact",
      "time": 0.2
    }
  ]
});
