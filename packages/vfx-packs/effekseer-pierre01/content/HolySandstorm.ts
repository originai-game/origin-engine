import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0 "Smoke": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.0 "Smoke": nested generation: 2 instances above its generator folded into one emitter
// effekseer: node 0.0.0.1 "Wind_Ring": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.1 "Wind_Ring": ring 240 instances: the first 4 drawn
// effekseer: node 0.0.0.1 "Wind_Ring": nested generation: 2 instances above its generator folded into one emitter
// effekseer: node 0.0.0.1 "Wind_Ring": a ring whose height changes over its life: its middle height kept
// effekseer: node 0.0.0.2 "Wind_Sprite": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.2 "Wind_Sprite": nested generation: 2 instances above its generator folded into one emitter
// effekseer: node 0.0.0.3 "Particle": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.3 "Particle": nested generation: 2 instances above its generator folded into one emitter
// effekseer: node 0.0.1.0 "Wind_Ring": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.1.0 "Wind_Ring": ring 15 instances: the first 4 drawn
// effekseer: node 0.0.1.0 "Wind_Ring": nested generation: 2 instances above its generator folded into one emitter
// effekseer: node 0.0.1.0 "Wind_Ring": a ring whose height changes over its life: its middle height kept
// effekseer: node 0.0.1.1 "Wind_Sprite": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.1.1 "Wind_Sprite": nested generation: 2 instances above its generator folded into one emitter
// effekseer: node 0.0.1.2 "Distortion": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.1.2 "Distortion": ring 10 instances: the first 4 drawn
// effekseer: node 0.0.1.2 "Distortion": nested generation: 2 instances above its generator folded into one emitter
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/HolySandstorm/n0-0-0-0.emitter"
        }
      },
      "at": 1.25,
      "offset": [
        1.7412,
        3.0,
        1.0341
      ],
      "rotate": [
        -90.0,
        0.0,
        -165.0
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
              0.0,
              0.5536
            ],
            "colors": [
              [
                0.5356,
                0.4423,
                0.2195,
                1.0
              ],
              [
                0.5356,
                0.4423,
                0.2195,
                1.0
              ],
              [
                0.5356,
                0.4423,
                0.2195,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/HolySandstorm/n0-0-0-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.2344,
              3.002
            ],
            [
              0.4688,
              5.1558
            ],
            [
              0.7188,
              8.0751
            ],
            [
              1.0,
              12.0
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 1.25,
      "offset": [
        2.0,
        3.0,
        2.0
      ],
      "rotate": [
        -180.0,
        -15.0,
        -90.0
      ],
      "life": 1.4167
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
              0.0,
              0.5536
            ],
            "colors": [
              [
                0.5356,
                0.4423,
                0.2195,
                1.0
              ],
              [
                0.5356,
                0.4423,
                0.2195,
                1.0
              ],
              [
                0.5356,
                0.4423,
                0.2195,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/HolySandstorm/n0-0-0-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.2344,
              3.002
            ],
            [
              0.4688,
              5.1558
            ],
            [
              0.7188,
              8.0751
            ],
            [
              1.0,
              12.0
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 1.2667,
      "offset": [
        2.0,
        3.0,
        2.0
      ],
      "rotate": [
        -180.0,
        -15.0,
        90.0
      ],
      "life": 1.4167
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
              0.0,
              0.5536
            ],
            "colors": [
              [
                0.5356,
                0.4423,
                0.2195,
                1.0
              ],
              [
                0.5356,
                0.4423,
                0.2195,
                1.0
              ],
              [
                0.5356,
                0.4423,
                0.2195,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/HolySandstorm/n0-0-0-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.2344,
              3.002
            ],
            [
              0.4688,
              5.1558
            ],
            [
              0.7188,
              8.0751
            ],
            [
              1.0,
              12.0
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 1.2833,
      "offset": [
        2.0,
        3.0,
        2.0
      ],
      "rotate": [
        -180.0,
        -15.0,
        -90.0
      ],
      "life": 1.4167
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
              0.0,
              0.5536
            ],
            "colors": [
              [
                0.5356,
                0.4423,
                0.2195,
                1.0
              ],
              [
                0.5356,
                0.4423,
                0.2195,
                1.0
              ],
              [
                0.5356,
                0.4423,
                0.2195,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/HolySandstorm/n0-0-0-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.2344,
              3.002
            ],
            [
              0.4688,
              5.1558
            ],
            [
              0.7188,
              8.0751
            ],
            [
              1.0,
              12.0
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 1.3,
      "offset": [
        2.0,
        3.0,
        2.0
      ],
      "rotate": [
        -180.0,
        -15.0,
        90.0
      ],
      "life": 1.4167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/HolySandstorm/n0-0-0-2.emitter"
        }
      },
      "at": 1.25,
      "offset": [
        1.7412,
        3.0,
        1.0341
      ],
      "rotate": [
        90.0,
        0.0,
        165.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/HolySandstorm/n0-0-0-3.emitter"
        }
      },
      "at": 1.25,
      "offset": [
        -4.0544,
        3.0,
        2.587
      ],
      "rotate": [
        -90.0,
        0.0,
        -165.0
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
              1.5805,
              0.0
            ],
            "colors": [
              [
                0.7519,
                0.6388,
                0.4423,
                0.251
              ],
              [
                0.7519,
                0.6388,
                0.4423,
                0.251
              ],
              [
                0.7519,
                0.6388,
                0.4423,
                0.251
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/HolySandstorm/n0-0-1-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.5
            ],
            [
              0.1094,
              1.7659
            ],
            [
              0.2344,
              1.0455
            ],
            [
              0.3438,
              0.519
            ],
            [
              0.4688,
              0.036
            ],
            [
              0.75,
              0.5881
            ],
            [
              0.875,
              0.6598
            ],
            [
              1.0,
              0.605
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.0333,
      "offset": [
        2.0,
        3.0,
        2.0
      ],
      "rotate": [
        -180.0,
        -15.0,
        -90.0
      ],
      "life": 1.5
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
              1.5805,
              0.0
            ],
            "colors": [
              [
                0.7519,
                0.6388,
                0.4423,
                0.251
              ],
              [
                0.7519,
                0.6388,
                0.4423,
                0.251
              ],
              [
                0.7519,
                0.6388,
                0.4423,
                0.251
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/HolySandstorm/n0-0-1-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.5
            ],
            [
              0.1094,
              1.7659
            ],
            [
              0.2344,
              1.0455
            ],
            [
              0.3438,
              0.519
            ],
            [
              0.4688,
              0.036
            ],
            [
              0.75,
              0.5881
            ],
            [
              0.875,
              0.6598
            ],
            [
              1.0,
              0.605
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.0833,
      "offset": [
        2.0,
        3.0,
        2.0
      ],
      "rotate": [
        -180.0,
        -15.0,
        90.0
      ],
      "life": 1.5
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
              1.5805,
              0.0
            ],
            "colors": [
              [
                0.7519,
                0.6388,
                0.4423,
                0.251
              ],
              [
                0.7519,
                0.6388,
                0.4423,
                0.251
              ],
              [
                0.7519,
                0.6388,
                0.4423,
                0.251
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/HolySandstorm/n0-0-1-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.5
            ],
            [
              0.1094,
              1.7659
            ],
            [
              0.2344,
              1.0455
            ],
            [
              0.3438,
              0.519
            ],
            [
              0.4688,
              0.036
            ],
            [
              0.75,
              0.5881
            ],
            [
              0.875,
              0.6598
            ],
            [
              1.0,
              0.605
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.1333,
      "offset": [
        2.0,
        3.0,
        2.0
      ],
      "rotate": [
        -180.0,
        -15.0,
        -90.0
      ],
      "life": 1.5
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
              1.5805,
              0.0
            ],
            "colors": [
              [
                0.7519,
                0.6388,
                0.4423,
                0.251
              ],
              [
                0.7519,
                0.6388,
                0.4423,
                0.251
              ],
              [
                0.7519,
                0.6388,
                0.4423,
                0.251
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/HolySandstorm/n0-0-1-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.5
            ],
            [
              0.1094,
              1.7659
            ],
            [
              0.2344,
              1.0455
            ],
            [
              0.3438,
              0.519
            ],
            [
              0.4688,
              0.036
            ],
            [
              0.75,
              0.5881
            ],
            [
              0.875,
              0.6598
            ],
            [
              1.0,
              0.605
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.1833,
      "offset": [
        2.0,
        3.0,
        2.0
      ],
      "rotate": [
        -180.0,
        -15.0,
        90.0
      ],
      "life": 1.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/HolySandstorm/n0-0-1-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        1.6894,
        3.0,
        0.8409
      ],
      "rotate": [
        -90.0,
        0.0,
        -165.0
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
              0.0,
              2.0
            ],
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
                1.0
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
          "$ogRef": "base/vfx/effekseer-pierre01/HolySandstorm/n0-0-1-2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.02
            ],
            [
              0.125,
              0.6197
            ],
            [
              0.25,
              1.1725
            ],
            [
              0.375,
              1.6784
            ],
            [
              0.5,
              2.1375
            ],
            [
              0.625,
              2.5497
            ],
            [
              0.75,
              2.915
            ],
            [
              1.0,
              3.505
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          120.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.0,
      "offset": [
        2.0026,
        3.0,
        2.0097
      ],
      "rotate": [
        180.0,
        -15.0,
        45.0
      ],
      "velocity": [
        0.4659,
        0.0,
        1.7387
      ],
      "life": 1.6667
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
              0.0,
              2.0
            ],
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
                1.0
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
          "$ogRef": "base/vfx/effekseer-pierre01/HolySandstorm/n0-0-1-2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.02
            ],
            [
              0.125,
              0.6197
            ],
            [
              0.25,
              1.1725
            ],
            [
              0.375,
              1.6784
            ],
            [
              0.5,
              2.1375
            ],
            [
              0.625,
              2.5497
            ],
            [
              0.75,
              2.915
            ],
            [
              1.0,
              3.505
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          120.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.2,
      "offset": [
        2.0026,
        3.0,
        2.0097
      ],
      "rotate": [
        -180.0,
        -15.0,
        135.0
      ],
      "velocity": [
        0.4659,
        0.0,
        1.7387
      ],
      "life": 1.6667
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
              0.0,
              2.0
            ],
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
                1.0
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
          "$ogRef": "base/vfx/effekseer-pierre01/HolySandstorm/n0-0-1-2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.02
            ],
            [
              0.125,
              0.6197
            ],
            [
              0.25,
              1.1725
            ],
            [
              0.375,
              1.6784
            ],
            [
              0.5,
              2.1375
            ],
            [
              0.625,
              2.5497
            ],
            [
              0.75,
              2.915
            ],
            [
              1.0,
              3.505
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          120.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.4,
      "offset": [
        2.0026,
        3.0,
        2.0097
      ],
      "rotate": [
        180.0,
        -15.0,
        -135.0
      ],
      "velocity": [
        0.4659,
        0.0,
        1.7387
      ],
      "life": 1.6667
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
              0.0,
              2.0
            ],
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
                1.0
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
          "$ogRef": "base/vfx/effekseer-pierre01/HolySandstorm/n0-0-1-2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.02
            ],
            [
              0.125,
              0.6197
            ],
            [
              0.25,
              1.1725
            ],
            [
              0.375,
              1.6784
            ],
            [
              0.5,
              2.1375
            ],
            [
              0.625,
              2.5497
            ],
            [
              0.75,
              2.915
            ],
            [
              1.0,
              3.505
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          120.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.6,
      "offset": [
        2.0026,
        3.0,
        2.0097
      ],
      "rotate": [
        -180.0,
        -15.0,
        -45.0
      ],
      "velocity": [
        0.4659,
        0.0,
        1.7387
      ],
      "life": 1.6667
    }
  ],
  "duration": 6.7334,
  "events": [
    {
      "name": "impact",
      "time": 1.25
    }
  ]
});
