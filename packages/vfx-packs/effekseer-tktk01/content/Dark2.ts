import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 2 "広がる魔法陣": a mesh's colour over its life: its brightest kept, alpha over life kept
// effekseer: node 3 "押さえつけリング": ring 5 instances: the first 4 drawn
// effekseer: node 3 "押さえつけリング": a ring whose height changes over its life: its middle height kept
// effekseer: node 4 "押さえつけリング中心": ring 6 instances: the first 4 drawn
// effekseer: node 4 "押さえつけリング中心": a ring whose height changes over its life: its middle height kept
// effekseer: node 5.0 "ライン下降軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 5.0 "ライン下降軌跡": nested generation: 50 instances above its generator folded into one emitter
// effekseer: node 6 "魔法陣衝撃用グロー": ring 5 instances: the first 4 drawn
// effekseer: node 6 "魔法陣衝撃用グロー": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 7 "魔法陣衝撃立体リング": ring 5 instances: the first 4 drawn
// effekseer: node 7 "魔法陣衝撃立体リング": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 7 "魔法陣衝撃立体リング": a UV region of a texture whose size is not known (not a PNG)
// effekseer: node 8.0 "ライン下降軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 8.0 "ライン下降軌跡": nested generation: 50 instances above its generator folded into one emitter
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Dark2/n0.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Dark2/n1.emitter"
        }
      },
      "at": 0.5833
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Dark2/n2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              4.0
            ],
            [
              0.0781,
              4.5273
            ],
            [
              0.1719,
              5.0236
            ],
            [
              0.2656,
              5.3899
            ],
            [
              0.375,
              5.6791
            ],
            [
              0.5781,
              5.9249
            ],
            [
              1.0,
              6.0
            ]
          ]
        },
        "color": [
          0.4368,
          0.0415,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.75,
      "rotate": [
        0.0,
        0.0,
        90.0
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
            "band": 0.75,
            "height": [
              0.0,
              3.8272
            ],
            "colors": [
              [
                0.0278,
                0.0278,
                0.0677,
                0.0
              ],
              [
                0.1275,
                0.0,
                1.0,
                0.5882
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
          "$ogRef": "base/vfx/effekseer-tktk01/Dark2/n3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.4
            ],
            [
              0.875,
              1.5813
            ],
            [
              0.8906,
              1.6
            ],
            [
              1.0,
              1.6
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
      "life": 0.75
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 2.0,
            "band": 0.0,
            "height": [
              1.0617,
              1.0617
            ],
            "colors": [
              [
                0.0623,
                0.0278,
                0.3112,
                0.0
              ],
              [
                0.0278,
                0.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Dark2/n4.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.5
            ],
            [
              0.875,
              1.9766
            ],
            [
              0.8906,
              2.0
            ],
            [
              1.0,
              2.0
            ]
          ]
        },
        "color": [
          1.5,
          1.5,
          1.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "rotate": [
        0.0,
        0.0,
        -135.0
      ],
      "life": 0.75
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 2.0,
            "band": 0.0,
            "height": [
              1.0617,
              1.0617
            ],
            "colors": [
              [
                0.0623,
                0.0278,
                0.3112,
                0.0
              ],
              [
                0.0278,
                0.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Dark2/n4.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.5
            ],
            [
              0.875,
              1.9766
            ],
            [
              0.8906,
              2.0
            ],
            [
              1.0,
              2.0
            ]
          ]
        },
        "color": [
          1.5,
          1.5,
          1.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0017,
      "rotate": [
        0.0,
        0.0,
        -45.0
      ],
      "life": 0.75
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 2.0,
            "band": 0.0,
            "height": [
              1.0617,
              1.0617
            ],
            "colors": [
              [
                0.0623,
                0.0278,
                0.3112,
                0.0
              ],
              [
                0.0278,
                0.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Dark2/n4.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.5
            ],
            [
              0.875,
              1.9766
            ],
            [
              0.8906,
              2.0
            ],
            [
              1.0,
              2.0
            ]
          ]
        },
        "color": [
          1.5,
          1.5,
          1.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0033,
      "rotate": [
        0.0,
        0.0,
        45.0
      ],
      "life": 0.75
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 2.0,
            "band": 0.0,
            "height": [
              1.0617,
              1.0617
            ],
            "colors": [
              [
                0.0623,
                0.0278,
                0.3112,
                0.0
              ],
              [
                0.0278,
                0.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Dark2/n4.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.5
            ],
            [
              0.875,
              1.9766
            ],
            [
              0.8906,
              2.0
            ],
            [
              1.0,
              2.0
            ]
          ]
        },
        "color": [
          1.5,
          1.5,
          1.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.005,
      "rotate": [
        0.0,
        0.0,
        135.0
      ],
      "life": 0.75
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Dark2/n5-0.effect"
        },
        "width": [
          0.0,
          0.0
        ],
        "lifetime": 0.3333,
        "color": [
          0.1275,
          0.0278,
          0.7593,
          1.0
        ]
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.0,
        2.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Dark2/n5-1.emitter"
        }
      },
      "at": 0.3333,
      "offset": [
        0.0,
        0.0,
        2.0
      ],
      "rotate": [
        -90.0,
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
            "inner": 0.35,
            "band": 0.5,
            "colors": [
              [
                0.0278,
                0.0037,
                0.0677,
                0.0
              ],
              [
                0.0278,
                0.0037,
                0.0677,
                0.0
              ],
              [
                0.1275,
                0.0,
                0.4368,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Dark2/n6.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              1.0,
              8.0
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
      "at": 0.6667,
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5359,
            "band": 0.05,
            "height": [
              0.3517,
              0.0
            ],
            "colors": [
              [
                0.0143,
                0.0037,
                0.0677,
                0.0
              ],
              [
                0.0143,
                0.0037,
                0.0677,
                0.0
              ],
              [
                0.0278,
                0.0278,
                0.4647,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Dark2/n7.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.5
            ],
            [
              0.0781,
              3.1591
            ],
            [
              0.1719,
              3.7795
            ],
            [
              0.2656,
              4.2373
            ],
            [
              0.375,
              4.5989
            ],
            [
              0.5781,
              4.9061
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          1.25,
          1.25,
          1.25,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.75,
      "rotate": [
        0.0,
        0.0,
        -135.0
      ],
      "life": 0.8333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5359,
            "band": 0.05,
            "height": [
              0.3517,
              0.0
            ],
            "colors": [
              [
                0.0143,
                0.0037,
                0.0677,
                0.0
              ],
              [
                0.0143,
                0.0037,
                0.0677,
                0.0
              ],
              [
                0.0278,
                0.0278,
                0.4647,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Dark2/n7.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.5
            ],
            [
              0.0781,
              3.1591
            ],
            [
              0.1719,
              3.7795
            ],
            [
              0.2656,
              4.2373
            ],
            [
              0.375,
              4.5989
            ],
            [
              0.5781,
              4.9061
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          1.25,
          1.25,
          1.25,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.7517,
      "rotate": [
        0.0,
        0.0,
        -45.0
      ],
      "life": 0.8333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5359,
            "band": 0.05,
            "height": [
              0.3517,
              0.0
            ],
            "colors": [
              [
                0.0143,
                0.0037,
                0.0677,
                0.0
              ],
              [
                0.0143,
                0.0037,
                0.0677,
                0.0
              ],
              [
                0.0278,
                0.0278,
                0.4647,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Dark2/n7.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.5
            ],
            [
              0.0781,
              3.1591
            ],
            [
              0.1719,
              3.7795
            ],
            [
              0.2656,
              4.2373
            ],
            [
              0.375,
              4.5989
            ],
            [
              0.5781,
              4.9061
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          1.25,
          1.25,
          1.25,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.7533,
      "rotate": [
        0.0,
        0.0,
        45.0
      ],
      "life": 0.8333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5359,
            "band": 0.05,
            "height": [
              0.3517,
              0.0
            ],
            "colors": [
              [
                0.0143,
                0.0037,
                0.0677,
                0.0
              ],
              [
                0.0143,
                0.0037,
                0.0677,
                0.0
              ],
              [
                0.0278,
                0.0278,
                0.4647,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Dark2/n7.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.5
            ],
            [
              0.0781,
              3.1591
            ],
            [
              0.1719,
              3.7795
            ],
            [
              0.2656,
              4.2373
            ],
            [
              0.375,
              4.5989
            ],
            [
              0.5781,
              4.9061
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          1.25,
          1.25,
          1.25,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.755,
      "rotate": [
        0.0,
        0.0,
        135.0
      ],
      "life": 0.8333
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Dark2/n8-0.effect"
        },
        "width": [
          0.0,
          0.0
        ],
        "lifetime": 0.3333,
        "color": [
          0.1275,
          0.0278,
          0.586,
          1.0
        ]
      },
      "at": 0.1167,
      "offset": [
        0.0,
        0.0,
        2.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Dark2/n8-1.emitter"
        }
      },
      "at": 0.45,
      "offset": [
        0.0,
        0.0,
        2.0
      ],
      "rotate": [
        -90.0,
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
      "at": 0.6667,
      "life": 0.3
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.6667
    }
  ],
  "duration": 1.7,
  "events": [
    {
      "name": "impact",
      "time": 0.6667
    }
  ]
});
