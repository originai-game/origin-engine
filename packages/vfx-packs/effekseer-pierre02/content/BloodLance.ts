import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.2.0.0.0.0.0.0 "Lance": model 40 instances: the first 4 drawn
// effekseer: node 0.2.0.0.0.0.0.0 "Lance": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 0.2.0.0.0.0.0.1 "Blood": model 40 instances: the first 4 drawn
// effekseer: node 0.2.0.0.0.0.0.1 "Blood": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 0.2.0.0.0.0.0.2.0 "EmberVariation": nested generation: 120 instances above its generator folded into one emitter
// effekseer: node 0.2.0.0.0.0.0.2.0 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.0.0.0.0.0.2.1 "EmberVariation": nested generation: 120 instances above its generator folded into one emitter
// effekseer: node 0.2.0.0.0.0.0.2.1 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.0.0.0.0.0.2.2 "EmberVariation": nested generation: 120 instances above its generator folded into one emitter
// effekseer: node 0.2.0.0.0.0.0.2.2 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.0.0.0.1.0.0 "Wind": ring 2 instances: the first 4 drawn
// effekseer: node 0.2.0.0.0.1.0.0 "Wind": nested generation: 240 instances above its generator folded into one emitter
// effekseer: node 0.2.0.0.0.1.0.1 "Wind": ring 2 instances: the first 4 drawn
// effekseer: node 0.2.0.0.0.1.0.1 "Wind": nested generation: 240 instances above its generator folded into one emitter
// effekseer: node 0.2.0.0.0.1.1.0 "FireVariation": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 0.2.0.0.0.1.1.0 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.0.0.0.1.1.1 "FireVariation": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 0.2.0.0.0.1.1.1 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.0.0.0.1.1.2 "FireVariation": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 0.2.0.0.0.1.1.2 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.0.0.0.1.1.3 "FireVariation": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 0.2.0.0.0.1.1.3 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.0.0.0.1.2.0.0 "FireVariation": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 0.2.0.0.0.1.2.0.0 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.0.0.0.1.2.0.1 "FireVariation": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 0.2.0.0.0.1.2.0.1 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.0.0.0.1.2.0.2 "FireVariation": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 0.2.0.0.0.1.2.0.2 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.0.0.0.1.2.0.3 "FireVariation": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 0.2.0.0.0.1.2.0.3 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.0.0.0.1.2.0.4 "EmberVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.0.0.0.1.2.0.4 "EmberVariation": nested generation: 120 instances above its generator folded into one emitter
// effekseer: node 0.2.0.0.0.1.2.0.4 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.0.0.0.1.2.0.5 "EmberVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.0.0.0.1.2.0.5 "EmberVariation": nested generation: 120 instances above its generator folded into one emitter
// effekseer: node 0.2.0.0.0.1.2.0.5 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.0.0.0.1.3.0.0 "EmberVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.0.0.0.1.3.0.0 "EmberVariation": nested generation: 120 instances above its generator folded into one emitter
// effekseer: node 0.2.0.0.0.1.3.0.0 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.0.0.0.1.3.0.1 "EmberVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.0.0.0.1.3.0.1 "EmberVariation": nested generation: 120 instances above its generator folded into one emitter
// effekseer: node 0.2.0.0.0.1.3.0.1 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.0968,
        0.0
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "until": 2.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-0-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.0968,
        0.0
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "until": 2.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-0-2.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.0984,
        0.0
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "until": 2.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-0-3.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.1,
        0.0
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "until": 2.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-0-4.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              9.6
            ],
            [
              1.0,
              20.0
            ]
          ]
        },
        "color": [
          0.2195,
          0.0055,
          0.0104,
          0.3765
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.1,
        0.0
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 2.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-0-4.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              9.6
            ],
            [
              1.0,
              20.0
            ]
          ]
        },
        "color": [
          0.2195,
          0.0055,
          0.0104,
          0.3765
        ],
        "blend": "additive"
      },
      "at": 0.5,
      "offset": [
        0.0,
        0.1,
        0.0
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 2.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-0-5-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.1,
        0.0
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "until": 2.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-0-5-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.1,
        0.0
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
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-0-5-2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              8.0
            ],
            [
              0.6719,
              17.9879
            ],
            [
              0.8438,
              19.8423
            ],
            [
              1.0,
              20.8
            ]
          ]
        },
        "color": [
          1.0,
          0.1166,
          0.0254,
          0.3765
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.1,
        0.0
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 0.35
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-0-5-3.emitter"
        }
      },
      "at": 4.8833,
      "offset": [
        0.0,
        0.1,
        0.0
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
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-0-5-4.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              20.8
            ],
            [
              0.6719,
              7.0666
            ],
            [
              0.8438,
              4.5168
            ],
            [
              1.0,
              3.2
            ]
          ]
        },
        "color": [
          1.0,
          0.1166,
          0.0254,
          0.251
        ],
        "blend": "additive"
      },
      "at": 4.8833,
      "offset": [
        0.0,
        0.1,
        0.0
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 0.35
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-1-0.emitter"
        }
      },
      "at": 0.1,
      "offset": [
        0.0,
        0.1,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-1-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.1,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-1-2.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.1,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-1-3.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.1,
        0.0
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/model-ShortLance"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-0-0-0.effect"
        },
        "scale": 0.1433,
        "color": [
          0.1495,
          0.0035,
          0.0074,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 1.65,
      "offset": [
        0.0,
        0.1,
        5.2
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 2.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/model-ShortLance"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-0-0-0.effect"
        },
        "scale": 0.1433,
        "color": [
          0.1495,
          0.0035,
          0.0074,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 1.6767,
      "offset": [
        0.0,
        0.1,
        5.2
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 2.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/model-ShortLance"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-0-0-0.effect"
        },
        "scale": 0.1433,
        "color": [
          0.1495,
          0.0035,
          0.0074,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 1.7033,
      "offset": [
        0.0,
        0.1,
        5.2
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 2.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/model-ShortLance"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-0-0-0.effect"
        },
        "scale": 0.1433,
        "color": [
          0.1495,
          0.0035,
          0.0074,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 1.73,
      "offset": [
        0.0,
        0.1,
        5.2
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 2.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/model-ShortLance"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-0-0-1.effect"
        },
        "scale": 0.1433,
        "color": [
          0.2195,
          0.0055,
          0.0104,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 1.6,
      "offset": [
        0.0,
        0.1,
        5.2
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 2.5833
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/model-ShortLance"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-0-0-1.effect"
        },
        "scale": 0.1433,
        "color": [
          0.2195,
          0.0055,
          0.0104,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 1.6267,
      "offset": [
        0.0,
        0.1,
        5.2
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 2.5833
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/model-ShortLance"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-0-0-1.effect"
        },
        "scale": 0.1433,
        "color": [
          0.2195,
          0.0055,
          0.0104,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 1.6533,
      "offset": [
        0.0,
        0.1,
        5.2
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 2.5833
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/model-ShortLance"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-0-0-1.effect"
        },
        "scale": 0.1433,
        "color": [
          0.2195,
          0.0055,
          0.0104,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 1.68,
      "offset": [
        0.0,
        0.1,
        5.2
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 2.5833
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-0-0-2-0.emitter"
        }
      },
      "at": 1.95,
      "offset": [
        0.0,
        1.1204,
        6.4161
      ],
      "rotate": [
        50.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        3.2139,
        3.8302
      ],
      "until": 1.95
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-0-0-2-1.emitter"
        }
      },
      "at": 1.95,
      "offset": [
        0.0,
        1.1204,
        6.4161
      ],
      "rotate": [
        50.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        3.2139,
        3.8302
      ],
      "until": 1.95
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-0-0-2-2.emitter"
        }
      },
      "at": 1.95,
      "offset": [
        0.0,
        1.1204,
        6.4161
      ],
      "rotate": [
        50.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        3.2139,
        3.8302
      ],
      "until": 1.95
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.5,
            "height": [
              -0.2,
              0.2
            ],
            "colors": [
              [
                0.2195,
                0.0055,
                0.0104,
                0.3765
              ],
              [
                0.2195,
                0.0055,
                0.0104,
                0.8784
              ],
              [
                0.2195,
                0.0055,
                0.0104,
                0.0941
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-1-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.5344
            ],
            [
              0.0781,
              0.8192
            ],
            [
              0.1562,
              1.0792
            ],
            [
              0.2344,
              1.3179
            ],
            [
              0.3281,
              1.5807
            ],
            [
              0.4688,
              1.9384
            ],
            [
              0.8281,
              2.7754
            ],
            [
              1.0,
              3.2067
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          180.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          0.6667
        ],
        "blend": "alpha"
      },
      "at": 1.75,
      "offset": [
        0.0,
        0.3577,
        5.5071
      ],
      "rotate": [
        141.5,
        0.0,
        178.5
      ],
      "life": 0.45
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.5,
            "height": [
              -0.2,
              0.2
            ],
            "colors": [
              [
                0.2195,
                0.0055,
                0.0104,
                0.3765
              ],
              [
                0.2195,
                0.0055,
                0.0104,
                0.8784
              ],
              [
                0.2195,
                0.0055,
                0.0104,
                0.0941
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-1-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.5344
            ],
            [
              0.0781,
              0.8192
            ],
            [
              0.1562,
              1.0792
            ],
            [
              0.2344,
              1.3179
            ],
            [
              0.3281,
              1.5807
            ],
            [
              0.4688,
              1.9384
            ],
            [
              0.8281,
              2.7754
            ],
            [
              1.0,
              3.2067
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          180.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          0.6667
        ],
        "blend": "alpha"
      },
      "at": 1.7667,
      "offset": [
        0.0,
        0.3577,
        5.5071
      ],
      "rotate": [
        138.5,
        0.0,
        -178.5
      ],
      "life": 0.45
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.5,
            "height": [
              -0.2,
              0.2
            ],
            "colors": [
              [
                0.2195,
                0.0055,
                0.0104,
                0.3765
              ],
              [
                0.2195,
                0.0055,
                0.0104,
                0.8784
              ],
              [
                0.2195,
                0.0055,
                0.0104,
                0.0941
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-1-0-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.5344
            ],
            [
              0.0781,
              0.8192
            ],
            [
              0.1562,
              1.0792
            ],
            [
              0.2344,
              1.3179
            ],
            [
              0.3281,
              1.5807
            ],
            [
              0.4688,
              1.9384
            ],
            [
              0.8281,
              2.7754
            ],
            [
              1.0,
              3.2067
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          180.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          0.9167
        ],
        "blend": "alpha"
      },
      "at": 1.75,
      "offset": [
        0.0,
        0.3577,
        5.5071
      ],
      "rotate": [
        141.5,
        0.0,
        178.5
      ],
      "life": 0.6
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.5,
            "height": [
              -0.2,
              0.2
            ],
            "colors": [
              [
                0.2195,
                0.0055,
                0.0104,
                0.3765
              ],
              [
                0.2195,
                0.0055,
                0.0104,
                0.8784
              ],
              [
                0.2195,
                0.0055,
                0.0104,
                0.0941
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-1-0-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.5344
            ],
            [
              0.0781,
              0.8192
            ],
            [
              0.1562,
              1.0792
            ],
            [
              0.2344,
              1.3179
            ],
            [
              0.3281,
              1.5807
            ],
            [
              0.4688,
              1.9384
            ],
            [
              0.8281,
              2.7754
            ],
            [
              1.0,
              3.2067
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          180.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          0.9167
        ],
        "blend": "alpha"
      },
      "at": 1.7667,
      "offset": [
        0.0,
        0.3577,
        5.5071
      ],
      "rotate": [
        138.5,
        0.0,
        -178.5
      ],
      "life": 0.6
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-1-1-0.emitter"
        }
      },
      "at": 1.7,
      "offset": [
        0.0,
        0.1,
        5.2
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-1-1-1.emitter"
        }
      },
      "at": 1.8333,
      "offset": [
        0.0,
        0.1,
        5.2
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-1-1-2.emitter"
        }
      },
      "at": 1.9667,
      "offset": [
        0.0,
        0.1,
        5.2
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-1-1-3.emitter"
        }
      },
      "at": 2.1,
      "offset": [
        0.0,
        0.1,
        5.2
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-1-2-0-0.emitter"
        }
      },
      "at": 1.6833,
      "offset": [
        0.0,
        0.1,
        5.2
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-1-2-0-1.emitter"
        }
      },
      "at": 1.8333,
      "offset": [
        0.0,
        0.1,
        5.2
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-1-2-0-2.emitter"
        }
      },
      "at": 1.9833,
      "offset": [
        0.0,
        0.1,
        5.2
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-1-2-0-3.emitter"
        }
      },
      "at": 2.1333,
      "offset": [
        0.0,
        0.1,
        5.2
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-1-2-0-4.emitter"
        }
      },
      "at": 2.1,
      "offset": [
        0.0,
        1.1042,
        6.3967
      ],
      "rotate": [
        50.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-1-2-0-5.emitter"
        }
      },
      "at": 2.2,
      "offset": [
        0.0,
        1.1042,
        6.3967
      ],
      "rotate": [
        50.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-1-3-0-0.parent"
        }
      },
      "at": 2.0167,
      "offset": [
        0.0,
        0.1,
        5.2
      ],
      "rotate": [
        50.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-1-3-0-1.parent"
        }
      },
      "at": 2.0167,
      "offset": [
        0.0,
        0.1,
        5.2
      ],
      "rotate": [
        50.0,
        0.0,
        0.0
      ]
    }
  ],
  "duration": 6.5,
  "looping": true
});
