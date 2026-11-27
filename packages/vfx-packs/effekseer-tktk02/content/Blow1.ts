import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.2.0.0.0 "LineMain": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.2.0.0.0 "LineMain": nested generation: 50 instances above its generator folded into one emitter
// effekseer: node 0.0.2.0.0.1 "LineColor": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.2.0.0.1 "LineColor": nested generation: 50 instances above its generator folded into one emitter
// effekseer: node 0.1.0 "Ring": ring 20 instances: the first 4 drawn
// effekseer: node 0.1.0 "Ring": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.2 "Distortion": motion out from a circle: approximated along one direction
// effekseer: node 0.3 "Ring": ring 10 instances: the first 4 drawn
// effekseer: node 0.3 "Ring": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 1.0.0.0 "LineMain": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.0.0.1 "LineColor": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.0.0.2 "LineColor": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.0.0.3.0 "Ring": ring 3 instances: the first 4 drawn
// effekseer: node 1.0.0.3.0 "Ring": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 1.0.0.3.0 "Ring": texture not found: Parts/のnoise.png
// effekseer: node 1.0.0.3.0 "Ring": a ring whose thickness changes over its life: its middle thickness kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow1/n0-0-0-0-0-0.emitter"
        }
      },
      "at": 0.1667,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow1/n0-0-0-0-0-1-0.emitter"
        }
      },
      "at": 0.1667,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow1/n0-0-1-0-0-0.emitter"
        }
      },
      "at": 0.1667,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow1/n0-0-1-0-0-1.emitter"
        }
      },
      "at": 0.1667,
      "offset": [
        0.0,
        0.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow1/n0-0-2-0-0-0.effect"
        },
        "width": [
          0.5009,
          0.5009
        ],
        "lifetime": 0.5,
        "color": [
          1.0,
          1.0,
          1.0,
          0.251
        ]
      },
      "at": 0.1667,
      "offset": [
        0.0,
        0.5,
        0.0
      ],
      "rotate": [
        -180.0,
        0.0,
        0.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow1/n0-0-2-0-0-1.effect"
        },
        "width": [
          3.5417,
          3.5417
        ],
        "lifetime": 0.5,
        "color": [
          1.0,
          0.2195,
          0.0478,
          0.251
        ]
      },
      "at": 0.1667,
      "offset": [
        0.0,
        0.5,
        0.0
      ],
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
            "inner": 0.8462,
            "band": 0.5,
            "start": 150.0,
            "length": 240.0,
            "colors": [
              [
                1.0,
                0.2618,
                0.0781,
                0.0
              ],
              [
                1.0,
                0.3409,
                0.107,
                0.3621
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow1/n0-1-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.875
            ],
            [
              0.2031,
              1.1681
            ],
            [
              0.4375,
              1.3738
            ],
            [
              0.6875,
              1.4775
            ],
            [
              1.0,
              1.5
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          420.0
        ],
        "color": [
          20.0,
          20.0,
          20.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.2333,
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow1/n0-1-1.emitter"
        }
      },
      "at": 0.2333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow1/n0-1-1-0.emitter"
        }
      },
      "at": 0.2333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow1/n0-2.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.0829,
            "band": 1.0,
            "colors": [
              [
                1.0,
                0.4522,
                0.1694,
                0.0
              ],
              [
                1.0,
                0.673,
                0.4116,
                0.1673
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow1/n0-3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.25
            ],
            [
              0.125,
              0.6454
            ],
            [
              0.4219,
              1.7238
            ],
            [
              0.5781,
              2.2067
            ],
            [
              0.6719,
              2.4463
            ],
            [
              0.7812,
              2.6767
            ],
            [
              0.8906,
              2.8586
            ],
            [
              1.0,
              3.0
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
      "at": 0.1667,
      "rotate": [
        90.0,
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
            "inner": 1.0829,
            "band": 1.0,
            "colors": [
              [
                1.0,
                0.4522,
                0.1694,
                0.0
              ],
              [
                1.0,
                0.673,
                0.4116,
                0.1673
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow1/n0-3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.25
            ],
            [
              0.125,
              0.6454
            ],
            [
              0.4219,
              1.7238
            ],
            [
              0.5781,
              2.2067
            ],
            [
              0.6719,
              2.4463
            ],
            [
              0.7812,
              2.6767
            ],
            [
              0.8906,
              2.8586
            ],
            [
              1.0,
              3.0
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
      "at": 0.1683,
      "rotate": [
        90.0,
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
            "inner": 1.0829,
            "band": 1.0,
            "colors": [
              [
                1.0,
                0.4522,
                0.1694,
                0.0
              ],
              [
                1.0,
                0.673,
                0.4116,
                0.1673
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow1/n0-3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.25
            ],
            [
              0.125,
              0.6454
            ],
            [
              0.4219,
              1.7238
            ],
            [
              0.5781,
              2.2067
            ],
            [
              0.6719,
              2.4463
            ],
            [
              0.7812,
              2.6767
            ],
            [
              0.8906,
              2.8586
            ],
            [
              1.0,
              3.0
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
      "at": 0.17,
      "rotate": [
        90.0,
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
            "inner": 1.0829,
            "band": 1.0,
            "colors": [
              [
                1.0,
                0.4522,
                0.1694,
                0.0
              ],
              [
                1.0,
                0.673,
                0.4116,
                0.1673
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow1/n0-3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.25
            ],
            [
              0.125,
              0.6454
            ],
            [
              0.4219,
              1.7238
            ],
            [
              0.5781,
              2.2067
            ],
            [
              0.6719,
              2.4463
            ],
            [
              0.7812,
              2.6767
            ],
            [
              0.8906,
              2.8586
            ],
            [
              1.0,
              3.0
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
      "at": 0.1717,
      "rotate": [
        90.0,
        0.0,
        135.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow1/n0-3-0.emitter"
        }
      },
      "at": 0.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow1/n0-4.emitter"
        }
      },
      "at": 0.2333
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow1/n1-0-0-0.effect"
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
          1.0
        ]
      },
      "at": 0.0,
      "offset": [
        0.0,
        -4.0,
        0.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow1/n1-0-0-1.effect"
        },
        "width": [
          2.0,
          2.0
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          0.5356,
          0.2195,
          1.0
        ]
      },
      "at": 0.0,
      "offset": [
        0.0,
        -4.0,
        0.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow1/n1-0-0-2.effect"
        },
        "width": [
          5.0,
          5.0
        ],
        "lifetime": 0.3333,
        "color": [
          1.0,
          0.2195,
          0.0478,
          0.251
        ]
      },
      "at": 0.0,
      "offset": [
        0.0,
        -4.0,
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
            "inner": 0.8409,
            "band": 0.625,
            "colors": [
              [
                1.0,
                0.3587,
                0.1166,
                0.0
              ],
              [
                1.0,
                0.7482,
                0.5326,
                0.3765
              ],
              [
                1.0,
                0.3587,
                0.1166,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow1/n1-0-0-3-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.925
            ],
            [
              0.0938,
              1.1494
            ],
            [
              0.1875,
              1.3306
            ],
            [
              0.2969,
              1.4978
            ],
            [
              0.4062,
              1.6266
            ],
            [
              0.6719,
              1.8255
            ],
            [
              1.0,
              1.9375
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          1200.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1,
      "offset": [
        0.0,
        -2.9,
        0.0
      ],
      "rotate": [
        90.0,
        0.0,
        -120.0
      ],
      "velocity": [
        0.0,
        21.0,
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
            "inner": 0.8409,
            "band": 0.625,
            "colors": [
              [
                1.0,
                0.3587,
                0.1166,
                0.0
              ],
              [
                1.0,
                0.7482,
                0.5326,
                0.3765
              ],
              [
                1.0,
                0.3587,
                0.1166,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow1/n1-0-0-3-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.925
            ],
            [
              0.0938,
              1.1494
            ],
            [
              0.1875,
              1.3306
            ],
            [
              0.2969,
              1.4978
            ],
            [
              0.4062,
              1.6266
            ],
            [
              0.6719,
              1.8255
            ],
            [
              1.0,
              1.9375
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          1200.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1002,
      "offset": [
        0.0,
        -2.9,
        0.0
      ],
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        21.0,
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
            "inner": 0.8409,
            "band": 0.625,
            "colors": [
              [
                1.0,
                0.3587,
                0.1166,
                0.0
              ],
              [
                1.0,
                0.7482,
                0.5326,
                0.3765
              ],
              [
                1.0,
                0.3587,
                0.1166,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow1/n1-0-0-3-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.925
            ],
            [
              0.0938,
              1.1494
            ],
            [
              0.1875,
              1.3306
            ],
            [
              0.2969,
              1.4978
            ],
            [
              0.4062,
              1.6266
            ],
            [
              0.6719,
              1.8255
            ],
            [
              1.0,
              1.9375
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          1200.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1003,
      "offset": [
        0.0,
        -2.9,
        0.0
      ],
      "rotate": [
        90.0,
        0.0,
        120.0
      ],
      "velocity": [
        0.0,
        21.0,
        0.0
      ],
      "life": 0.3333
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
      "at": 0.0,
      "life": 0.3
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.0
    }
  ],
  "duration": 1.1667,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
