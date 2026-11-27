import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.1 "Distortion": motion out from a circle: approximated along one direction
// effekseer: node 0.0.3.0.0.0.0 "LineMain": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.3.0.0.0.0 "LineMain": nested generation: 100 instances above its generator folded into one emitter
// effekseer: node 0.0.3.0.0.0.1.0 "LineColor": nested generation: 100 instances above its generator folded into one emitter
// effekseer: node 0.0.3.0.0.0.1.1.0 "LineColor": nested generation: 100 instances above its generator folded into one emitter
// effekseer: node 0.0.5.0.0 "WindRing": ring 30 instances: the first 4 drawn
// effekseer: node 0.0.5.0.0 "WindRing": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 1.0.0.0.0 "Ring": ring 6 instances: the first 4 drawn
// effekseer: node 1.0.0.0.0 "Ring": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 2.0 "Ring": a moving ancestor: its descendants are placed where it starts
// effekseer: node 2.0 "Ring": ring 6 instances: the first 4 drawn
// effekseer: node 2.0 "Ring": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 3.0.0.0.1 "Wind": a moving ancestor: its descendants are placed where it starts
// effekseer: node 3.0.0.0.1 "Wind": ring unlimited instances: the first 4 drawn
// effekseer: node 3.0.0.0.1 "Wind": a ring whose height changes over its life: its middle height kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun3/n0-0-0-0-0-0.emitter"
        }
      },
      "at": 0.8333,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Gun3/n0-0-0-0-0-1.emitter"
        }
      },
      "at": 0.8333,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Gun3/n0-0-0-0-0-1-0.emitter"
        }
      },
      "at": 0.8333,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Gun3/n0-0-1.emitter"
        }
      },
      "at": 0.835
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun3/n0-0-2-0-0-0.emitter"
        }
      },
      "at": 0.8333,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Gun3/n0-0-2-0-0-1.emitter"
        }
      },
      "at": 0.8333,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Gun3/n0-0-2-0-0-1-0.emitter"
        }
      },
      "at": 0.8333,
      "offset": [
        0.0,
        2.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun3/n0-0-3-0-0-0-0.effect"
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
      "at": 0.8333,
      "offset": [
        0.0,
        0.5,
        0.0
      ],
      "velocity": [
        0.0,
        -3.2432,
        0.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun3/n0-0-3-0-0-0-1-0.effect"
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
      "at": 0.8333,
      "offset": [
        0.0,
        0.5,
        0.0
      ],
      "velocity": [
        0.0,
        4.471,
        0.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun3/n0-0-3-0-0-0-1-1-0.effect"
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
      "at": 0.8333,
      "offset": [
        0.0,
        0.5,
        0.0
      ],
      "velocity": [
        0.0,
        4.471,
        0.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun3/n0-0-4.emitter"
        }
      },
      "at": 0.8333
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
          "$ogRef": "base/vfx/effekseer-tktk02/Gun3/n0-0-5-0-0.effect"
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
      "at": 0.9667,
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.7955,
            "band": 1.0,
            "colors": [
              [
                1.0,
                0.2618,
                0.0671,
                0.0
              ],
              [
                1.0,
                0.6022,
                0.3074,
                1.0
              ],
              [
                1.0,
                0.2618,
                0.0671,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun3/n1-0-0-0-0.effect"
        },
        "scale": 0.6667,
        "color": [
          6.0,
          6.0,
          6.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0017,
      "life": 1.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9336,
            "band": 0.875,
            "colors": [
              [
                1.0,
                0.3081,
                0.09,
                0.0
              ],
              [
                1.0,
                0.673,
                0.4116,
                1.0
              ],
              [
                1.0,
                0.3081,
                0.09,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun3/n2-0.effect"
        },
        "scale": 2.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.5,
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
            "inner": 0.9336,
            "band": 0.875,
            "colors": [
              [
                1.0,
                0.3081,
                0.09,
                0.0
              ],
              [
                1.0,
                0.673,
                0.4116,
                1.0
              ],
              [
                1.0,
                0.3081,
                0.09,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun3/n2-0.effect"
        },
        "scale": 2.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.5167,
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
            "inner": 0.9336,
            "band": 0.875,
            "colors": [
              [
                1.0,
                0.3081,
                0.09,
                0.0
              ],
              [
                1.0,
                0.673,
                0.4116,
                1.0
              ],
              [
                1.0,
                0.3081,
                0.09,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun3/n2-0.effect"
        },
        "scale": 2.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.5333,
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
            "inner": 0.9336,
            "band": 0.875,
            "colors": [
              [
                1.0,
                0.3081,
                0.09,
                0.0
              ],
              [
                1.0,
                0.673,
                0.4116,
                1.0
              ],
              [
                1.0,
                0.3081,
                0.09,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun3/n2-0.effect"
        },
        "scale": 2.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.55,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Gun3/n3-0-0-0-0-0.emitter"
        }
      },
      "at": 0.7,
      "offset": [
        1.3054,
        -2.7362,
        7.4033
      ],
      "rotate": [
        20.2836,
        9.3913,
        -3.4512
      ],
      "velocity": [
        -15.6649,
        32.8339,
        -88.84
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun3/n3-0-0-0-0-1.emitter"
        }
      },
      "at": 0.7,
      "offset": [
        1.3054,
        -2.7362,
        7.4033
      ],
      "rotate": [
        20.2836,
        9.3913,
        -3.4512
      ],
      "velocity": [
        -15.6649,
        32.8339,
        -88.84
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun3/n3-0-0-0-0-2-0.effect"
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
      "at": 0.7,
      "offset": [
        1.3054,
        -2.7362,
        7.4033
      ],
      "rotate": [
        20.2836,
        9.3913,
        -3.4512
      ],
      "velocity": [
        -15.6649,
        32.8339,
        -88.84
      ],
      "life": 0.1667
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun3/n3-0-0-0-0-2-1-0.effect"
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
      "at": 0.7,
      "offset": [
        1.3054,
        -2.7362,
        7.4033
      ],
      "rotate": [
        20.2836,
        9.3913,
        -3.4512
      ],
      "velocity": [
        -15.6649,
        32.8339,
        -88.84
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
              1.6004,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Gun3/n3-0-0-0-1.effect"
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
      "at": 0.7167,
      "offset": [
        1.3054,
        -2.7362,
        7.4033
      ],
      "rotate": [
        20.2836,
        9.3913,
        -138.4512
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
              1.6004,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Gun3/n3-0-0-0-1.effect"
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
      "at": 0.7183,
      "offset": [
        1.3054,
        -2.7362,
        7.4033
      ],
      "rotate": [
        20.2836,
        9.3913,
        -48.4512
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
              1.6004,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Gun3/n3-0-0-0-1.effect"
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
      "at": 0.72,
      "offset": [
        1.3054,
        -2.7362,
        7.4033
      ],
      "rotate": [
        20.2836,
        9.3913,
        41.5488
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
              1.6004,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Gun3/n3-0-0-0-1.effect"
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
      "at": 0.7217,
      "offset": [
        1.3054,
        -2.7362,
        7.4033
      ],
      "rotate": [
        20.2836,
        9.3913,
        131.5488
      ],
      "life": 0.25
    }
  ],
  "duration": 2.1693,
  "events": [
    {
      "name": "impact",
      "time": 0.7167
    }
  ]
});
