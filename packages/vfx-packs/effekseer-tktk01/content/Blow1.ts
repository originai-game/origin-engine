import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0 "Ring": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 1.0 "衝撃メイン": a fixed sprite turned at random: its mean plane drawn
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.2587,
            "band": 0.9,
            "colors": [
              [
                1.0,
                0.002,
                0.0041,
                1.0
              ],
              [
                1.0,
                0.0415,
                0.0415,
                0.7843
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow1/n0-0.effect"
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
              2.5273
            ],
            [
              0.1719,
              3.0236
            ],
            [
              0.2656,
              3.3899
            ],
            [
              0.375,
              3.6791
            ],
            [
              0.5781,
              3.9249
            ],
            [
              1.0,
              4.0
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "life": 0.75
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow1/n1-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        1.0,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow1/n2-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        1.5,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          0.6698,
          0.6698
        ],
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              8.0
            ],
            [
              1.0,
              0.0
            ]
          ]
        },
        "distance": 12.0
      },
      "at": 0.0,
      "life": 0.3,
      "offset": [
        0.5044,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.0
    }
  ],
  "duration": 0.775,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
