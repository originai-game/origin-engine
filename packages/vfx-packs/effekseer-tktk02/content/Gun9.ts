import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0.0.0.0 "LineMain": nested generation: 7 instances above its generator folded into one emitter
// effekseer: node 0.0.0.0.0.0.0 "LineMain": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0.0.0.0.1 "LineColor": nested generation: 7 instances above its generator folded into one emitter
// effekseer: node 0.0.0.0.0.0.1 "LineColor": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0.0.0.0.1.0 "LineColor": nested generation: 7 instances above its generator folded into one emitter
// effekseer: node 0.0.0.0.0.0.1.0 "LineColor": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0.1.0.0.0.0 "LineMain": nested generation: 7 instances above its generator folded into one emitter
// effekseer: node 0.0.0.1.0.0.0.0 "LineMain": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0.1.0.0.0.1 "LineColor": nested generation: 7 instances above its generator folded into one emitter
// effekseer: node 0.0.0.1.0.0.0.1 "LineColor": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0.1.0.0.0.1.0 "LineGlow": nested generation: 7 instances above its generator folded into one emitter
// effekseer: node 0.0.0.1.0.0.0.1.0 "LineGlow": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0.2.0 "RingMain": ring 7 instances: the first 4 drawn
// effekseer: node 0.0.0.2.0 "RingMain": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.0.0.2.1 "Ring1": ring 7 instances: the first 4 drawn
// effekseer: node 0.0.0.2.1 "Ring1": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.0.0.3 "Distortion": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0 "ｉｎｔｅｒｖａｌ": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0.4 "Glow1": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0.5 "GlowRGB": motion out from a circle: approximated along one direction
// effekseer: node 1.0.0.0.0.0.0 "LineMain": motion out from a circle: approximated along one direction
// effekseer: node 1.0.0.0.0.0.1 "LineColor": motion out from a circle: approximated along one direction
// effekseer: node 1.0.0.0.0.0.1.0 "LineColor": motion out from a circle: approximated along one direction
// effekseer: node 1.0.0.1.0.0.0.0 "LineMain": motion out from a circle: approximated along one direction
// effekseer: node 1.0.0.1.0.0.0.1 "LineColor": motion out from a circle: approximated along one direction
// effekseer: node 1.0.0.1.0.0.0.1.0 "LineGlow": motion out from a circle: approximated along one direction
// effekseer: node 1.0.0.2.0 "RingMain": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 1.0.0.2.1 "Ring1": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 1.0.1 "ｃｒａｓｈ": ring 72 instances: the first 4 drawn
// effekseer: node 1.0.2 "Distortion": motion out from a circle: approximated along one direction
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n0-0-0-0-0-0-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n0-0-0-0-0-0-1.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n0-0-0-0-0-0-1-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n0-0-0-1-0-0-0-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n0-0-0-1-0-0-0-1.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n0-0-0-1-0-0-0-1-0.parent"
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
            "inner": 0.8146,
            "band": 1.0,
            "colors": [
              [
                0.0461,
                0.1768,
                1.0,
                0.0
              ],
              [
                0.2853,
                0.5862,
                1.0,
                0.1415
              ],
              [
                0.0461,
                0.1768,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n0-0-0-2-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.375
            ],
            [
              0.1719,
              0.439
            ],
            [
              0.375,
              0.4799
            ],
            [
              0.5781,
              0.4953
            ],
            [
              1.0,
              0.5
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
      "at": 0.0333,
      "rotate": [
        0.0,
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
            "inner": 0.8146,
            "band": 1.0,
            "colors": [
              [
                0.0461,
                0.1768,
                1.0,
                0.0
              ],
              [
                0.2853,
                0.5862,
                1.0,
                0.1415
              ],
              [
                0.0461,
                0.1768,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n0-0-0-2-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.375
            ],
            [
              0.1719,
              0.439
            ],
            [
              0.375,
              0.4799
            ],
            [
              0.5781,
              0.4953
            ],
            [
              1.0,
              0.5
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
      "rotate": [
        0.0,
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
            "inner": 0.8146,
            "band": 1.0,
            "colors": [
              [
                0.0461,
                0.1768,
                1.0,
                0.0
              ],
              [
                0.2853,
                0.5862,
                1.0,
                0.1415
              ],
              [
                0.0461,
                0.1768,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n0-0-0-2-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.375
            ],
            [
              0.1719,
              0.439
            ],
            [
              0.375,
              0.4799
            ],
            [
              0.5781,
              0.4953
            ],
            [
              1.0,
              0.5
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
      "at": 0.1667,
      "rotate": [
        0.0,
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
            "inner": 0.8146,
            "band": 1.0,
            "colors": [
              [
                0.0461,
                0.1768,
                1.0,
                0.0
              ],
              [
                0.2853,
                0.5862,
                1.0,
                0.1415
              ],
              [
                0.0461,
                0.1768,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n0-0-0-2-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.375
            ],
            [
              0.1719,
              0.439
            ],
            [
              0.375,
              0.4799
            ],
            [
              0.5781,
              0.4953
            ],
            [
              1.0,
              0.5
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
      "at": 0.2333,
      "rotate": [
        0.0,
        0.0,
        135.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5085,
            "band": 1.0,
            "colors": [
              [
                0.0461,
                0.1768,
                1.0,
                0.0
              ],
              [
                0.2853,
                0.5862,
                1.0,
                0.502
              ],
              [
                0.0461,
                0.1768,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n0-0-0-2-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.375
            ],
            [
              0.1719,
              0.439
            ],
            [
              0.375,
              0.4799
            ],
            [
              0.5781,
              0.4953
            ],
            [
              1.0,
              0.5
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
      "at": 0.0333,
      "rotate": [
        0.0,
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
            "inner": 0.5085,
            "band": 1.0,
            "colors": [
              [
                0.0461,
                0.1768,
                1.0,
                0.0
              ],
              [
                0.2853,
                0.5862,
                1.0,
                0.502
              ],
              [
                0.0461,
                0.1768,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n0-0-0-2-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.375
            ],
            [
              0.1719,
              0.439
            ],
            [
              0.375,
              0.4799
            ],
            [
              0.5781,
              0.4953
            ],
            [
              1.0,
              0.5
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
      "rotate": [
        0.0,
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
            "inner": 0.5085,
            "band": 1.0,
            "colors": [
              [
                0.0461,
                0.1768,
                1.0,
                0.0
              ],
              [
                0.2853,
                0.5862,
                1.0,
                0.502
              ],
              [
                0.0461,
                0.1768,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n0-0-0-2-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.375
            ],
            [
              0.1719,
              0.439
            ],
            [
              0.375,
              0.4799
            ],
            [
              0.5781,
              0.4953
            ],
            [
              1.0,
              0.5
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
      "at": 0.1667,
      "rotate": [
        0.0,
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
            "inner": 0.5085,
            "band": 1.0,
            "colors": [
              [
                0.0461,
                0.1768,
                1.0,
                0.0
              ],
              [
                0.2853,
                0.5862,
                1.0,
                0.502
              ],
              [
                0.0461,
                0.1768,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n0-0-0-2-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.375
            ],
            [
              0.1719,
              0.439
            ],
            [
              0.375,
              0.4799
            ],
            [
              0.5781,
              0.4953
            ],
            [
              1.0,
              0.5
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
      "at": 0.2333,
      "rotate": [
        0.0,
        0.0,
        135.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n0-0-0-3.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n0-0-0-4.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n0-0-0-5.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n1-0-0-0-0-0-0.parent"
        }
      },
      "at": 0.6
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n1-0-0-0-0-0-1.parent"
        }
      },
      "at": 0.6
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n1-0-0-0-0-0-1-0.parent"
        }
      },
      "at": 0.6
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n1-0-0-1-0-0-0-0.parent"
        }
      },
      "at": 0.6
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n1-0-0-1-0-0-0-1.parent"
        }
      },
      "at": 0.6
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n1-0-0-1-0-0-0-1-0.parent"
        }
      },
      "at": 0.6
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.8146,
            "band": 1.0,
            "colors": [
              [
                0.0605,
                0.2158,
                1.0,
                0.0
              ],
              [
                0.3585,
                0.6378,
                1.0,
                0.1571
              ],
              [
                0.0605,
                0.2158,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n1-0-0-2-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.5625
            ],
            [
              0.1719,
              0.6585
            ],
            [
              0.375,
              0.7199
            ],
            [
              0.5781,
              0.743
            ],
            [
              1.0,
              0.75
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
      "at": 0.6333,
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5085,
            "band": 1.0,
            "colors": [
              [
                0.0605,
                0.2158,
                1.0,
                0.0
              ],
              [
                0.3585,
                0.6378,
                1.0,
                0.502
              ],
              [
                0.0605,
                0.2158,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n1-0-0-2-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.5625
            ],
            [
              0.1719,
              0.6585
            ],
            [
              0.375,
              0.7199
            ],
            [
              0.5781,
              0.743
            ],
            [
              1.0,
              0.75
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
      "at": 0.6333,
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
            "start": 265.0,
            "length": 10.0,
            "colors": [
              [
                0.0714,
                0.2444,
                1.0,
                0.0
              ],
              [
                0.3298,
                0.5521,
                1.0,
                0.2968
              ],
              [
                0.0714,
                0.2444,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n1-0-1.effect"
        },
        "scale": 1.0,
        "color": [
          72.0,
          72.0,
          72.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.6833,
      "velocity": [
        1.8,
        0.0,
        0.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n1-0-2.emitter"
        }
      },
      "at": 0.6017
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n1-0-3.emitter"
        }
      },
      "at": 0.6
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n1-0-4.emitter"
        }
      },
      "at": 0.6
    }
  ],
  "duration": 3.0667,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
