import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0 "ナイフ": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0 "ナイフ軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0 "ナイフ軌跡": nested generation: 9 instances above its generator folded into one emitter
// effekseer: node 0.0.1.0 "*衝撃生業用": nested generation: 9 instances above its generator folded into one emitter
// effekseer: node 1.0 "衝撃線": a child of an instanced parent: its motion drawn up the world's y
// effekseer: node 2 "衝撃リング": a ring whose thickness changes over its life: its middle thickness kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword15/n0-0.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword15/n0-0-0.effect"
        },
        "width": [
          2.0,
          0.5
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 0.0,
      "offset": [
        -7.0,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ],
      "life": 0.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword15/n0-0-1-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        -8.0,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ],
      "velocity": [
        30.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword15/n1-0.parent"
        }
      },
      "at": 0.1
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword15/n1-1.emitter"
        }
      },
      "at": 0.1,
      "rotate": [
        0.0,
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
            "inner": 0.6251,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword15/n2.effect"
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
              2.7909
            ],
            [
              0.1719,
              3.5354
            ],
            [
              0.2656,
              4.0848
            ],
            [
              0.375,
              4.5186
            ],
            [
              0.4688,
              4.7469
            ],
            [
              0.5781,
              4.8874
            ],
            [
              1.0,
              5.0
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
      "at": 0.1,
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword15/n3.emitter"
        }
      },
      "at": 0.1
    }
  ],
  "duration": 2.275,
  "events": [
    {
      "name": "impact",
      "time": 0.1
    }
  ]
});
