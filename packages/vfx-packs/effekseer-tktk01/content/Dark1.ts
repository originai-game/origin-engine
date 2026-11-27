import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0 "煙": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 1.0 "赤リング": ring 10 instances: the first 4 drawn
// effekseer: node 1.0 "赤リング": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 3.0 "登る柱": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 4.0 "中心衝撃": a fixed sprite turned at random: its mean plane drawn
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Dark1/n0-0-0.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.7165,
            "band": 0.35,
            "height": [
              0.5173,
              0.0
            ],
            "colors": [
              [
                0.6524,
                0.0303,
                0.82,
                1.0
              ],
              [
                1.0,
                0.0,
                0.0,
                0.5
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
          "$ogRef": "base/vfx/effekseer-tktk01/Dark1/n1-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.125
            ],
            [
              0.1875,
              2.3124
            ],
            [
              0.3906,
              3.2559
            ],
            [
              0.5781,
              3.8807
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          10.0,
          10.0,
          10.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.45,
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Dark1/n3-0.emitter"
        }
      },
      "at": 0.5333,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Dark1/n4-0.emitter"
        }
      },
      "at": 0.45,
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
          0.351,
          0.4176
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
      "at": 0.45,
      "life": 0.3
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.45
    }
  ],
  "duration": 1.6166,
  "events": [
    {
      "name": "impact",
      "time": 0.45
    }
  ]
});
