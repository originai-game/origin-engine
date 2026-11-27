import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0 "Wind": ring 3 instances: the first 4 drawn
// effekseer: node 0.0 "Wind": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.1.2.0.0 "SparkEmitter": nested generation: 26 instances above its generator folded into one emitter
// effekseer: node 0.1.2.0.0 "SparkEmitter": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.2.0.0 "SparkEmitter": a child of an instanced parent: its motion drawn up the world's y
// effekseer: node 0.1.2.1 "FireVariation": nested generation: 26 instances above its generator folded into one emitter
// effekseer: node 0.1.2.1 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.2.2 "FireVariation": nested generation: 26 instances above its generator folded into one emitter
// effekseer: node 0.1.2.2 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.2.3 "FireVariation": nested generation: 26 instances above its generator folded into one emitter
// effekseer: node 0.1.2.3 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.2.4 "FireVariation": nested generation: 26 instances above its generator folded into one emitter
// effekseer: node 0.1.2.4 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.3.0.0 "FireVariation": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.1.3.0.0 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.3.0.1 "FireVariation": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.1.3.0.1 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.3.0.2 "FireVariation": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.1.3.0.2 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.3.0.3 "FireVariation": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.1.3.0.3 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.3.0.4 "EmberVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.1.3.0.4 "EmberVariation": nested generation: 52 instances above its generator folded into one emitter
// effekseer: node 0.1.3.0.4 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.3.0.5 "EmberVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.1.3.0.5 "EmberVariation": nested generation: 52 instances above its generator folded into one emitter
// effekseer: node 0.1.3.0.5 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.4.0.0 "EmberVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.1.4.0.0 "EmberVariation": nested generation: 26 instances above its generator folded into one emitter
// effekseer: node 0.1.4.0.0 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.4.0.1 "EmberVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.1.4.0.1 "EmberVariation": nested generation: 26 instances above its generator folded into one emitter
// effekseer: node 0.1.4.0.1 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
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
              -0.25,
              0.25
            ],
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.1882
              ],
              [
                1.0,
                1.0,
                1.0,
                0.7529
              ],
              [
                1.0,
                1.0,
                1.0,
                0.1882
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/FeatherBomb/n0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.125
            ],
            [
              1.0,
              11.25
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
        "blend": "alpha"
      },
      "at": 0.0833,
      "rotate": [
        100.3141,
        17.2294,
        -141.5667
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
                1.0,
                1.0,
                0.1882
              ],
              [
                1.0,
                1.0,
                1.0,
                0.7529
              ],
              [
                1.0,
                1.0,
                1.0,
                0.1882
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/FeatherBomb/n0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.125
            ],
            [
              1.0,
              11.25
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
        "blend": "alpha"
      },
      "at": 0.0888,
      "rotate": [
        90.0,
        0.0,
        0.0
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
                1.0,
                1.0,
                0.1882
              ],
              [
                1.0,
                1.0,
                1.0,
                0.7529
              ],
              [
                1.0,
                1.0,
                1.0,
                0.1882
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/FeatherBomb/n0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.125
            ],
            [
              1.0,
              11.25
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
        "blend": "alpha"
      },
      "at": 0.0943,
      "rotate": [
        79.6859,
        17.2294,
        141.5667
      ],
      "life": 0.45
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FeatherBomb/n0-1-0-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FeatherBomb/n0-1-1-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FeatherBomb/n0-1-2-0-0.parent"
        }
      },
      "at": 0.35,
      "offset": [
        0.0,
        0.0,
        1.125
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
          "$ogRef": "base/vfx/effekseer-pierre02/FeatherBomb/n0-1-2-1.emitter"
        }
      },
      "at": 0.35,
      "offset": [
        0.0,
        0.0,
        1.125
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
          "$ogRef": "base/vfx/effekseer-pierre02/FeatherBomb/n0-1-2-2.emitter"
        }
      },
      "at": 0.4833,
      "offset": [
        0.0,
        0.0,
        1.125
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
          "$ogRef": "base/vfx/effekseer-pierre02/FeatherBomb/n0-1-2-3.emitter"
        }
      },
      "at": 0.6167,
      "offset": [
        0.0,
        0.0,
        1.125
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
          "$ogRef": "base/vfx/effekseer-pierre02/FeatherBomb/n0-1-2-4.emitter"
        }
      },
      "at": 0.75,
      "offset": [
        0.0,
        0.0,
        1.125
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
          "$ogRef": "base/vfx/effekseer-pierre02/FeatherBomb/n0-1-3-0-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FeatherBomb/n0-1-3-0-1.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FeatherBomb/n0-1-3-0-2.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FeatherBomb/n0-1-3-0-3.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FeatherBomb/n0-1-3-0-4.emitter"
        }
      },
      "at": 0.6042,
      "offset": [
        0.0,
        2.0434,
        2.9182
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
          "$ogRef": "base/vfx/effekseer-pierre02/FeatherBomb/n0-1-3-0-5.emitter"
        }
      },
      "at": 0.7042,
      "offset": [
        0.0,
        2.0434,
        2.9182
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
          "$ogRef": "base/vfx/effekseer-pierre02/FeatherBomb/n0-1-4-0-0.parent"
        }
      },
      "at": 0.5167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FeatherBomb/n0-1-4-0-1.parent"
        }
      },
      "at": 0.5167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/FeatherBomb/n0-2.emitter"
        }
      },
      "at": 0.0
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
  "duration": 13.006,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
