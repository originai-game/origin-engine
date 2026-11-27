import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0 "火の玉中心": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.0.0.0.0 "火の尻尾": ring 350 instances: the first 4 drawn
// effekseer: node 0.0.0.0.1 "火の尻尾": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 1.0 "リング": ring 5 instances: the first 4 drawn
// effekseer: node 1.0 "リング": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 1.1 "中心": ring 50 instances: the first 4 drawn
// effekseer: node 1.2 "球体": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 1.3 "リング": ring 10 instances: the first 4 drawn
// effekseer: node 1.3 "リング": a ring whose thickness changes over its life: its middle thickness kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Fire6/n0-0-0.emitter"
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
        -90.0
      ],
      "velocity": [
        -36.0,
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
            "inner": 0.6,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Fire6/n0-0-0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.125
            ],
            [
              1.0,
              5.625
            ]
          ]
        },
        "color": [
          87.5,
          87.5,
          87.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        15.0,
        0.0,
        0.0
      ],
      "rotate": [
        -54.7356,
        30.0,
        80.2644
      ],
      "velocity": [
        -36.0,
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
            "inner": 0.6,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Fire6/n0-0-0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.125
            ],
            [
              1.0,
              5.625
            ]
          ]
        },
        "color": [
          87.5,
          87.5,
          87.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0017,
      "offset": [
        15.0,
        0.0,
        0.0
      ],
      "rotate": [
        -54.7356,
        -30.0,
        -80.2644
      ],
      "velocity": [
        -36.0,
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
            "inner": 0.6,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Fire6/n0-0-0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.125
            ],
            [
              1.0,
              5.625
            ]
          ]
        },
        "color": [
          87.5,
          87.5,
          87.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0033,
      "offset": [
        15.0,
        0.0,
        0.0
      ],
      "rotate": [
        54.7356,
        30.0,
        9.7356
      ],
      "velocity": [
        -36.0,
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
            "inner": 0.6,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Fire6/n0-0-0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.125
            ],
            [
              1.0,
              5.625
            ]
          ]
        },
        "color": [
          87.5,
          87.5,
          87.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.005,
      "offset": [
        15.0,
        0.0,
        0.0
      ],
      "rotate": [
        54.7356,
        -30.0,
        -9.7356
      ],
      "velocity": [
        -36.0,
        0.0,
        0.0
      ],
      "life": 0.6667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Fire6/n0-0-0-0-1.emitter"
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
        -90.0
      ],
      "velocity": [
        -36.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Fire6/n0-0-0-0-1-0.emitter"
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
        -90.0
      ],
      "velocity": [
        -36.0,
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
            "inner": 0.5923,
            "band": 1.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.9323,
                0.1573,
                0.0781,
                0.6667
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
          "$ogRef": "base/vfx/effekseer-tktk01/Fire6/n1-0.effect"
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
          1.25,
          1.25,
          1.25,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.6667,
      "offset": [
        -10.0,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        75.0,
        -22.5
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5923,
            "band": 1.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.9323,
                0.1573,
                0.0781,
                0.6667
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
          "$ogRef": "base/vfx/effekseer-tktk01/Fire6/n1-0.effect"
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
          1.25,
          1.25,
          1.25,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.6683,
      "offset": [
        -10.0,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        75.0,
        -7.5
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5923,
            "band": 1.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.9323,
                0.1573,
                0.0781,
                0.6667
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
          "$ogRef": "base/vfx/effekseer-tktk01/Fire6/n1-0.effect"
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
          1.25,
          1.25,
          1.25,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.67,
      "offset": [
        -10.0,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        75.0,
        7.5
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5923,
            "band": 1.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.9323,
                0.1573,
                0.0781,
                0.6667
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
          "$ogRef": "base/vfx/effekseer-tktk01/Fire6/n1-0.effect"
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
          1.25,
          1.25,
          1.25,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.6717,
      "offset": [
        -10.0,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        75.0,
        22.5
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Fire6/n1-0-0.emitter"
        }
      },
      "at": 0.6667,
      "offset": [
        -10.0,
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
            "inner": 0.0,
            "band": 0.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                0.1011,
                0.0494,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Fire6/n1-1.effect"
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
              3.0381
            ],
            [
              0.1562,
              3.8406
            ],
            [
              0.2344,
              4.4389
            ],
            [
              0.3281,
              4.9312
            ],
            [
              0.4688,
              5.3207
            ],
            [
              0.8281,
              5.5798
            ],
            [
              1.0,
              6.0
            ]
          ]
        },
        "color": [
          12.5,
          12.5,
          12.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.6667,
      "offset": [
        -10.0,
        0.0,
        0.0
      ],
      "rotate": [
        -54.7356,
        30.0,
        80.2644
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                0.1011,
                0.0494,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Fire6/n1-1.effect"
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
              3.0381
            ],
            [
              0.1562,
              3.8406
            ],
            [
              0.2344,
              4.4389
            ],
            [
              0.3281,
              4.9312
            ],
            [
              0.4688,
              5.3207
            ],
            [
              0.8281,
              5.5798
            ],
            [
              1.0,
              6.0
            ]
          ]
        },
        "color": [
          12.5,
          12.5,
          12.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.6668,
      "offset": [
        -10.0,
        0.0,
        0.0
      ],
      "rotate": [
        -54.7356,
        -30.0,
        -80.2644
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                0.1011,
                0.0494,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Fire6/n1-1.effect"
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
              3.0381
            ],
            [
              0.1562,
              3.8406
            ],
            [
              0.2344,
              4.4389
            ],
            [
              0.3281,
              4.9312
            ],
            [
              0.4688,
              5.3207
            ],
            [
              0.8281,
              5.5798
            ],
            [
              1.0,
              6.0
            ]
          ]
        },
        "color": [
          12.5,
          12.5,
          12.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.667,
      "offset": [
        -10.0,
        0.0,
        0.0
      ],
      "rotate": [
        54.7356,
        30.0,
        9.7356
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                0.1011,
                0.0494,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Fire6/n1-1.effect"
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
              3.0381
            ],
            [
              0.1562,
              3.8406
            ],
            [
              0.2344,
              4.4389
            ],
            [
              0.3281,
              4.9312
            ],
            [
              0.4688,
              5.3207
            ],
            [
              0.8281,
              5.5798
            ],
            [
              1.0,
              6.0
            ]
          ]
        },
        "color": [
          12.5,
          12.5,
          12.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.6672,
      "offset": [
        -10.0,
        0.0,
        0.0
      ],
      "rotate": [
        54.7356,
        -30.0,
        -9.7356
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Fire6/n1-2.emitter"
        }
      },
      "at": 0.6667,
      "offset": [
        -10.0,
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
            "inner": 0.4073,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Fire6/n1-3.effect"
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
              5.0381
            ],
            [
              0.1562,
              5.8406
            ],
            [
              0.2344,
              6.4389
            ],
            [
              0.3281,
              6.9312
            ],
            [
              0.4688,
              7.3207
            ],
            [
              0.8281,
              7.5798
            ],
            [
              1.0,
              8.0
            ]
          ]
        },
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.6667,
      "offset": [
        -10.0,
        0.0,
        0.0
      ],
      "rotate": [
        -54.7356,
        30.0,
        80.2644
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.4073,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Fire6/n1-3.effect"
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
              5.0381
            ],
            [
              0.1562,
              5.8406
            ],
            [
              0.2344,
              6.4389
            ],
            [
              0.3281,
              6.9312
            ],
            [
              0.4688,
              7.3207
            ],
            [
              0.8281,
              7.5798
            ],
            [
              1.0,
              8.0
            ]
          ]
        },
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.6683,
      "offset": [
        -10.0,
        0.0,
        0.0
      ],
      "rotate": [
        -54.7356,
        -30.0,
        -80.2644
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.4073,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Fire6/n1-3.effect"
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
              5.0381
            ],
            [
              0.1562,
              5.8406
            ],
            [
              0.2344,
              6.4389
            ],
            [
              0.3281,
              6.9312
            ],
            [
              0.4688,
              7.3207
            ],
            [
              0.8281,
              7.5798
            ],
            [
              1.0,
              8.0
            ]
          ]
        },
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.67,
      "offset": [
        -10.0,
        0.0,
        0.0
      ],
      "rotate": [
        54.7356,
        30.0,
        9.7356
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.4073,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Fire6/n1-3.effect"
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
              5.0381
            ],
            [
              0.1562,
              5.8406
            ],
            [
              0.2344,
              6.4389
            ],
            [
              0.3281,
              6.9312
            ],
            [
              0.4688,
              7.3207
            ],
            [
              0.8281,
              7.5798
            ],
            [
              1.0,
              8.0
            ]
          ]
        },
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.6717,
      "offset": [
        -10.0,
        0.0,
        0.0
      ],
      "rotate": [
        54.7356,
        -30.0,
        -9.7356
      ],
      "life": 0.5
    }
  ],
  "duration": 1.2508,
  "events": [
    {
      "name": "impact",
      "time": 0.6667
    }
  ]
});
