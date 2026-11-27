import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.2 "リング": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.4 "立体リング": a ring whose height changes over its life: its middle height kept
// effekseer: node 0.5.0.0 "砂煙": a fixed sprite turned at random: its mean plane drawn
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow10/n0-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        1.0,
        0.0,
        0.0
      ],
      "rotate": [
        90.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow10/n0-0-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        1.0,
        0.0,
        0.0
      ],
      "rotate": [
        90.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow10/n0-1.effect"
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
              4.4196
            ],
            [
              0.2031,
              6.7879
            ],
            [
              0.3125,
              8.726
            ],
            [
              0.4219,
              10.2962
            ],
            [
              0.6562,
              12.7121
            ],
            [
              1.0,
              15.0
            ]
          ]
        },
        "color": [
          1.0,
          0.0478,
          0.0478,
          0.1961
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "rotate": [
        -180.0,
        0.0,
        0.0
      ],
      "life": 0.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.2084,
            "band": 1.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                0.0478,
                0.0478,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow10/n0-2.effect"
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
          3.0,
          3.0,
          3.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0833,
      "rotate": [
        -180.0,
        0.0,
        0.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.8604,
            "band": 1.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow10/n0-3.effect"
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
          4.0,
          4.0,
          4.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0833,
      "rotate": [
        -180.0,
        0.0,
        0.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.0,
            "band": 1.0,
            "height": [
              0.0,
              -0.8475
            ],
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                0.0478,
                0.0478,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow10/n0-4.effect"
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
          3.0,
          3.0,
          3.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0833,
      "rotate": [
        -180.0,
        0.0,
        0.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow10/n0-5-0-0.emitter"
        }
      },
      "at": 0.0333,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          1.0,
          1.0
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
      "at": 0.0833,
      "life": 0.3
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.0833
    }
  ],
  "duration": 0.8665,
  "events": [
    {
      "name": "impact",
      "time": 0.0833
    }
  ]
});
