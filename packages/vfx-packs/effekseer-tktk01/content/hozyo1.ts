import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.1 "軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.1 "軌跡": nested generation: 100 instances above its generator folded into one emitter
// effekseer: node 0.0.0.2 "Glow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.2 "Glow": nested generation: 100 instances above its generator folded into one emitter
// effekseer: node 0.1.0.1 "軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.1.0.1 "軌跡": nested generation: 200 instances above its generator folded into one emitter
// effekseer: node 0.1.0.2 "軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.1.0.2 "軌跡": nested generation: 200 instances above its generator folded into one emitter
// effekseer: node 2 "地面リング": ring 5 instances: the first 4 drawn
// effekseer: node 2 "地面リング": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 3.0 "気の上昇": a fixed sprite turned at random: its mean plane drawn
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo1/n0-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        4.0,
        0.0,
        0.0
      ],
      "rotate": [
        90.0,
        0.0,
        90.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo1/n0-0-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        4.0,
        0.0,
        0.0
      ],
      "rotate": [
        90.0,
        0.0,
        90.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo1/n0-0-0-1.effect"
        },
        "width": [
          0.25,
          0.0625
        ],
        "lifetime": 0.4167,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          2.0,
          1.0
        ]
      },
      "at": 0.0,
      "offset": [
        2.0,
        0.0,
        0.0
      ],
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo1/n0-0-0-2.effect"
        },
        "width": [
          1.0,
          0.25
        ],
        "lifetime": 0.4167,
        "color": [
          0.047,
          0.047,
          0.047,
          1.0
        ],
        "orbit": [
          2.0,
          1.0
        ]
      },
      "at": 0.0,
      "offset": [
        2.0,
        0.0,
        0.0
      ],
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo1/n0-1-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        3.0,
        0.0,
        0.0
      ],
      "rotate": [
        90.0,
        0.0,
        90.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo1/n0-1-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        3.0,
        0.0,
        0.0
      ],
      "rotate": [
        90.0,
        0.0,
        90.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo1/n0-1-0-1.effect"
        },
        "width": [
          0.125,
          0.0312
        ],
        "lifetime": 0.4167,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 0.0,
      "offset": [
        3.0,
        0.0,
        0.0
      ],
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo1/n0-1-0-2.effect"
        },
        "width": [
          0.125,
          0.0312
        ],
        "lifetime": 0.4167,
        "color": [
          0.047,
          0.047,
          0.047,
          1.0
        ]
      },
      "at": 0.0,
      "offset": [
        3.0,
        0.0,
        0.0
      ],
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo1/n1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              15.0
            ],
            [
              1.0,
              0.0
            ]
          ]
        },
        "color": [
          1.0,
          0.0,
          0.0,
          0.1961
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "life": 0.8333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6978,
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
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo1/n2.effect"
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
              1.5273
            ],
            [
              0.1719,
              2.0236
            ],
            [
              0.2656,
              2.3899
            ],
            [
              0.375,
              2.6791
            ],
            [
              0.4688,
              2.8313
            ],
            [
              0.5781,
              2.9249
            ],
            [
              1.0,
              3.0
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
      "at": 0.4167,
      "rotate": [
        0.0,
        0.0,
        -135.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6978,
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
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo1/n2.effect"
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
              1.5273
            ],
            [
              0.1719,
              2.0236
            ],
            [
              0.2656,
              2.3899
            ],
            [
              0.375,
              2.6791
            ],
            [
              0.4688,
              2.8313
            ],
            [
              0.5781,
              2.9249
            ],
            [
              1.0,
              3.0
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
      "at": 0.4183,
      "rotate": [
        0.0,
        0.0,
        -45.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6978,
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
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo1/n2.effect"
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
              1.5273
            ],
            [
              0.1719,
              2.0236
            ],
            [
              0.2656,
              2.3899
            ],
            [
              0.375,
              2.6791
            ],
            [
              0.4688,
              2.8313
            ],
            [
              0.5781,
              2.9249
            ],
            [
              1.0,
              3.0
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
      "at": 0.42,
      "rotate": [
        0.0,
        0.0,
        45.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6978,
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
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo1/n2.effect"
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
              1.5273
            ],
            [
              0.1719,
              2.0236
            ],
            [
              0.2656,
              2.3899
            ],
            [
              0.375,
              2.6791
            ],
            [
              0.4688,
              2.8313
            ],
            [
              0.5781,
              2.9249
            ],
            [
              1.0,
              3.0
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
      "at": 0.4217,
      "rotate": [
        0.0,
        0.0,
        135.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo1/n3-0.emitter"
        }
      },
      "at": 0.4167,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    }
  ],
  "duration": 1.2912,
  "events": [
    {
      "name": "impact",
      "time": 0.4167
    }
  ]
});
