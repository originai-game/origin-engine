import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0 "粒の子ノード": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.0.0 "開幕粒": motion out from a circle: approximated along one direction
// effekseer: node 1 "後半リング": ring 30 instances: the first 4 drawn
// effekseer: node 1 "後半リング": a ring whose thickness changes over its life: its middle thickness kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Cure6/n0-0-0-0.parent"
        }
      },
      "at": 0.0,
      "rotate": [
        -15.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.94,
            "band": 0.35,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Cure6/n1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.0938,
              2.6802
            ],
            [
              0.1875,
              3.2193
            ],
            [
              0.2969,
              3.6955
            ],
            [
              0.4062,
              4.0339
            ],
            [
              0.6562,
              4.4272
            ],
            [
              1.0,
              4.5
            ]
          ]
        },
        "color": [
          30.0,
          30.0,
          30.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.5167,
      "rotate": [
        -70.0,
        0.0,
        0.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Cure6/n2-0.emitter"
        }
      },
      "at": 0.5
    }
  ],
  "duration": 1.2508,
  "events": [
    {
      "name": "impact",
      "time": 0.5167
    }
  ]
});
