import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0 "LineColor": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.1 "LineGlow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.2.0.0 "particleMain": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.2.0.1 "particleColor": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.2.0.2 "particleGlow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.3.0.0 "Ring": ring 50 instances: the first 4 drawn
// effekseer: node 0.0.0.3.0.0 "Ring": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.0.0.3.1 "Ring_ｈｅａｄ": ring 30 instances: the first 4 drawn
// effekseer: node 0.0.0.3.1 "Ring_ｈｅａｄ": a ring whose height changes over its life: its middle height kept
// effekseer: node 1.0.0.0 "giza_uti": ring 10 instances: the first 4 drawn
// effekseer: node 1.0.0.0 "giza_uti": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 1.0.0.1 "giza_soto": ring 10 instances: the first 4 drawn
// effekseer: node 1.0.0.1 "giza_soto": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 1.0.0.2 "ghiza_Pillar": ring 10 instances: the first 4 drawn
// effekseer: node 1.0.0.2 "ghiza_Pillar": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 1.2.0.0.0.0.0.0 "LineColor": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.2.0.0.0.0.0.0 "LineColor": nested generation: 200 instances above its generator folded into one emitter
// effekseer: node 1.2.0.1 "Ring": ring 40 instances: the first 4 drawn
// effekseer: node 1.2.0.1 "Ring": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 2.0.0.0.0.0.0 "LineMain": a moving ancestor: its descendants are placed where it starts
// effekseer: node 2.0.0.0.0.0.0 "LineMain": nested generation: 100 instances above its generator folded into one emitter
// effekseer: node 2.0.0.0.0.0.1 "LineColor": a moving ancestor: its descendants are placed where it starts
// effekseer: node 2.0.0.0.0.0.1 "LineColor": nested generation: 100 instances above its generator folded into one emitter
// effekseer: node 2.0.1.0.0.0.0.0 "LineColor": a moving ancestor: its descendants are placed where it starts
// effekseer: node 2.0.1.0.0.0.0.0 "LineColor": nested generation: 200 instances above its generator folded into one emitter
// effekseer: node 2.1.1 "Ring": ring 10 instances: the first 4 drawn
// effekseer: node 2.1.1 "Ring": a ring whose thickness changes over its life: its middle thickness kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n0-0-0-0.effect"
        },
        "width": [
          2.0,
          2.0
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          0.5356,
          0.0478,
          1.0
        ]
      },
      "at": 0.0,
      "offset": [
        0.0,
        10.0,
        0.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n0-0-0-1.effect"
        },
        "width": [
          5.0,
          5.0
        ],
        "lifetime": 0.3333,
        "color": [
          1.0,
          0.2195,
          0.0104,
          0.251
        ]
      },
      "at": 0.0,
      "offset": [
        0.0,
        10.0,
        0.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n0-0-0-2-0-0.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        0.0,
        10.25,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n0-0-0-2-0-1.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        0.0,
        10.25,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n0-0-0-2-0-2.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        0.0,
        10.25,
        0.0
      ]
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
                0.0478,
                0.0
              ],
              [
                1.0,
                0.458,
                0.269,
                0.3514
              ],
              [
                1.0,
                0.2195,
                0.0478,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n0-0-0-3-0-0.effect"
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
          432.0
        ],
        "color": [
          50.0,
          50.0,
          50.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        0.0,
        10.0,
        0.0
      ],
      "velocity": [
        0.0,
        -30.75,
        0.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.1905,
            "band": 0.5,
            "height": [
              -1.8106,
              -0.0631
            ],
            "colors": [
              [
                1.0,
                0.1636,
                0.0356,
                0.0
              ],
              [
                1.0,
                0.7464,
                0.6337,
                0.8755
              ],
              [
                1.0,
                0.1636,
                0.0356,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n0-0-0-3-1.effect"
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
              2.6494
            ],
            [
              0.2188,
              3.1968
            ],
            [
              0.4531,
              4.0713
            ],
            [
              0.7031,
              4.6445
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          7.5,
          7.5,
          7.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0333,
      "offset": [
        0.0,
        6.975,
        0.0
      ],
      "rotate": [
        90.0,
        0.0,
        -135.0
      ],
      "velocity": [
        0.0,
        -30.75,
        0.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.1905,
            "band": 0.5,
            "height": [
              -1.8106,
              -0.0631
            ],
            "colors": [
              [
                1.0,
                0.1636,
                0.0356,
                0.0
              ],
              [
                1.0,
                0.7464,
                0.6337,
                0.8755
              ],
              [
                1.0,
                0.1636,
                0.0356,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n0-0-0-3-1.effect"
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
              2.6494
            ],
            [
              0.2188,
              3.1968
            ],
            [
              0.4531,
              4.0713
            ],
            [
              0.7031,
              4.6445
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          7.5,
          7.5,
          7.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0375,
      "offset": [
        0.0,
        6.975,
        0.0
      ],
      "rotate": [
        90.0,
        0.0,
        -45.0
      ],
      "velocity": [
        0.0,
        -30.75,
        0.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.1905,
            "band": 0.5,
            "height": [
              -1.8106,
              -0.0631
            ],
            "colors": [
              [
                1.0,
                0.1636,
                0.0356,
                0.0
              ],
              [
                1.0,
                0.7464,
                0.6337,
                0.8755
              ],
              [
                1.0,
                0.1636,
                0.0356,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n0-0-0-3-1.effect"
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
              2.6494
            ],
            [
              0.2188,
              3.1968
            ],
            [
              0.4531,
              4.0713
            ],
            [
              0.7031,
              4.6445
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          7.5,
          7.5,
          7.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0417,
      "offset": [
        0.0,
        6.975,
        0.0
      ],
      "rotate": [
        90.0,
        0.0,
        45.0
      ],
      "velocity": [
        0.0,
        -30.75,
        0.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.1905,
            "band": 0.5,
            "height": [
              -1.8106,
              -0.0631
            ],
            "colors": [
              [
                1.0,
                0.1636,
                0.0356,
                0.0
              ],
              [
                1.0,
                0.7464,
                0.6337,
                0.8755
              ],
              [
                1.0,
                0.1636,
                0.0356,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n0-0-0-3-1.effect"
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
              2.6494
            ],
            [
              0.2188,
              3.1968
            ],
            [
              0.4531,
              4.0713
            ],
            [
              0.7031,
              4.6445
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          7.5,
          7.5,
          7.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0458,
      "offset": [
        0.0,
        6.975,
        0.0
      ],
      "rotate": [
        90.0,
        0.0,
        135.0
      ],
      "velocity": [
        0.0,
        -30.75,
        0.0
      ],
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
                0.1535,
                0.0334,
                0.0
              ],
              [
                1.0,
                0.4248,
                0.1535,
                1.0
              ],
              [
                1.0,
                0.1535,
                0.0334,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n1-0-0-0.effect"
        },
        "scale": 1.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.2,
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
                0.1535,
                0.0334,
                0.0
              ],
              [
                1.0,
                0.4248,
                0.1535,
                1.0
              ],
              [
                1.0,
                0.1535,
                0.0334,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n1-0-0-0.effect"
        },
        "scale": 1.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.2167,
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
                0.1535,
                0.0334,
                0.0
              ],
              [
                1.0,
                0.4248,
                0.1535,
                1.0
              ],
              [
                1.0,
                0.1535,
                0.0334,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n1-0-0-0.effect"
        },
        "scale": 1.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.2333,
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
                0.1535,
                0.0334,
                0.0
              ],
              [
                1.0,
                0.4248,
                0.1535,
                1.0
              ],
              [
                1.0,
                0.1535,
                0.0334,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n1-0-0-0.effect"
        },
        "scale": 1.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.25,
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.067,
            "band": 1.0,
            "colors": [
              [
                1.0,
                0.1535,
                0.0334,
                0.0
              ],
              [
                1.0,
                0.4248,
                0.1535,
                1.0
              ],
              [
                1.0,
                0.1535,
                0.0334,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n1-0-0-1.effect"
        },
        "scale": 1.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.2,
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.067,
            "band": 1.0,
            "colors": [
              [
                1.0,
                0.1535,
                0.0334,
                0.0
              ],
              [
                1.0,
                0.4248,
                0.1535,
                1.0
              ],
              [
                1.0,
                0.1535,
                0.0334,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n1-0-0-1.effect"
        },
        "scale": 1.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.2167,
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.067,
            "band": 1.0,
            "colors": [
              [
                1.0,
                0.1535,
                0.0334,
                0.0
              ],
              [
                1.0,
                0.4248,
                0.1535,
                1.0
              ],
              [
                1.0,
                0.1535,
                0.0334,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n1-0-0-1.effect"
        },
        "scale": 1.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.2333,
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.067,
            "band": 1.0,
            "colors": [
              [
                1.0,
                0.1535,
                0.0334,
                0.0
              ],
              [
                1.0,
                0.4248,
                0.1535,
                1.0
              ],
              [
                1.0,
                0.1535,
                0.0334,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n1-0-0-1.effect"
        },
        "scale": 1.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.25,
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.2045,
            "band": 1.0,
            "height": [
              0.0,
              -1.0
            ],
            "colors": [
              [
                1.0,
                0.111,
                0.0241,
                0.0
              ],
              [
                1.0,
                0.3482,
                0.111,
                0.6005
              ],
              [
                1.0,
                0.111,
                0.0241,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n1-0-0-2.effect"
        },
        "scale": 1.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.2,
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.2045,
            "band": 1.0,
            "height": [
              0.0,
              -1.0
            ],
            "colors": [
              [
                1.0,
                0.111,
                0.0241,
                0.0
              ],
              [
                1.0,
                0.3482,
                0.111,
                0.6005
              ],
              [
                1.0,
                0.111,
                0.0241,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n1-0-0-2.effect"
        },
        "scale": 1.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.2167,
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.2045,
            "band": 1.0,
            "height": [
              0.0,
              -1.0
            ],
            "colors": [
              [
                1.0,
                0.111,
                0.0241,
                0.0
              ],
              [
                1.0,
                0.3482,
                0.111,
                0.6005
              ],
              [
                1.0,
                0.111,
                0.0241,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n1-0-0-2.effect"
        },
        "scale": 1.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.2333,
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.2045,
            "band": 1.0,
            "height": [
              0.0,
              -1.0
            ],
            "colors": [
              [
                1.0,
                0.111,
                0.0241,
                0.0
              ],
              [
                1.0,
                0.3482,
                0.111,
                0.6005
              ],
              [
                1.0,
                0.111,
                0.0241,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n1-0-0-2.effect"
        },
        "scale": 1.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.25,
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n1-0-1-0.emitter"
        }
      },
      "at": 0.2
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n1-0-1-1.emitter"
        }
      },
      "at": 0.2
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n1-0-1-2.emitter"
        }
      },
      "at": 0.2
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n1-1-0.emitter"
        }
      },
      "at": 0.1667
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n1-2-0-0-0-0-0-0.effect"
        },
        "width": [
          2.0,
          2.0
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          0.2195,
          0.0478,
          1.0
        ]
      },
      "at": 0.4,
      "offset": [
        4.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        -6.8571,
        0.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6416,
            "band": 0.35,
            "colors": [
              [
                1.0,
                0.2195,
                0.0478,
                0.0
              ],
              [
                1.0,
                0.3179,
                0.1255,
                0.2973
              ],
              [
                1.0,
                0.2195,
                0.0478,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n1-2-0-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.75
            ],
            [
              0.3125,
              2.3625
            ],
            [
              0.4219,
              2.8563
            ],
            [
              0.5312,
              3.2842
            ],
            [
              0.6406,
              3.6417
            ],
            [
              0.75,
              3.9305
            ],
            [
              0.875,
              4.1854
            ],
            [
              1.0,
              4.375
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
      "at": 0.4,
      "rotate": [
        90.0,
        0.0,
        -135.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6416,
            "band": 0.35,
            "colors": [
              [
                1.0,
                0.2195,
                0.0478,
                0.0
              ],
              [
                1.0,
                0.3179,
                0.1255,
                0.2973
              ],
              [
                1.0,
                0.2195,
                0.0478,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n1-2-0-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.75
            ],
            [
              0.3125,
              2.3625
            ],
            [
              0.4219,
              2.8563
            ],
            [
              0.5312,
              3.2842
            ],
            [
              0.6406,
              3.6417
            ],
            [
              0.75,
              3.9305
            ],
            [
              0.875,
              4.1854
            ],
            [
              1.0,
              4.375
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
      "at": 0.4167,
      "rotate": [
        90.0,
        0.0,
        -45.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6416,
            "band": 0.35,
            "colors": [
              [
                1.0,
                0.2195,
                0.0478,
                0.0
              ],
              [
                1.0,
                0.3179,
                0.1255,
                0.2973
              ],
              [
                1.0,
                0.2195,
                0.0478,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n1-2-0-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.75
            ],
            [
              0.3125,
              2.3625
            ],
            [
              0.4219,
              2.8563
            ],
            [
              0.5312,
              3.2842
            ],
            [
              0.6406,
              3.6417
            ],
            [
              0.75,
              3.9305
            ],
            [
              0.875,
              4.1854
            ],
            [
              1.0,
              4.375
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
      "at": 0.4333,
      "rotate": [
        90.0,
        0.0,
        45.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6416,
            "band": 0.35,
            "colors": [
              [
                1.0,
                0.2195,
                0.0478,
                0.0
              ],
              [
                1.0,
                0.3179,
                0.1255,
                0.2973
              ],
              [
                1.0,
                0.2195,
                0.0478,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n1-2-0-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.75
            ],
            [
              0.3125,
              2.3625
            ],
            [
              0.4219,
              2.8563
            ],
            [
              0.5312,
              3.2842
            ],
            [
              0.6406,
              3.6417
            ],
            [
              0.75,
              3.9305
            ],
            [
              0.875,
              4.1854
            ],
            [
              1.0,
              4.375
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
      "at": 0.45,
      "rotate": [
        90.0,
        0.0,
        135.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n1-2-0-1-0.emitter"
        }
      },
      "at": 0.4,
      "offset": [
        4.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n1-2-0-1-1.emitter"
        }
      },
      "at": 0.4,
      "offset": [
        4.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n1-2-0-1-2.emitter"
        }
      },
      "at": 0.4,
      "offset": [
        4.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n2-0-0-0-0-0-0.effect"
        },
        "width": [
          1.0,
          1.0
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          1.0,
          1.0,
          0.502
        ]
      },
      "at": 1.2333,
      "velocity": [
        0.0,
        -3.6923,
        0.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n2-0-0-0-0-0-1.effect"
        },
        "width": [
          2.0,
          2.0
        ],
        "lifetime": 0.4167,
        "color": [
          1.0,
          0.2195,
          0.0478,
          1.0
        ]
      },
      "at": 1.2333,
      "velocity": [
        0.0,
        -3.6923,
        0.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n2-0-1-0-0-0-0-0.effect"
        },
        "width": [
          2.0,
          2.0
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          0.2195,
          0.0478,
          1.0
        ]
      },
      "at": 1.2333,
      "offset": [
        2.5,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        -4.8,
        0.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n2-1-0-0.emitter"
        }
      },
      "at": 1.2333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n2-1-0-1.emitter"
        }
      },
      "at": 1.2333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6416,
            "band": 0.35,
            "colors": [
              [
                1.0,
                0.2195,
                0.0478,
                0.0
              ],
              [
                1.0,
                0.2596,
                0.0765,
                0.271
              ],
              [
                1.0,
                0.2195,
                0.0478,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n2-1-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.75
            ],
            [
              0.1406,
              2.4339
            ],
            [
              0.2344,
              3.4238
            ],
            [
              0.3438,
              4.3675
            ],
            [
              0.4375,
              4.978
            ],
            [
              0.5312,
              5.4145
            ],
            [
              0.6406,
              5.7393
            ],
            [
              1.0,
              6.25
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
      "at": 1.2333,
      "rotate": [
        90.0,
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
            "inner": 0.6416,
            "band": 0.35,
            "colors": [
              [
                1.0,
                0.2195,
                0.0478,
                0.0
              ],
              [
                1.0,
                0.2596,
                0.0765,
                0.271
              ],
              [
                1.0,
                0.2195,
                0.0478,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n2-1-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.75
            ],
            [
              0.1406,
              2.4339
            ],
            [
              0.2344,
              3.4238
            ],
            [
              0.3438,
              4.3675
            ],
            [
              0.4375,
              4.978
            ],
            [
              0.5312,
              5.4145
            ],
            [
              0.6406,
              5.7393
            ],
            [
              1.0,
              6.25
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
      "at": 1.25,
      "rotate": [
        90.0,
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
            "inner": 0.6416,
            "band": 0.35,
            "colors": [
              [
                1.0,
                0.2195,
                0.0478,
                0.0
              ],
              [
                1.0,
                0.2596,
                0.0765,
                0.271
              ],
              [
                1.0,
                0.2195,
                0.0478,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n2-1-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.75
            ],
            [
              0.1406,
              2.4339
            ],
            [
              0.2344,
              3.4238
            ],
            [
              0.3438,
              4.3675
            ],
            [
              0.4375,
              4.978
            ],
            [
              0.5312,
              5.4145
            ],
            [
              0.6406,
              5.7393
            ],
            [
              1.0,
              6.25
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
      "at": 1.2667,
      "rotate": [
        90.0,
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
            "inner": 0.6416,
            "band": 0.35,
            "colors": [
              [
                1.0,
                0.2195,
                0.0478,
                0.0
              ],
              [
                1.0,
                0.2596,
                0.0765,
                0.271
              ],
              [
                1.0,
                0.2195,
                0.0478,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n2-1-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.75
            ],
            [
              0.1406,
              2.4339
            ],
            [
              0.2344,
              3.4238
            ],
            [
              0.3438,
              4.3675
            ],
            [
              0.4375,
              4.978
            ],
            [
              0.5312,
              5.4145
            ],
            [
              0.6406,
              5.7393
            ],
            [
              1.0,
              6.25
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
      "at": 1.2833,
      "rotate": [
        90.0,
        0.0,
        135.0
      ],
      "life": 0.75
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n2-1-1-0.emitter"
        }
      },
      "at": 1.2333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n2-2-0-0-0.emitter"
        }
      },
      "at": 1.2667,
      "offset": [
        1.5,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n2-2-0-0-1.emitter"
        }
      },
      "at": 1.2667,
      "offset": [
        1.5,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow2/n2-2-0-0-1-0.emitter"
        }
      },
      "at": 1.2667,
      "offset": [
        1.5,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          0.2679,
          0.1314
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
      "at": 0.2,
      "life": 0.3
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.2
    }
  ],
  "duration": 2.3833,
  "events": [
    {
      "name": "impact",
      "time": 0.2
    }
  ]
});
