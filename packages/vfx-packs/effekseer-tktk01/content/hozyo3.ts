import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0 "ナイフ": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0 "ナイフ軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0 "ナイフ軌跡": nested generation: 6 instances above its generator folded into one emitter
// effekseer: node 0.0.1 "*衝撃生業用": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.1 "*衝撃生業用": nested generation: 6 instances above its generator folded into one emitter
// effekseer: node 1 "ヘックスメイン右": a ring's arc over its life
// effekseer: node 1 "ヘックスメイン右": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 2 "ヘックス外右": a ring's arc over its life
// effekseer: node 2 "ヘックス外右": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 3 "ヘックスメイン左": a ring's arc over its life
// effekseer: node 3 "ヘックスメイン左": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 4 "ヘックス外左": a ring's arc over its life
// effekseer: node 4 "ヘックス外左": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 5 "ヘックス模様": a ring's arc over its life
// effekseer: node 5 "ヘックス模様": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 6 "ヘックスグロー": a ring's arc over its life
// effekseer: node 6 "ヘックスグロー": a UV region of a texture whose size is not known (not a PNG)
// effekseer: node 7 "ヘックスグロー外側": a ring's arc over its life
// effekseer: node 7 "ヘックスグロー外側": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 7 "ヘックスグロー外側": a UV region of a texture whose size is not known (not a PNG)
// effekseer: node 8 "ヘックスメイン外枠": a ring's arc over its life
// effekseer: node 8 "ヘックスメイン外枠": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 9 "ヘックスメイン余韻": a ring's arc over its life
// effekseer: node 9 "ヘックスメイン余韻": a ring whose thickness changes over its life: its middle thickness kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo3/n0-0.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo3/n0-0-0.effect"
        },
        "width": [
          1.5,
          0.375
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 0.0,
      "offset": [
        5.0,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        90.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo3/n0-0-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        5.75,
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
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9665,
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
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo3/n1.effect"
        },
        "scale": 2.3,
        "spin": [
          0.0,
          0.0,
          -180.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1333,
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9868,
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
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo3/n2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.76
            ],
            [
              0.1719,
              2.5246
            ],
            [
              0.375,
              2.3738
            ],
            [
              0.5781,
              2.3173
            ],
            [
              1.0,
              2.3
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          -180.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1333,
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9665,
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
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo3/n3.effect"
        },
        "scale": 2.3,
        "spin": [
          0.0,
          0.0,
          180.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1333,
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9868,
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
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo3/n4.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.76
            ],
            [
              0.1719,
              2.5246
            ],
            [
              0.375,
              2.3738
            ],
            [
              0.5781,
              2.3173
            ],
            [
              1.0,
              2.3
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          180.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1333,
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5988,
            "band": 0.5,
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
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo3/n5.effect"
        },
        "scale": 2.3,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1333,
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.4348,
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
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo3/n6.effect"
        },
        "scale": 2.3,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1333,
      "life": 0.5833
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.2637,
            "band": 0.5,
            "colors": [
              [
                0.2621,
                1.0,
                1.0,
                0.0
              ],
              [
                0.3971,
                1.0,
                1.0,
                0.2212
              ],
              [
                0.6096,
                1.0,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo3/n7.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.3
            ],
            [
              0.1094,
              3.1206
            ],
            [
              0.2031,
              3.695
            ],
            [
              0.2969,
              4.1375
            ],
            [
              0.4062,
              4.4929
            ],
            [
              0.5,
              4.6775
            ],
            [
              0.5938,
              4.7752
            ],
            [
              1.0,
              4.875
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
      "at": 0.1333,
      "life": 0.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9502,
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
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo3/n8.effect"
        },
        "scale": 3.9,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1333,
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9621,
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
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo3/n9.effect"
        },
        "scale": 2.3,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1333,
      "life": 0.5
    }
  ],
  "duration": 0.8,
  "events": [
    {
      "name": "impact",
      "time": 0.1333
    }
  ]
});
