import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 1 "地面リング": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 2.0.0 "煙": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 3.0.0.0 "剣振り": a moving ancestor: its descendants are placed where it starts
// effekseer: node 3.0.0.0 "剣振り": nested generation: 5 instances above its generator folded into one emitter
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.0,
            "height": [
              0.0,
              2.5
            ],
            "colors": [
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
                1.0
              ],
              [
                1.0,
                1.0,
                1.0,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword11/n0-0.effect"
        },
        "scale": 0.0419,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.2833,
      "rotate": [
        0.0,
        0.0,
        50.0
      ],
      "life": 1.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.0,
            "height": [
              0.0,
              2.5
            ],
            "colors": [
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
                1.0
              ],
              [
                1.0,
                1.0,
                1.0,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword11/n0-1.effect"
        },
        "scale": 0.021,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.2833,
      "offset": [
        -0.0183,
        -0.012,
        0.0105
      ],
      "rotate": [
        32.7324,
        -24.4045,
        57.2676
      ],
      "life": 1.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.0,
            "height": [
              0.0,
              2.5
            ],
            "colors": [
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
                1.0
              ],
              [
                1.0,
                1.0,
                1.0,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword11/n0-2.effect"
        },
        "scale": 0.021,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.2833,
      "offset": [
        0.0087,
        0.0201,
        0.0105
      ],
      "rotate": [
        -32.7324,
        24.4045,
        57.2676
      ],
      "life": 1.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.1316,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword11/n1.effect"
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
              3.5818
            ],
            [
              0.1719,
              5.0709
            ],
            [
              0.2656,
              6.1696
            ],
            [
              0.375,
              7.0373
            ],
            [
              0.4688,
              7.4938
            ],
            [
              0.5781,
              7.7747
            ],
            [
              1.0,
              8.0
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
      "at": 0.3333,
      "life": 0.8333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword11/n2-0-0.emitter"
        }
      },
      "at": 0.3333,
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        0.0,
        7.0588
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword11/n3-0-0-0.effect"
        },
        "width": [
          5.25,
          1.3125
        ],
        "lifetime": 0.3333,
        "color": [
          1.0,
          1.0,
          1.0,
          0.3922
        ],
        "orbit": [
          3.702,
          2.25
        ]
      },
      "at": 0.1333,
      "offset": [
        6.0,
        0.0,
        3.0
      ],
      "rotate": [
        -180.0,
        62.0,
        0.0
      ],
      "life": 0.1167
    }
  ],
  "duration": 1.2833,
  "events": [
    {
      "name": "impact",
      "time": 0.3333
    }
  ]
});
