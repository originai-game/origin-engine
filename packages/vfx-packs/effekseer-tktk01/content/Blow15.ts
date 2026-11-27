import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 1.1 "外枠": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 1.1 "外枠": a UV region of a texture whose size is not known (not a PNG)
// effekseer: node 2.0.1 "外枠": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 2.0.1 "外枠": a UV region of a texture whose size is not known (not a PNG)
// effekseer: node 2.0.5 "地上リングギザ外": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 2.0.6 "地上リングギザ2外": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 2.1.1 "外枠": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 2.1.1 "外枠": a UV region of a texture whose size is not known (not a PNG)
// effekseer: node 2.1.5 "地上リングギザ外": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 2.1.6 "地上リングギザ2外": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 3.0 "割れ部分": ring 100 instances: the first 4 drawn
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow15/n0-0-0.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        0.0,
        0.0,
        1.0
      ],
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow15/n0-0-0-0.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        0.0,
        0.0,
        1.0
      ],
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow15/n0-0-0-1.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        0.0,
        0.0,
        1.0
      ],
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow15/n0-1-0.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        0.0,
        0.0,
        1.0
      ],
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow15/n0-1-0-0.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        0.0,
        0.0,
        1.0
      ],
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow15/n0-1-0-1.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        0.0,
        0.0,
        1.0
      ],
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow15/n0-2-0.emitter"
        }
      },
      "at": 0.0333,
      "offset": [
        0.0,
        0.0,
        2.0
      ],
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow15/n0-2-0-0.emitter"
        }
      },
      "at": 0.0333,
      "offset": [
        0.0,
        0.0,
        2.0
      ],
      "rotate": [
        90.0,
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
                0.502
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow15/n1-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0781,
              1.7909
            ],
            [
              0.1719,
              2.5354
            ],
            [
              0.2656,
              3.0848
            ],
            [
              0.375,
              3.5186
            ],
            [
              0.4688,
              3.7469
            ],
            [
              0.5781,
              3.8874
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
      "at": 0.05,
      "life": 0.5833
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9912,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow15/n1-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0781,
              1.7909
            ],
            [
              0.1719,
              2.5354
            ],
            [
              0.2656,
              3.0848
            ],
            [
              0.375,
              3.5186
            ],
            [
              0.4688,
              3.7469
            ],
            [
              0.5781,
              3.8874
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
      "at": 0.05,
      "life": 0.5833
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.6,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow15/n1-2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0781,
              1.7909
            ],
            [
              0.1719,
              2.5354
            ],
            [
              0.2656,
              3.0848
            ],
            [
              0.375,
              3.5186
            ],
            [
              0.4688,
              3.7469
            ],
            [
              0.5781,
              3.8874
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
      "at": 0.05,
      "life": 0.5833
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow15/n1-3.effect"
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
              6.6364
            ],
            [
              0.1719,
              9.1181
            ],
            [
              0.2656,
              10.9493
            ],
            [
              0.375,
              12.3954
            ],
            [
              0.4688,
              13.1564
            ],
            [
              0.5781,
              13.6246
            ],
            [
              1.0,
              14.0
            ]
          ]
        },
        "color": [
          1.0,
          0.0478,
          0.0478,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.05,
      "life": 0.5833
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow15/n2-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0781,
              1.7909
            ],
            [
              0.1719,
              2.5354
            ],
            [
              0.2656,
              3.0848
            ],
            [
              0.375,
              3.5186
            ],
            [
              0.4688,
              3.7469
            ],
            [
              0.5781,
              3.8874
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
      "at": 0.05,
      "offset": [
        -0.25,
        0.0,
        2.0
      ],
      "rotate": [
        0.0,
        30.0,
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
            "inner": 0.9912,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow15/n2-0-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0781,
              1.7909
            ],
            [
              0.1719,
              2.5354
            ],
            [
              0.2656,
              3.0848
            ],
            [
              0.375,
              3.5186
            ],
            [
              0.4688,
              3.7469
            ],
            [
              0.5781,
              3.8874
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
      "at": 0.05,
      "offset": [
        -0.25,
        0.0,
        2.0
      ],
      "rotate": [
        0.0,
        30.0,
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
            "inner": 0.0,
            "band": 0.6,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow15/n2-0-2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0781,
              1.7909
            ],
            [
              0.1719,
              2.5354
            ],
            [
              0.2656,
              3.0848
            ],
            [
              0.375,
              3.5186
            ],
            [
              0.4688,
              3.7469
            ],
            [
              0.5781,
              3.8874
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
      "at": 0.05,
      "offset": [
        -0.25,
        0.0,
        2.0
      ],
      "rotate": [
        0.0,
        30.0,
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
            "inner": 0.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow15/n2-0-3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0781,
              1.7909
            ],
            [
              0.1719,
              2.5354
            ],
            [
              0.2656,
              3.0848
            ],
            [
              0.375,
              3.5186
            ],
            [
              0.4688,
              3.7469
            ],
            [
              0.5781,
              3.8874
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
      "at": 0.05,
      "offset": [
        -0.25,
        0.0,
        2.0
      ],
      "rotate": [
        0.0,
        30.0,
        0.0
      ],
      "life": 0.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow15/n2-0-4.effect"
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
              6.6364
            ],
            [
              0.1719,
              9.1181
            ],
            [
              0.2656,
              10.9493
            ],
            [
              0.375,
              12.3954
            ],
            [
              0.4688,
              13.1564
            ],
            [
              0.5781,
              13.6246
            ],
            [
              1.0,
              14.0
            ]
          ]
        },
        "color": [
          1.0,
          0.0478,
          0.0478,
          0.502
        ],
        "blend": "additive"
      },
      "at": 0.05,
      "offset": [
        -0.25,
        0.0,
        2.0
      ],
      "rotate": [
        0.0,
        30.0,
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
            "inner": 2.0353,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow15/n2-0-5.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0781,
              1.7909
            ],
            [
              0.1719,
              2.5354
            ],
            [
              0.2656,
              3.0848
            ],
            [
              0.375,
              3.5186
            ],
            [
              0.4688,
              3.7469
            ],
            [
              0.5781,
              3.8874
            ],
            [
              1.0,
              4.0
            ]
          ]
        },
        "color": [
          2.0,
          2.0,
          2.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.05,
      "offset": [
        -0.25,
        0.0,
        2.0
      ],
      "rotate": [
        0.0,
        30.0,
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
            "inner": 1.5088,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow15/n2-0-6.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0781,
              1.7909
            ],
            [
              0.1719,
              2.5354
            ],
            [
              0.2656,
              3.0848
            ],
            [
              0.375,
              3.5186
            ],
            [
              0.4688,
              3.7469
            ],
            [
              0.5781,
              3.8874
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
      "at": 0.05,
      "offset": [
        -0.25,
        0.0,
        2.0
      ],
      "rotate": [
        0.0,
        30.0,
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
            "inner": 0.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow15/n2-1-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0781,
              1.7909
            ],
            [
              0.1719,
              2.5354
            ],
            [
              0.2656,
              3.0848
            ],
            [
              0.375,
              3.5186
            ],
            [
              0.4688,
              3.7469
            ],
            [
              0.5781,
              3.8874
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
      "at": 0.05,
      "offset": [
        0.25,
        0.0,
        2.0
      ],
      "rotate": [
        -180.0,
        30.0,
        -180.0
      ],
      "life": 0.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9912,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow15/n2-1-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0781,
              1.7909
            ],
            [
              0.1719,
              2.5354
            ],
            [
              0.2656,
              3.0848
            ],
            [
              0.375,
              3.5186
            ],
            [
              0.4688,
              3.7469
            ],
            [
              0.5781,
              3.8874
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
      "at": 0.05,
      "offset": [
        0.25,
        0.0,
        2.0
      ],
      "rotate": [
        -180.0,
        30.0,
        -180.0
      ],
      "life": 0.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.6,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow15/n2-1-2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0781,
              1.7909
            ],
            [
              0.1719,
              2.5354
            ],
            [
              0.2656,
              3.0848
            ],
            [
              0.375,
              3.5186
            ],
            [
              0.4688,
              3.7469
            ],
            [
              0.5781,
              3.8874
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
      "at": 0.05,
      "offset": [
        0.25,
        0.0,
        2.0
      ],
      "rotate": [
        -180.0,
        30.0,
        -180.0
      ],
      "life": 0.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow15/n2-1-3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0781,
              1.7909
            ],
            [
              0.1719,
              2.5354
            ],
            [
              0.2656,
              3.0848
            ],
            [
              0.375,
              3.5186
            ],
            [
              0.4688,
              3.7469
            ],
            [
              0.5781,
              3.8874
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
      "at": 0.05,
      "offset": [
        0.25,
        0.0,
        2.0
      ],
      "rotate": [
        -180.0,
        30.0,
        -180.0
      ],
      "life": 0.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow15/n2-1-4.effect"
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
              6.6364
            ],
            [
              0.1719,
              9.1181
            ],
            [
              0.2656,
              10.9493
            ],
            [
              0.375,
              12.3954
            ],
            [
              0.4688,
              13.1564
            ],
            [
              0.5781,
              13.6246
            ],
            [
              1.0,
              14.0
            ]
          ]
        },
        "color": [
          1.0,
          0.0478,
          0.0478,
          0.502
        ],
        "blend": "additive"
      },
      "at": 0.05,
      "offset": [
        0.25,
        0.0,
        2.0
      ],
      "rotate": [
        -180.0,
        30.0,
        -180.0
      ],
      "life": 0.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 2.0353,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow15/n2-1-5.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0781,
              1.7909
            ],
            [
              0.1719,
              2.5354
            ],
            [
              0.2656,
              3.0848
            ],
            [
              0.375,
              3.5186
            ],
            [
              0.4688,
              3.7469
            ],
            [
              0.5781,
              3.8874
            ],
            [
              1.0,
              4.0
            ]
          ]
        },
        "color": [
          2.0,
          2.0,
          2.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.05,
      "offset": [
        0.25,
        0.0,
        2.0
      ],
      "rotate": [
        -180.0,
        30.0,
        -180.0
      ],
      "life": 0.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.5088,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow15/n2-1-6.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0781,
              1.7909
            ],
            [
              0.1719,
              2.5354
            ],
            [
              0.2656,
              3.0848
            ],
            [
              0.375,
              3.5186
            ],
            [
              0.4688,
              3.7469
            ],
            [
              0.5781,
              3.8874
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
      "at": 0.05,
      "offset": [
        0.25,
        0.0,
        2.0
      ],
      "rotate": [
        -180.0,
        30.0,
        -180.0
      ],
      "life": 0.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.0,
            "start": 265.0,
            "length": 10.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow15/n3-0.effect"
        },
        "scale": 2.0,
        "spin": [
          0.0,
          0.0,
          270.0
        ],
        "color": [
          100.0,
          100.0,
          100.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0667,
      "velocity": [
        7.5,
        0.0,
        0.0
      ],
      "life": 0.6667
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          0.4374,
          0.4374
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
      "at": 0.05,
      "life": 0.3,
      "offset": [
        0.0,
        0.0,
        1.2659
      ]
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.05
    }
  ],
  "duration": 0.9079,
  "events": [
    {
      "name": "impact",
      "time": 0.05
    }
  ]
});
