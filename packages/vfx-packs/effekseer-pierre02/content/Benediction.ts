import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.1 "Ray": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.1 "Ray": nested generation: 12 instances above its generator folded into one emitter
// effekseer: node 0.1.0.0 "Wind": ring 3 instances: the first 4 drawn
// effekseer: node 0.1.0.0 "Wind": nested generation: 21 instances above its generator folded into one emitter
// effekseer: node 0.1.0.1.0.0 "FireVariation": nested generation: 42 instances above its generator folded into one emitter
// effekseer: node 0.1.0.1.0.0 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.0.1.0.1 "FireVariation": nested generation: 42 instances above its generator folded into one emitter
// effekseer: node 0.1.0.1.0.1 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.0.1.0.2 "FireVariation": nested generation: 42 instances above its generator folded into one emitter
// effekseer: node 0.1.0.1.0.2 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.0.1.0.3 "FireVariation": nested generation: 42 instances above its generator folded into one emitter
// effekseer: node 0.1.0.1.0.3 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.0.1.1.0.0 "FireVariation": nested generation: 21 instances above its generator folded into one emitter
// effekseer: node 0.1.0.1.1.0.0 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.0.1.1.0.1 "FireVariation": nested generation: 21 instances above its generator folded into one emitter
// effekseer: node 0.1.0.1.1.0.1 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.0.1.1.0.2 "FireVariation": nested generation: 21 instances above its generator folded into one emitter
// effekseer: node 0.1.0.1.1.0.2 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.0.1.1.0.3 "FireVariation": nested generation: 21 instances above its generator folded into one emitter
// effekseer: node 0.1.0.1.1.0.3 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.0.1.1.0.4 "EmberVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.1.0.1.1.0.4 "EmberVariation": nested generation: 84 instances above its generator folded into one emitter
// effekseer: node 0.1.0.1.1.0.4 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.0.1.1.0.5 "EmberVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.1.0.1.1.0.5 "EmberVariation": nested generation: 84 instances above its generator folded into one emitter
// effekseer: node 0.1.0.1.1.0.5 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.0.1.2.0.0 "EmberVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.1.0.1.2.0.0 "EmberVariation": nested generation: 42 instances above its generator folded into one emitter
// effekseer: node 0.1.0.1.2.0.0 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.0.1.2.0.1 "EmberVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.1.0.1.2.0.1 "EmberVariation": nested generation: 42 instances above its generator folded into one emitter
// effekseer: node 0.1.0.1.2.0.1 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.1.0.0 "Tracker": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.1.1.0.0 "Tracker": nested generation: 21 instances above its generator folded into one emitter
// effekseer: node 0.1.1.0.1 "Light": a child of an instanced parent: its motion drawn up the world's y
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Benediction/n0-0-0-0.parent"
        }
      },
      "at": 0.75,
      "offset": [
        0.0,
        0.0,
        6.75
      ],
      "rotate": [
        45.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Benediction/n0-0-0-0-0.parent"
        }
      },
      "at": 0.75,
      "offset": [
        0.0,
        0.0,
        6.75
      ],
      "rotate": [
        45.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Benediction/n0-0-0-1.emitter"
        }
      },
      "at": 0.75,
      "offset": [
        0.0,
        3.375,
        6.75
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
                0.4647,
                0.0356,
                0.1255
              ],
              [
                1.0,
                0.4647,
                0.0356,
                0.3765
              ],
              [
                1.0,
                0.4647,
                0.0356,
                0.1255
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/Benediction/n0-1-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.2656
            ],
            [
              1.0,
              12.6562
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
          0.9333
        ],
        "blend": "additive"
      },
      "at": 0.2167,
      "rotate": [
        100.3141,
        17.2294,
        -141.5667
      ],
      "life": 0.5167
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
                0.4647,
                0.0356,
                0.1255
              ],
              [
                1.0,
                0.4647,
                0.0356,
                0.3765
              ],
              [
                1.0,
                0.4647,
                0.0356,
                0.1255
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/Benediction/n0-1-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.2656
            ],
            [
              1.0,
              12.6562
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
          0.9333
        ],
        "blend": "additive"
      },
      "at": 0.2222,
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "life": 0.5167
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
                0.4647,
                0.0356,
                0.1255
              ],
              [
                1.0,
                0.4647,
                0.0356,
                0.3765
              ],
              [
                1.0,
                0.4647,
                0.0356,
                0.1255
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/Benediction/n0-1-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.2656
            ],
            [
              1.0,
              12.6562
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
          0.9333
        ],
        "blend": "additive"
      },
      "at": 0.2277,
      "rotate": [
        79.6859,
        17.2294,
        141.5667
      ],
      "life": 0.5167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Benediction/n0-1-0-1-0-0.emitter"
        }
      },
      "at": 0.35,
      "offset": [
        0.0,
        0.0,
        1.2656
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
          "$ogRef": "base/vfx/effekseer-pierre02/Benediction/n0-1-0-1-0-1.emitter"
        }
      },
      "at": 0.4833,
      "offset": [
        0.0,
        0.0,
        1.2656
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
          "$ogRef": "base/vfx/effekseer-pierre02/Benediction/n0-1-0-1-0-2.emitter"
        }
      },
      "at": 0.6167,
      "offset": [
        0.0,
        0.0,
        1.2656
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
          "$ogRef": "base/vfx/effekseer-pierre02/Benediction/n0-1-0-1-0-3.emitter"
        }
      },
      "at": 0.75,
      "offset": [
        0.0,
        0.0,
        1.2656
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
          "$ogRef": "base/vfx/effekseer-pierre02/Benediction/n0-1-0-1-1-0-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Benediction/n0-1-0-1-1-0-1.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Benediction/n0-1-0-1-1-0-2.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Benediction/n0-1-0-1-1-0-3.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Benediction/n0-1-0-1-1-0-4.emitter"
        }
      },
      "at": 0.6042,
      "offset": [
        0.0,
        2.2988,
        3.283
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
          "$ogRef": "base/vfx/effekseer-pierre02/Benediction/n0-1-0-1-1-0-5.emitter"
        }
      },
      "at": 0.7042,
      "offset": [
        0.0,
        2.2988,
        3.283
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
          "$ogRef": "base/vfx/effekseer-pierre02/Benediction/n0-1-0-1-2-0-0.parent"
        }
      },
      "at": 0.5167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Benediction/n0-1-0-1-2-0-1.parent"
        }
      },
      "at": 0.5167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Benediction/n0-1-0-2.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/Benediction/n0-1-1-0-0.effect"
        },
        "width": [
          0.0,
          0.0
        ],
        "lifetime": 0.0167,
        "color": [
          1.0,
          0.4647,
          0.0356,
          0.0732
        ],
        "orbit": [
          0.0,
          0.0682
        ]
      },
      "at": 0.4167,
      "rotate": [
        0.0,
        89.9802,
        0.0
      ],
      "life": 1.3167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Benediction/n0-1-1-0-1.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Benediction/n0-1-2.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Benediction/n0-1-3.emitter"
        }
      },
      "at": 0.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Benediction/n0-1-4.emitter"
        }
      },
      "at": 0.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Benediction/n0-1-5.emitter"
        }
      },
      "at": 0.1667
    }
  ],
  "duration": 5.9333,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
