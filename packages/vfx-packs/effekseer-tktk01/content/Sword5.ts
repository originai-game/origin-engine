import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0 "斬撃": ring 5 instances: the first 4 drawn
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 1.0,
            "colors": [
              [
                0.1275,
                1.0,
                1.0,
                0.0
              ],
              [
                0.1275,
                1.0,
                1.0,
                0.0784
              ],
              [
                0.1275,
                1.0,
                1.0,
                0.5882
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword5/n0-0.effect"
        },
        "scale": 0.5477,
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
        -9.0,
        0.0,
        0.0
      ],
      "velocity": [
        50.0,
        0.0,
        0.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword5/n1-0-0.emitter"
        }
      },
      "at": 0.0667,
      "offset": [
        0.5,
        0.0,
        0.0
      ],
      "rotate": [
        -163.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.5,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.1275,
                1.0,
                1.0,
                1.0
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword5/n2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.1094,
              3.7318
            ],
            [
              0.2188,
              5.1914
            ],
            [
              0.3281,
              6.4048
            ],
            [
              0.4531,
              7.5234
            ],
            [
              0.5781,
              8.3929
            ],
            [
              0.7031,
              9.0519
            ],
            [
              1.0,
              10.0
            ]
          ]
        },
        "color": [
          4.0,
          4.0,
          4.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1,
      "rotate": [
        107.0,
        0.0,
        0.0
      ],
      "life": 0.5
    }
  ],
  "duration": 0.6,
  "events": [
    {
      "name": "impact",
      "time": 0.1
    }
  ]
});
