import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0 "長針": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 0.0.1.0 "短針": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 1.0 "針": location over an f-curve: straightened
// effekseer: node 1.0 "針": motion out from a circle: approximated along one direction
// effekseer: node 1.0.0 "glow": location over an f-curve: straightened
// effekseer: node 1.0.0 "glow": motion out from a circle: approximated along one direction
// effekseer: node 3.0.0 "リング": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 3.0.1 "リング柱": a ring whose height changes over its life: its middle height kept
// effekseer: node 3.1.0 "リング": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 3.1.1 "リング柱": a ring whose height changes over its life: its middle height kept
// effekseer: node 3.1.1 "リング柱": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 3.1.2 "リング外枠": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 3.2.0.2 "軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 3.2.0.2 "軌跡": nested generation: 500 instances above its generator folded into one emitter
// effekseer: node 3.2.0.3 "軌跡glow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 3.2.0.3 "軌跡glow": nested generation: 500 instances above its generator folded into one emitter
// effekseer: node 3.3.0.2 "軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 3.3.0.2 "軌跡": nested generation: 250 instances above its generator folded into one emitter
// effekseer: node 3.3.0.3 "軌跡glow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 3.3.0.3 "軌跡glow": nested generation: 250 instances above its generator folded into one emitter
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Magic2/n0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              5.0
            ],
            [
              0.5625,
              5.0
            ],
            [
              0.7656,
              9.2748
            ],
            [
              1.0,
              9.25
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
      "life": 1.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Magic2/n0-0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.7454
            ],
            [
              0.3906,
              0.7454
            ],
            [
              0.4062,
              0.7336
            ],
            [
              0.4688,
              0.6038
            ],
            [
              0.5469,
              0.3841
            ],
            [
              0.5781,
              0.2465
            ],
            [
              0.6094,
              0.0
            ],
            [
              1.0,
              0.0
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
      "offset": [
        0.0,
        0.6667,
        0.0
      ],
      "life": 1.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Magic2/n0-0-1-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.6455
            ],
            [
              0.3906,
              0.6455
            ],
            [
              0.4062,
              0.6353
            ],
            [
              0.4688,
              0.5229
            ],
            [
              0.5469,
              0.3327
            ],
            [
              0.5781,
              0.2135
            ],
            [
              0.6094,
              0.0
            ],
            [
              1.0,
              0.0
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
      "offset": [
        0.0,
        -0.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        180.0
      ],
      "life": 1.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Magic2/n0-0-2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              4.1667
            ],
            [
              0.5938,
              3.3594
            ],
            [
              0.9375,
              4.9826
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          1.0,
          0.0104,
          0.2195,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "life": 1.4167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Magic2/n1-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        4.0,
        0.0,
        4.0
      ],
      "rotate": [
        -54.7356,
        -30.0,
        54.7356
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Magic2/n1-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        4.0,
        0.0,
        4.0
      ],
      "rotate": [
        -54.7356,
        -30.0,
        54.7356
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Magic2/n2-0.emitter"
        }
      },
      "at": 0.6667,
      "offset": [
        0.0,
        0.0,
        5.0
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Magic2/n2-0-0.emitter"
        }
      },
      "at": 0.6667,
      "offset": [
        0.0,
        0.0,
        5.0
      ],
      "rotate": [
        90.0,
        89.972,
        0.0
      ],
      "velocity": [
        0.0,
        0.0,
        -13.5
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.2582,
            "band": 1.0,
            "colors": [
              [
                1.0,
                0.0104,
                0.2195,
                0.0
              ],
              [
                1.0,
                0.0478,
                0.2195,
                0.0
              ],
              [
                1.0,
                0.0478,
                0.2195,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Magic2/n3-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.1
            ],
            [
              0.0781,
              2.6009
            ],
            [
              0.1719,
              3.0724
            ],
            [
              0.2656,
              3.4204
            ],
            [
              0.375,
              3.6951
            ],
            [
              0.5781,
              3.9287
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
      "at": 0.8,
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
              1.4455
            ],
            "colors": [
              [
                1.0,
                0.0104,
                0.2195,
                0.0
              ],
              [
                1.0,
                0.0478,
                0.2195,
                0.0
              ],
              [
                1.0,
                0.0478,
                0.2195,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Magic2/n3-0-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.1
            ],
            [
              0.0781,
              2.6009
            ],
            [
              0.1719,
              3.0724
            ],
            [
              0.2656,
              3.4204
            ],
            [
              0.375,
              3.6951
            ],
            [
              0.5781,
              3.9287
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
      "at": 0.8,
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9725,
            "band": 0.5,
            "colors": [
              [
                1.0,
                0.0104,
                0.2195,
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
                0.0478,
                0.2195,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Magic2/n3-0-2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.2
            ],
            [
              0.0781,
              2.6745
            ],
            [
              0.1719,
              3.1213
            ],
            [
              0.2656,
              3.4509
            ],
            [
              0.375,
              3.7112
            ],
            [
              0.5781,
              3.9324
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
      "at": 0.8,
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.2587,
            "band": 1.0,
            "colors": [
              [
                1.0,
                0.0104,
                0.2195,
                0.0
              ],
              [
                1.0,
                0.0478,
                0.2195,
                0.0
              ],
              [
                1.0,
                0.0478,
                0.2195,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Magic2/n3-1-0.effect"
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
          2.0,
          2.0,
          2.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.6333,
      "life": 0.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9827,
            "band": 1.0,
            "height": [
              0.0,
              0.9653
            ],
            "colors": [
              [
                1.0,
                0.0104,
                0.2195,
                0.0
              ],
              [
                1.0,
                0.0478,
                0.2195,
                0.0
              ],
              [
                1.0,
                0.0478,
                0.2195,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Magic2/n3-1-1.effect"
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
          2.0,
          2.0,
          2.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.6333,
      "life": 0.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9585,
            "band": 0.5,
            "colors": [
              [
                1.0,
                0.0104,
                0.2195,
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
                0.0478,
                0.2195,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Magic2/n3-1-2.effect"
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
          2.0,
          2.0,
          2.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.6333,
      "life": 0.6667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Magic2/n3-2-0.emitter"
        }
      },
      "at": 0.6333,
      "offset": [
        0.0,
        0.0,
        1.125
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
          "$ogRef": "base/vfx/effekseer-tktk01/Magic2/n3-2-0-0.emitter"
        }
      },
      "at": 0.6333,
      "offset": [
        0.0,
        0.0,
        1.125
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
          "$ogRef": "base/vfx/effekseer-tktk01/Magic2/n3-2-0-1.emitter"
        }
      },
      "at": 0.6333,
      "offset": [
        0.0,
        0.0,
        1.125
      ],
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Magic2/n3-2-0-2.effect"
        },
        "width": [
          0.1,
          0.025
        ],
        "lifetime": 0.0833,
        "color": [
          1.0,
          1.0,
          1.0,
          0.3922
        ]
      },
      "at": 0.6333,
      "offset": [
        0.0,
        0.0,
        1.125
      ],
      "life": 0.0833
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Magic2/n3-2-0-3.effect"
        },
        "width": [
          0.2,
          0.05
        ],
        "lifetime": 0.0833,
        "color": [
          1.0,
          0.0104,
          0.0478,
          0.3922
        ]
      },
      "at": 0.6333,
      "offset": [
        0.0,
        0.0,
        1.125
      ],
      "life": 0.0833
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Magic2/n3-3-0.emitter"
        }
      },
      "at": 0.8,
      "offset": [
        0.0,
        0.0,
        1.125
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
          "$ogRef": "base/vfx/effekseer-tktk01/Magic2/n3-3-0-0.emitter"
        }
      },
      "at": 0.8,
      "offset": [
        0.0,
        0.0,
        1.125
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
          "$ogRef": "base/vfx/effekseer-tktk01/Magic2/n3-3-0-1.emitter"
        }
      },
      "at": 0.8,
      "offset": [
        0.0,
        0.0,
        1.125
      ],
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Magic2/n3-3-0-2.effect"
        },
        "width": [
          0.1,
          0.025
        ],
        "lifetime": 0.0833,
        "color": [
          1.0,
          1.0,
          1.0,
          0.3922
        ]
      },
      "at": 0.8,
      "offset": [
        0.0,
        0.0,
        1.125
      ],
      "life": 0.0833
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Magic2/n3-3-0-3.effect"
        },
        "width": [
          0.2,
          0.05
        ],
        "lifetime": 0.0833,
        "color": [
          1.0,
          0.0104,
          0.0478,
          0.3922
        ]
      },
      "at": 0.8,
      "offset": [
        0.0,
        0.0,
        1.125
      ],
      "life": 0.0833
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          0.5497,
          0.5667
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
      "at": 0.8,
      "life": 0.3,
      "offset": [
        0.0,
        0.0,
        0.5359
      ]
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.8
    }
  ],
  "duration": 2.4708,
  "events": [
    {
      "name": "impact",
      "time": 0.8
    }
  ]
});
