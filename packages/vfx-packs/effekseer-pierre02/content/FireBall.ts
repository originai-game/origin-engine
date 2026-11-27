import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0 "Wind": ring 9 instances: the first 4 drawn
// effekseer: node 0.0.0 "Wind": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 0.0.1.0.0 "Tracker": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.1.0.0 "Tracker": nested generation: 27 instances above its generator folded into one emitter
// effekseer: node 0.1.0.0.0 "Light": nested generation: 24 instances above its generator folded into one emitter
// effekseer: node 0.1.0.0.1 "FireVariation": nested generation: 24 instances above its generator folded into one emitter
// effekseer: node 0.1.0.0.1 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.0.0.2 "FireVariation": nested generation: 24 instances above its generator folded into one emitter
// effekseer: node 0.1.0.0.2 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.0.0.3 "FireVariation": nested generation: 24 instances above its generator folded into one emitter
// effekseer: node 0.1.0.0.3 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.0.0.4 "FireVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.1.0.0.4 "FireVariation": nested generation: 24 instances above its generator folded into one emitter
// effekseer: node 0.1.0.0.4 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.1 "Wind": ring unlimited instances: the first 4 drawn
// effekseer: node 0.1.1 "Wind": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.2.0.0 "Tracker": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.1.2.0.0 "Tracker": nested generation: 64 instances above its generator folded into one emitter
// effekseer: node 0.1.3 "Wind": ring unlimited instances: the first 4 drawn
// effekseer: node 0.1.3 "Wind": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.2.0 "Wind": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.0 "Wind": ring 2 instances: the first 4 drawn
// effekseer: node 0.2.0 "Wind": nested generation: 9 instances above its generator folded into one emitter
// effekseer: node 0.2.1.0.0 "Light": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.1.0.0 "Light": nested generation: 27 instances above its generator folded into one emitter
// effekseer: node 0.2.1.0.1.0.0 "Tracker": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.1.0.1.0.0 "Tracker": nested generation: 54 instances above its generator folded into one emitter
// effekseer: node 0.2.1.0.2 "FireVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.1.0.2 "FireVariation": nested generation: 27 instances above its generator folded into one emitter
// effekseer: node 0.2.1.0.2 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.1.0.3 "FireVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.1.0.3 "FireVariation": nested generation: 27 instances above its generator folded into one emitter
// effekseer: node 0.2.1.0.3 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.1.0.4 "FireVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.1.0.4 "FireVariation": nested generation: 27 instances above its generator folded into one emitter
// effekseer: node 0.2.1.0.4 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.1.0.5 "FireVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.1.0.5 "FireVariation": nested generation: 27 instances above its generator folded into one emitter
// effekseer: node 0.2.1.0.5 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.1.0.6 "SmokeVariation": nested generation: 27 instances above its generator folded into one emitter
// effekseer: node 0.2.1.0 "FireEmitter": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.1.1.0.0 "FireVariation": nested generation: 9 instances above its generator folded into one emitter
// effekseer: node 0.2.1 "MainFireGenerator": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.1.1.0.0 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.1.1.0.1 "FireVariation": nested generation: 9 instances above its generator folded into one emitter
// effekseer: node 0.2.1.1.0.1 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.1.1.0.2 "FireVariation": nested generation: 9 instances above its generator folded into one emitter
// effekseer: node 0.2.1.1.0.2 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.1.1.0.3 "FireVariation": nested generation: 9 instances above its generator folded into one emitter
// effekseer: node 0.2.1.1.0.3 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.1.1.0.4 "EmberVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.1.1.0.4 "EmberVariation": nested generation: 54 instances above its generator folded into one emitter
// effekseer: node 0.2.1.1.0.4 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.1.1.0.5 "EmberVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.1.1.0.5 "EmberVariation": nested generation: 54 instances above its generator folded into one emitter
// effekseer: node 0.2.1.1.0.5 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.1.2.0.0 "EmberVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.1.2.0.0 "EmberVariation": nested generation: 27 instances above its generator folded into one emitter
// effekseer: node 0.2.1.2 "FireGenerator": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.1.2.0.0 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.1.2.0.1 "EmberVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.1.2.0.1 "EmberVariation": nested generation: 27 instances above its generator folded into one emitter
// effekseer: node 0.2.1.2.0.1 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.2 "Flash": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.3 "Flash": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.4 "HeatHaze": a moving ancestor: its descendants are placed where it starts
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.0,
            "band": 0.5,
            "height": [
              -0.3,
              0.3
            ],
            "colors": [
              [
                1.0,
                0.1166,
                0.0104,
                0.1882
              ],
              [
                1.0,
                0.1166,
                0.0104,
                1.0
              ],
              [
                1.0,
                0.1166,
                0.0104,
                0.1882
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              1.0,
              3.0
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          540.0
        ],
        "color": [
          2.25,
          2.25,
          2.25,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        0.0,
        4.0,
        -0.2
      ],
      "rotate": [
        90.0,
        45.0,
        -135.0
      ],
      "velocity": [
        0.0,
        -1.4655,
        5.8237
      ],
      "life": 0.75
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.0,
            "band": 0.5,
            "height": [
              -0.3,
              0.3
            ],
            "colors": [
              [
                1.0,
                0.1166,
                0.0104,
                0.1882
              ],
              [
                1.0,
                0.1166,
                0.0104,
                1.0
              ],
              [
                1.0,
                0.1166,
                0.0104,
                0.1882
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              1.0,
              3.0
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          540.0
        ],
        "color": [
          2.25,
          2.25,
          2.25,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0055,
      "offset": [
        0.0,
        4.0,
        -0.2
      ],
      "rotate": [
        90.0,
        -15.0,
        75.0
      ],
      "velocity": [
        0.0,
        -1.4655,
        5.8237
      ],
      "life": 0.75
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.0,
            "band": 0.5,
            "height": [
              -0.3,
              0.3
            ],
            "colors": [
              [
                1.0,
                0.1166,
                0.0104,
                0.1882
              ],
              [
                1.0,
                0.1166,
                0.0104,
                1.0
              ],
              [
                1.0,
                0.1166,
                0.0104,
                0.1882
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              1.0,
              3.0
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          540.0
        ],
        "color": [
          2.25,
          2.25,
          2.25,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.011,
      "offset": [
        0.0,
        4.0,
        -0.2
      ],
      "rotate": [
        90.0,
        -15.0,
        -75.0
      ],
      "velocity": [
        0.0,
        -1.4655,
        5.8237
      ],
      "life": 0.75
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.0,
            "band": 0.5,
            "height": [
              -0.3,
              0.3
            ],
            "colors": [
              [
                1.0,
                0.1166,
                0.0104,
                0.1882
              ],
              [
                1.0,
                0.1166,
                0.0104,
                1.0
              ],
              [
                1.0,
                0.1166,
                0.0104,
                0.1882
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              1.0,
              3.0
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          540.0
        ],
        "color": [
          2.25,
          2.25,
          2.25,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0165,
      "offset": [
        0.0,
        4.0,
        -0.2
      ],
      "rotate": [
        90.0,
        45.0,
        135.0
      ],
      "velocity": [
        0.0,
        -1.4655,
        5.8237
      ],
      "life": 0.75
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-0-1-0-0.effect"
        },
        "width": [
          0.0,
          0.0
        ],
        "lifetime": 0.0333,
        "color": [
          1.0,
          0.5356,
          0.2195,
          0.0714
        ]
      },
      "at": 0.15,
      "offset": [
        0.0,
        3.7802,
        0.6736
      ],
      "velocity": [
        0.0,
        -1.4655,
        5.8237
      ],
      "life": 1.4833
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-0-2.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        4.0,
        -0.2
      ],
      "velocity": [
        0.0,
        -1.4655,
        5.8237
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-0-3.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        4.5,
        -0.2
      ],
      "velocity": [
        0.0,
        -1.4655,
        5.8237
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-0-4.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        4.5,
        -0.2
      ],
      "velocity": [
        0.0,
        -1.4655,
        5.8237
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-1-0-0-0.emitter"
        }
      },
      "at": 0.15,
      "offset": [
        0.0,
        4.2302,
        0.6736
      ],
      "velocity": [
        0.0,
        -0.8303,
        5.8237
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-1-0-0-1.emitter"
        }
      },
      "at": 0.15,
      "offset": [
        0.0,
        3.0302,
        0.6736
      ],
      "velocity": [
        0.0,
        -0.8303,
        5.8237
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-1-0-0-2.emitter"
        }
      },
      "at": 0.25,
      "offset": [
        0.0,
        2.9471,
        1.2559
      ],
      "velocity": [
        0.0,
        -0.8303,
        5.8237
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-1-0-0-3.emitter"
        }
      },
      "at": 0.35,
      "offset": [
        0.0,
        2.8641,
        1.8383
      ],
      "velocity": [
        0.0,
        -0.8303,
        5.8237
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-1-0-0-4.emitter"
        }
      },
      "at": 0.45,
      "offset": [
        0.0,
        2.5905,
        2.4207
      ],
      "velocity": [
        0.0,
        -1.4655,
        5.8237
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.0,
            "band": 0.5,
            "height": [
              -0.25,
              0.25
            ],
            "colors": [
              [
                1.0,
                0.1166,
                0.0104,
                0.1255
              ],
              [
                1.0,
                0.1166,
                0.0104,
                0.502
              ],
              [
                1.0,
                0.1166,
                0.0104,
                0.1255
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-1-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.75
            ],
            [
              1.0,
              1.65
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          360.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          0.8
        ],
        "blend": "additive"
      },
      "at": 0.15,
      "offset": [
        0.0,
        3.7802,
        0.6736
      ],
      "rotate": [
        90.0,
        45.0,
        -135.0
      ],
      "velocity": [
        0.0,
        -1.4655,
        5.8237
      ],
      "life": 0.45
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.0,
            "band": 0.5,
            "height": [
              -0.25,
              0.25
            ],
            "colors": [
              [
                1.0,
                0.1166,
                0.0104,
                0.1255
              ],
              [
                1.0,
                0.1166,
                0.0104,
                0.502
              ],
              [
                1.0,
                0.1166,
                0.0104,
                0.1255
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-1-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.75
            ],
            [
              1.0,
              1.65
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          360.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          0.8
        ],
        "blend": "additive"
      },
      "at": 0.1583,
      "offset": [
        0.0,
        3.7802,
        0.6736
      ],
      "rotate": [
        90.0,
        -15.0,
        75.0
      ],
      "velocity": [
        0.0,
        -1.4655,
        5.8237
      ],
      "life": 0.45
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.0,
            "band": 0.5,
            "height": [
              -0.25,
              0.25
            ],
            "colors": [
              [
                1.0,
                0.1166,
                0.0104,
                0.1255
              ],
              [
                1.0,
                0.1166,
                0.0104,
                0.502
              ],
              [
                1.0,
                0.1166,
                0.0104,
                0.1255
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-1-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.75
            ],
            [
              1.0,
              1.65
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          360.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          0.8
        ],
        "blend": "additive"
      },
      "at": 0.1667,
      "offset": [
        0.0,
        3.7802,
        0.6736
      ],
      "rotate": [
        90.0,
        -15.0,
        -75.0
      ],
      "velocity": [
        0.0,
        -1.4655,
        5.8237
      ],
      "life": 0.45
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.0,
            "band": 0.5,
            "height": [
              -0.25,
              0.25
            ],
            "colors": [
              [
                1.0,
                0.1166,
                0.0104,
                0.1255
              ],
              [
                1.0,
                0.1166,
                0.0104,
                0.502
              ],
              [
                1.0,
                0.1166,
                0.0104,
                0.1255
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-1-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.75
            ],
            [
              1.0,
              1.65
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          360.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          0.8
        ],
        "blend": "additive"
      },
      "at": 0.175,
      "offset": [
        0.0,
        3.7802,
        0.6736
      ],
      "rotate": [
        90.0,
        45.0,
        135.0
      ],
      "velocity": [
        0.0,
        -1.4655,
        5.8237
      ],
      "life": 0.45
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-1-2-0-0.effect"
        },
        "width": [
          0.0,
          0.0
        ],
        "lifetime": 0.0333,
        "color": [
          1.0,
          0.5356,
          0.2195,
          0.0714
        ]
      },
      "at": 0.3125,
      "offset": [
        0.0,
        3.542,
        1.6199
      ],
      "velocity": [
        0.0,
        -1.4655,
        5.8237
      ],
      "life": 1.4833
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.0,
            "band": 0.5,
            "height": [
              -0.3,
              0.3
            ],
            "colors": [
              [
                1.0,
                0.2195,
                0.0254,
                0.251
              ],
              [
                1.0,
                0.2195,
                0.0254,
                1.0
              ],
              [
                1.0,
                0.2195,
                0.0254,
                0.251
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-1-3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              1.0,
              4.5
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          360.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          0.8
        ],
        "blend": "additive"
      },
      "at": 0.9,
      "offset": [
        0.0,
        2.681,
        5.0413
      ],
      "rotate": [
        90.0,
        45.0,
        -135.0
      ],
      "velocity": [
        0.0,
        -1.4655,
        5.8237
      ],
      "life": 0.45
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.0,
            "band": 0.5,
            "height": [
              -0.3,
              0.3
            ],
            "colors": [
              [
                1.0,
                0.2195,
                0.0254,
                0.251
              ],
              [
                1.0,
                0.2195,
                0.0254,
                1.0
              ],
              [
                1.0,
                0.2195,
                0.0254,
                0.251
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-1-3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              1.0,
              4.5
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          360.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          0.8
        ],
        "blend": "additive"
      },
      "at": 1.05,
      "offset": [
        0.0,
        2.681,
        5.0413
      ],
      "rotate": [
        90.0,
        -15.0,
        75.0
      ],
      "velocity": [
        0.0,
        -1.4655,
        5.8237
      ],
      "life": 0.45
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.0,
            "band": 0.5,
            "height": [
              -0.3,
              0.3
            ],
            "colors": [
              [
                1.0,
                0.2195,
                0.0254,
                0.251
              ],
              [
                1.0,
                0.2195,
                0.0254,
                1.0
              ],
              [
                1.0,
                0.2195,
                0.0254,
                0.251
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-1-3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              1.0,
              4.5
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          360.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          0.8
        ],
        "blend": "additive"
      },
      "at": 1.2,
      "offset": [
        0.0,
        2.681,
        5.0413
      ],
      "rotate": [
        90.0,
        -15.0,
        -75.0
      ],
      "velocity": [
        0.0,
        -1.4655,
        5.8237
      ],
      "life": 0.45
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.0,
            "band": 0.5,
            "height": [
              -0.3,
              0.3
            ],
            "colors": [
              [
                1.0,
                0.2195,
                0.0254,
                0.251
              ],
              [
                1.0,
                0.2195,
                0.0254,
                1.0
              ],
              [
                1.0,
                0.2195,
                0.0254,
                0.251
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-1-3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              1.0,
              4.5
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          360.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          0.8
        ],
        "blend": "additive"
      },
      "at": 1.35,
      "offset": [
        0.0,
        2.681,
        5.0413
      ],
      "rotate": [
        90.0,
        45.0,
        135.0
      ],
      "velocity": [
        0.0,
        -1.4655,
        5.8237
      ],
      "life": 0.45
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.0,
            "band": 0.5,
            "height": [
              -0.25,
              0.25
            ],
            "colors": [
              [
                1.0,
                0.1166,
                0.0104,
                0.251
              ],
              [
                1.0,
                0.1166,
                0.0104,
                0.7529
              ],
              [
                1.0,
                0.1166,
                0.0104,
                0.251
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-2-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              1.0,
              15.0
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          360.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          0.8
        ],
        "blend": "additive"
      },
      "at": 2.5333,
      "offset": [
        0.0,
        4.0,
        -0.2
      ],
      "rotate": [
        105.0,
        0.0,
        165.0
      ],
      "life": 0.45
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.0,
            "band": 0.5,
            "height": [
              -0.25,
              0.25
            ],
            "colors": [
              [
                1.0,
                0.1166,
                0.0104,
                0.251
              ],
              [
                1.0,
                0.1166,
                0.0104,
                0.7529
              ],
              [
                1.0,
                0.1166,
                0.0104,
                0.251
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-2-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              1.0,
              15.0
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          360.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          0.8
        ],
        "blend": "additive"
      },
      "at": 2.5388,
      "offset": [
        0.0,
        4.0,
        -0.2
      ],
      "rotate": [
        75.0,
        0.0,
        -165.0
      ],
      "life": 0.45
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-2-1-0-0.emitter"
        }
      },
      "at": 2.7,
      "offset": [
        0.0,
        4.0,
        1.3
      ],
      "rotate": [
        11.5346,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-2-1-0-1-0-0.effect"
        },
        "width": [
          0.0,
          0.0
        ],
        "lifetime": 0.0333,
        "color": [
          1.0,
          0.5356,
          0.2195,
          0.0714
        ]
      },
      "at": 2.85,
      "offset": [
        0.0,
        4.0,
        1.3
      ],
      "rotate": [
        25.0,
        90.0,
        0.0
      ],
      "life": 1.4833
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-2-1-0-2.emitter"
        }
      },
      "at": 2.7,
      "offset": [
        0.0,
        4.0,
        1.3
      ],
      "rotate": [
        11.5346,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-2-1-0-3.emitter"
        }
      },
      "at": 2.8333,
      "offset": [
        0.0,
        4.0,
        1.3
      ],
      "rotate": [
        11.5346,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-2-1-0-4.emitter"
        }
      },
      "at": 2.9667,
      "offset": [
        0.0,
        4.0,
        1.3
      ],
      "rotate": [
        11.5346,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-2-1-0-5.emitter"
        }
      },
      "at": 3.1,
      "offset": [
        0.0,
        4.0,
        1.3
      ],
      "rotate": [
        11.5346,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-2-1-0-6.parent"
        }
      },
      "at": 2.7,
      "offset": [
        0.0,
        4.0,
        1.3
      ],
      "rotate": [
        14.0362,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-2-1-1-0-0.parent"
        }
      },
      "at": 2.5,
      "offset": [
        0.0,
        4.0,
        -0.2
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-2-1-1-0-1.parent"
        }
      },
      "at": 2.5,
      "offset": [
        0.0,
        4.0,
        -0.2
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-2-1-1-0-2.parent"
        }
      },
      "at": 2.5,
      "offset": [
        0.0,
        4.0,
        -0.2
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-2-1-1-0-3.parent"
        }
      },
      "at": 2.5,
      "offset": [
        0.0,
        4.0,
        -0.2
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-2-1-1-0-4.emitter"
        }
      },
      "at": 3.1042,
      "offset": [
        0.0,
        6.7245,
        3.691
      ],
      "rotate": [
        55.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-2-1-1-0-5.emitter"
        }
      },
      "at": 3.2042,
      "offset": [
        0.0,
        6.7245,
        3.691
      ],
      "rotate": [
        55.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-2-1-2-0-0.parent"
        }
      },
      "at": 3.0167,
      "offset": [
        0.0,
        4.0,
        -0.2
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-2-1-2-0-1.parent"
        }
      },
      "at": 3.0167,
      "offset": [
        0.0,
        4.0,
        -0.2
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-2-2.emitter"
        }
      },
      "at": 2.5,
      "offset": [
        0.0,
        4.0,
        -0.2
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-2-3.emitter"
        }
      },
      "at": 2.5,
      "offset": [
        0.0,
        4.0,
        -0.2
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-2-4.emitter"
        }
      },
      "at": 2.75,
      "offset": [
        0.0,
        4.0,
        -0.2
      ]
    }
  ],
  "duration": 5.5167,
  "events": [
    {
      "name": "impact",
      "time": 2.5
    }
  ]
});
