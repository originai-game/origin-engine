import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 3.0 "〆リング": ring 50 instances: the first 4 drawn
// effekseer: node 3.0 "〆リング": a ring whose thickness changes over its life: its middle thickness kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword7/n0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        15.0,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        90.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword7/n1.emitter"
        }
      },
      "at": 0.0833
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword7/n2.emitter"
        }
      },
      "at": 0.0833
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9801,
            "band": 0.55,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.142,
                0.4368,
                0.8513,
                0.1961
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword7/n3-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.25
            ],
            [
              0.0781,
              3.0409
            ],
            [
              0.1719,
              3.7854
            ],
            [
              0.2656,
              4.3348
            ],
            [
              0.375,
              4.7686
            ],
            [
              0.4688,
              4.9969
            ],
            [
              0.5781,
              5.1374
            ],
            [
              1.0,
              5.25
            ]
          ]
        },
        "color": [
          50.0,
          50.0,
          50.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0833,
      "rotate": [
        -75.0,
        0.0,
        0.0
      ],
      "life": 0.5
    }
  ],
  "duration": 0.5833,
  "events": [
    {
      "name": "impact",
      "time": 0.0833
    }
  ]
});
