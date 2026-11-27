import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.1 "Distortion": motion out from a circle: approximated along one direction
// effekseer: node 0.0.4.0.0 "RIng": ring 10 instances: the first 4 drawn
// effekseer: node 0.0.4.0.0 "RIng": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.1.1 "Distortion": motion out from a circle: approximated along one direction
// effekseer: node 0.1.3.0.0.0.0 "LineMain": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.1.3.0.0.0.0 "LineMain": nested generation: 100 instances above its generator folded into one emitter
// effekseer: node 0.1.3.0.0.0.1.0 "LineColor": nested generation: 100 instances above its generator folded into one emitter
// effekseer: node 0.1.3.0.0.0.1.1.0 "LineColor": nested generation: 100 instances above its generator folded into one emitter
// effekseer: node 0.1.4 "Distortion": motion out from a circle: approximated along one direction
// effekseer: node 0.1.5.0.0 "ｌｅｎｓ": a ring's arc over its life
// effekseer: node 0.1.5.0.0 "ｌｅｎｓ": ring 100 instances: the first 4 drawn
// effekseer: node 0.1.6.0.0 "RIng": ring 5 instances: the first 4 drawn
// effekseer: node 0.1.6.0.0 "RIng": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.1.7.0.0 "WindRing": ring 30 instances: the first 4 drawn
// effekseer: node 0.1.7.0.0 "WindRing": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 1.0.0.0.0.2.0 "LineMain": nested generation: 2 instances above its generator folded into one emitter
// effekseer: node 1.0.0.0.0.2.1.0 "LineGlow": nested generation: 2 instances above its generator folded into one emitter
// effekseer: node 1.0.0.0.1 "Wind": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.0.0.0.1 "Wind": ring unlimited instances: the first 4 drawn
// effekseer: node 1.0.0.0.1 "Wind": nested generation: 2 instances above its generator folded into one emitter
// effekseer: node 1.0.0.0.1 "Wind": a ring whose height changes over its life: its middle height kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n0-0-0-0-0-0.emitter"
        }
      },
      "at": 0.1,
      "offset": [
        0.0,
        0.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n0-0-0-0-0-1.emitter"
        }
      },
      "at": 0.1,
      "offset": [
        0.0,
        0.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n0-0-0-0-0-1-0.emitter"
        }
      },
      "at": 0.1,
      "offset": [
        0.0,
        0.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n0-0-1.emitter"
        }
      },
      "at": 0.1017
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n0-0-2-0-0-0.emitter"
        }
      },
      "at": 0.1,
      "offset": [
        0.0,
        2.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n0-0-2-0-0-1.emitter"
        }
      },
      "at": 0.1,
      "offset": [
        0.0,
        2.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n0-0-2-0-0-1-0.emitter"
        }
      },
      "at": 0.1,
      "offset": [
        0.0,
        2.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n0-0-3.emitter"
        }
      },
      "at": 0.1
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9336,
            "band": 0.9375,
            "colors": [
              [
                1.0,
                0.139,
                0.0415,
                0.0
              ],
              [
                1.0,
                0.3192,
                0.1383,
                1.0
              ],
              [
                1.0,
                0.139,
                0.0415,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n0-0-4-0-0.effect"
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
              3.1864
            ],
            [
              0.1719,
              4.3032
            ],
            [
              0.2656,
              5.1272
            ],
            [
              0.375,
              5.7779
            ],
            [
              0.4688,
              6.1204
            ],
            [
              0.5781,
              6.3311
            ],
            [
              1.0,
              6.5
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
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9336,
            "band": 0.9375,
            "colors": [
              [
                1.0,
                0.139,
                0.0415,
                0.0
              ],
              [
                1.0,
                0.3192,
                0.1383,
                1.0
              ],
              [
                1.0,
                0.139,
                0.0415,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n0-0-4-0-0.effect"
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
              3.1864
            ],
            [
              0.1719,
              4.3032
            ],
            [
              0.2656,
              5.1272
            ],
            [
              0.375,
              5.7779
            ],
            [
              0.4688,
              6.1204
            ],
            [
              0.5781,
              6.3311
            ],
            [
              1.0,
              6.5
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
      "at": 0.1167,
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9336,
            "band": 0.9375,
            "colors": [
              [
                1.0,
                0.139,
                0.0415,
                0.0
              ],
              [
                1.0,
                0.3192,
                0.1383,
                1.0
              ],
              [
                1.0,
                0.139,
                0.0415,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n0-0-4-0-0.effect"
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
              3.1864
            ],
            [
              0.1719,
              4.3032
            ],
            [
              0.2656,
              5.1272
            ],
            [
              0.375,
              5.7779
            ],
            [
              0.4688,
              6.1204
            ],
            [
              0.5781,
              6.3311
            ],
            [
              1.0,
              6.5
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
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9336,
            "band": 0.9375,
            "colors": [
              [
                1.0,
                0.139,
                0.0415,
                0.0
              ],
              [
                1.0,
                0.3192,
                0.1383,
                1.0
              ],
              [
                1.0,
                0.139,
                0.0415,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n0-0-4-0-0.effect"
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
              3.1864
            ],
            [
              0.1719,
              4.3032
            ],
            [
              0.2656,
              5.1272
            ],
            [
              0.375,
              5.7779
            ],
            [
              0.4688,
              6.1204
            ],
            [
              0.5781,
              6.3311
            ],
            [
              1.0,
              6.5
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
      "at": 0.15,
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n0-1-0-0-0-0.emitter"
        }
      },
      "at": 0.7,
      "offset": [
        0.0,
        0.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n0-1-0-0-0-1.emitter"
        }
      },
      "at": 0.7,
      "offset": [
        0.0,
        0.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n0-1-0-0-0-1-0.emitter"
        }
      },
      "at": 0.7,
      "offset": [
        0.0,
        0.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n0-1-1.emitter"
        }
      },
      "at": 0.7017
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n0-1-2.emitter"
        }
      },
      "at": 0.7
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n0-1-3-0-0-0-0.effect"
        },
        "width": [
          1.25,
          1.25
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 0.7,
      "offset": [
        0.0,
        0.5,
        0.0
      ],
      "velocity": [
        0.0,
        -3.6,
        0.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n0-1-3-0-0-0-1-0.effect"
        },
        "width": [
          5.0,
          5.0
        ],
        "lifetime": 0.3333,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 0.7,
      "offset": [
        0.0,
        0.5,
        0.0
      ],
      "velocity": [
        0.0,
        4.9714,
        0.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n0-1-3-0-0-0-1-1-0.effect"
        },
        "width": [
          12.5,
          12.5
        ],
        "lifetime": 0.4167,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 0.7,
      "offset": [
        0.0,
        0.5,
        0.0
      ],
      "velocity": [
        0.0,
        4.9714,
        0.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n0-1-4.emitter"
        }
      },
      "at": 0.7017
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.0,
            "start": 268.0,
            "length": 4.0,
            "colors": [
              [
                1.0,
                0.2026,
                0.0556,
                0.0
              ],
              [
                1.0,
                0.5906,
                0.3794,
                1.0
              ],
              [
                1.0,
                0.2026,
                0.0556,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n0-1-5-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.1094,
              2.8118
            ],
            [
              0.2188,
              3.496
            ],
            [
              0.3281,
              4.0648
            ],
            [
              0.4531,
              4.5891
            ],
            [
              0.7031,
              5.3056
            ],
            [
              1.0,
              5.75
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          102.8571
        ],
        "color": [
          100.0,
          100.0,
          100.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.7,
      "life": 0.5833
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9833,
            "band": 0.9375,
            "colors": [
              [
                1.0,
                0.2195,
                0.0619,
                0.0
              ],
              [
                1.0,
                0.6354,
                0.4396,
                1.0
              ],
              [
                1.0,
                0.2195,
                0.0619,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n0-1-6-0-0.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.0781,
              3.0546
            ],
            [
              0.1719,
              4.0472
            ],
            [
              0.2656,
              4.7797
            ],
            [
              0.375,
              5.3582
            ],
            [
              0.4688,
              5.6625
            ],
            [
              0.5781,
              5.8498
            ],
            [
              1.0,
              6.0
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
      "at": 0.7,
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9833,
            "band": 0.9375,
            "colors": [
              [
                1.0,
                0.2195,
                0.0619,
                0.0
              ],
              [
                1.0,
                0.6354,
                0.4396,
                1.0
              ],
              [
                1.0,
                0.2195,
                0.0619,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n0-1-6-0-0.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.0781,
              3.0546
            ],
            [
              0.1719,
              4.0472
            ],
            [
              0.2656,
              4.7797
            ],
            [
              0.375,
              5.3582
            ],
            [
              0.4688,
              5.6625
            ],
            [
              0.5781,
              5.8498
            ],
            [
              1.0,
              6.0
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
      "at": 0.7167,
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9833,
            "band": 0.9375,
            "colors": [
              [
                1.0,
                0.2195,
                0.0619,
                0.0
              ],
              [
                1.0,
                0.6354,
                0.4396,
                1.0
              ],
              [
                1.0,
                0.2195,
                0.0619,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n0-1-6-0-0.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.0781,
              3.0546
            ],
            [
              0.1719,
              4.0472
            ],
            [
              0.2656,
              4.7797
            ],
            [
              0.375,
              5.3582
            ],
            [
              0.4688,
              5.6625
            ],
            [
              0.5781,
              5.8498
            ],
            [
              1.0,
              6.0
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
      "at": 0.7333,
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9833,
            "band": 0.9375,
            "colors": [
              [
                1.0,
                0.2195,
                0.0619,
                0.0
              ],
              [
                1.0,
                0.6354,
                0.4396,
                1.0
              ],
              [
                1.0,
                0.2195,
                0.0619,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n0-1-6-0-0.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.0781,
              3.0546
            ],
            [
              0.1719,
              4.0472
            ],
            [
              0.2656,
              4.7797
            ],
            [
              0.375,
              5.3582
            ],
            [
              0.4688,
              5.6625
            ],
            [
              0.5781,
              5.8498
            ],
            [
              1.0,
              6.0
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
      "at": 0.75,
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9153,
            "band": 0.5,
            "start": 165.0,
            "length": 210.0,
            "colors": [
              [
                1.0,
                0.2195,
                0.0619,
                0.0
              ],
              [
                1.0,
                0.4863,
                0.2186,
                1.0
              ],
              [
                1.0,
                0.2195,
                0.0619,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n0-1-7-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.4219
            ],
            [
              0.0938,
              1.6035
            ],
            [
              0.2031,
              1.7725
            ],
            [
              0.4219,
              2.0059
            ],
            [
              0.6719,
              2.1569
            ],
            [
              1.0,
              2.25
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          360.0
        ],
        "color": [
          30.0,
          30.0,
          30.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.8333,
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n1-0-0-0-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.0,
        12.5
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
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n1-0-0-0-0-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.0,
        12.5
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n1-0-0-0-0-2-0.effect"
        },
        "width": [
          1.3333,
          1.3333
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
        0.0,
        -2.1706,
        12.3101
      ],
      "rotate": [
        10.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        26.0472,
        -147.7212
      ],
      "life": 0.1667
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n1-0-0-0-0-2-1-0.effect"
        },
        "width": [
          6.6667,
          6.6667
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
        0.0,
        -2.1706,
        12.3101
      ],
      "rotate": [
        10.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        26.0472,
        -147.7212
      ],
      "life": 0.1667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.5,
            "height": [
              2.2292,
              0.0
            ],
            "colors": [
              [
                1.0,
                0.2618,
                0.0781,
                0.0
              ],
              [
                1.0,
                0.5668,
                0.283,
                0.7516
              ],
              [
                1.0,
                0.2618,
                0.0781,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n1-0-0-0-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              3.0
            ],
            [
              0.1094,
              6.247
            ],
            [
              0.2188,
              8.9839
            ],
            [
              0.3281,
              11.2591
            ],
            [
              0.4531,
              13.3564
            ],
            [
              0.5781,
              14.9866
            ],
            [
              0.7031,
              16.2224
            ],
            [
              1.0,
              18.0
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
      "at": 0.0167,
      "offset": [
        0.0,
        -2.1706,
        12.3101
      ],
      "rotate": [
        10.0,
        0.0,
        -135.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.5,
            "height": [
              2.2292,
              0.0
            ],
            "colors": [
              [
                1.0,
                0.2618,
                0.0781,
                0.0
              ],
              [
                1.0,
                0.5668,
                0.283,
                0.7516
              ],
              [
                1.0,
                0.2618,
                0.0781,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n1-0-0-0-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              3.0
            ],
            [
              0.1094,
              6.247
            ],
            [
              0.2188,
              8.9839
            ],
            [
              0.3281,
              11.2591
            ],
            [
              0.4531,
              13.3564
            ],
            [
              0.5781,
              14.9866
            ],
            [
              0.7031,
              16.2224
            ],
            [
              1.0,
              18.0
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
      "at": 0.0183,
      "offset": [
        0.0,
        -2.1706,
        12.3101
      ],
      "rotate": [
        10.0,
        0.0,
        -45.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.5,
            "height": [
              2.2292,
              0.0
            ],
            "colors": [
              [
                1.0,
                0.2618,
                0.0781,
                0.0
              ],
              [
                1.0,
                0.5668,
                0.283,
                0.7516
              ],
              [
                1.0,
                0.2618,
                0.0781,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n1-0-0-0-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              3.0
            ],
            [
              0.1094,
              6.247
            ],
            [
              0.2188,
              8.9839
            ],
            [
              0.3281,
              11.2591
            ],
            [
              0.4531,
              13.3564
            ],
            [
              0.5781,
              14.9866
            ],
            [
              0.7031,
              16.2224
            ],
            [
              1.0,
              18.0
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
      "at": 0.02,
      "offset": [
        0.0,
        -2.1706,
        12.3101
      ],
      "rotate": [
        10.0,
        0.0,
        45.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.5,
            "height": [
              2.2292,
              0.0
            ],
            "colors": [
              [
                1.0,
                0.2618,
                0.0781,
                0.0
              ],
              [
                1.0,
                0.5668,
                0.283,
                0.7516
              ],
              [
                1.0,
                0.2618,
                0.0781,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun1/n1-0-0-0-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              3.0
            ],
            [
              0.1094,
              6.247
            ],
            [
              0.2188,
              8.9839
            ],
            [
              0.3281,
              11.2591
            ],
            [
              0.4531,
              13.3564
            ],
            [
              0.5781,
              14.9866
            ],
            [
              0.7031,
              16.2224
            ],
            [
              1.0,
              18.0
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
      "at": 0.0217,
      "offset": [
        0.0,
        -2.1706,
        12.3101
      ],
      "rotate": [
        10.0,
        0.0,
        135.0
      ],
      "life": 0.25
    }
  ],
  "duration": 2.036,
  "events": [
    {
      "name": "impact",
      "time": 0.7
    }
  ]
});
