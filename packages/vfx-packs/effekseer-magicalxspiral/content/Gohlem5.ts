import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 3.2 "Node": model 10 instances: the first 4 drawn
// effekseer: node 3.3 "Node": model 10 instances: the first 4 drawn
// effekseer: node 3.4 "Node": a y-axis ring: drawn in its own plane
// effekseer: node 3.5 "Node": a y-axis ring: drawn in its own plane
// effekseer: node 3.6 "Node": a y-axis ring: drawn in its own plane
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/n0-0.effect"
        },
        "width": [
          2.5,
          0.625
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          1.0,
          0.5356,
          0.4183
        ]
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.75,
        0.0
      ],
      "rotate": [
        90.0,
        89.972,
        0.0
      ],
      "life": 0.65
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/n0-1.effect"
        },
        "width": [
          2.5,
          0.625
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          1.0,
          0.0,
          0.8366
        ]
      },
      "at": 0.0833,
      "offset": [
        0.0,
        0.75,
        0.0
      ],
      "rotate": [
        90.0,
        89.972,
        0.0
      ],
      "life": 0.65
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/n1-0.effect"
        },
        "width": [
          2.5,
          0.625
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          1.0,
          0.5356,
          0.4183
        ]
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.75,
        0.0
      ],
      "rotate": [
        90.0,
        75.0,
        0.0
      ],
      "life": 0.65
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/n2-0.effect"
        },
        "width": [
          2.5,
          0.625
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          1.0,
          0.5356,
          0.4183
        ]
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.75,
        0.0
      ],
      "rotate": [
        -90.0,
        75.0,
        -180.0
      ],
      "life": 0.65
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/n3-0.emitter"
        }
      },
      "at": 0.1667,
      "offset": [
        0.0,
        0.25,
        2.0
      ],
      "rotate": [
        0.0,
        -89.9802,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/n3-1.emitter"
        }
      },
      "at": 0.1667,
      "offset": [
        0.0,
        0.25,
        2.0
      ],
      "rotate": [
        0.0,
        -89.9802,
        0.0
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/model-block2"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/n3-2.effect"
        },
        "scale": 1.5,
        "color": [
          1.0,
          1.0,
          1.0,
          0.8784
        ],
        "blend": "alpha"
      },
      "at": 0.1667,
      "offset": [
        0.0,
        0.25,
        2.0
      ],
      "rotate": [
        0.0,
        -89.9802,
        0.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/model-block2"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/n3-2.effect"
        },
        "scale": 1.5,
        "color": [
          1.0,
          1.0,
          1.0,
          0.8784
        ],
        "blend": "alpha"
      },
      "at": 0.1833,
      "offset": [
        0.0,
        0.25,
        2.0
      ],
      "rotate": [
        0.0,
        -89.9802,
        0.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/model-block2"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/n3-2.effect"
        },
        "scale": 1.5,
        "color": [
          1.0,
          1.0,
          1.0,
          0.8784
        ],
        "blend": "alpha"
      },
      "at": 0.2,
      "offset": [
        0.0,
        0.25,
        2.0
      ],
      "rotate": [
        0.0,
        -89.9802,
        0.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/model-block2"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/n3-2.effect"
        },
        "scale": 1.5,
        "color": [
          1.0,
          1.0,
          1.0,
          0.8784
        ],
        "blend": "alpha"
      },
      "at": 0.2167,
      "offset": [
        0.0,
        0.25,
        2.0
      ],
      "rotate": [
        0.0,
        -89.9802,
        0.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/model-block3"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/n3-3.effect"
        },
        "scale": 1.5,
        "color": [
          1.0,
          1.0,
          1.0,
          0.8784
        ],
        "blend": "alpha"
      },
      "at": 0.1667,
      "offset": [
        0.0,
        0.25,
        2.0
      ],
      "rotate": [
        0.0,
        -89.9802,
        0.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/model-block3"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/n3-3.effect"
        },
        "scale": 1.5,
        "color": [
          1.0,
          1.0,
          1.0,
          0.8784
        ],
        "blend": "alpha"
      },
      "at": 0.1833,
      "offset": [
        0.0,
        0.25,
        2.0
      ],
      "rotate": [
        0.0,
        -89.9802,
        0.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/model-block3"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/n3-3.effect"
        },
        "scale": 1.5,
        "color": [
          1.0,
          1.0,
          1.0,
          0.8784
        ],
        "blend": "alpha"
      },
      "at": 0.2,
      "offset": [
        0.0,
        0.25,
        2.0
      ],
      "rotate": [
        0.0,
        -89.9802,
        0.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/model-block3"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/n3-3.effect"
        },
        "scale": 1.5,
        "color": [
          1.0,
          1.0,
          1.0,
          0.8784
        ],
        "blend": "alpha"
      },
      "at": 0.2167,
      "offset": [
        0.0,
        0.25,
        2.0
      ],
      "rotate": [
        0.0,
        -89.9802,
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
            "inner": 0.5,
            "band": 0.5,
            "colors": [
              [
                1.0,
                0.5356,
                0.0,
                0.0
              ],
              [
                1.0,
                0.5356,
                0.0,
                1.0
              ],
              [
                1.0,
                1.0,
                0.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/n3-4.effect"
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
              10.0
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
      "offset": [
        0.0,
        0.25,
        2.0
      ],
      "rotate": [
        90.0,
        0.0,
        90.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.5,
            "colors": [
              [
                1.0,
                0.5356,
                0.0,
                0.0
              ],
              [
                1.0,
                0.5356,
                0.0,
                1.0
              ],
              [
                1.0,
                1.0,
                0.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/n3-5.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              0.1719,
              1.4388
            ],
            [
              0.5312,
              3.7984
            ],
            [
              0.6719,
              5.0235
            ],
            [
              0.7656,
              6.0839
            ],
            [
              0.8438,
              7.1644
            ],
            [
              0.9219,
              8.4589
            ],
            [
              1.0,
              10.0
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
      "at": 0.2167,
      "offset": [
        0.0,
        0.25,
        2.0
      ],
      "rotate": [
        90.0,
        -45.0,
        90.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.5,
            "colors": [
              [
                1.0,
                0.5356,
                0.0,
                0.0
              ],
              [
                1.0,
                0.5356,
                0.0,
                1.0
              ],
              [
                1.0,
                1.0,
                0.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/n3-6.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              0.1719,
              1.4388
            ],
            [
              0.5312,
              3.7984
            ],
            [
              0.6719,
              5.0235
            ],
            [
              0.7656,
              6.0839
            ],
            [
              0.8438,
              7.1644
            ],
            [
              0.9219,
              8.4589
            ],
            [
              1.0,
              10.0
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
      "at": 0.25,
      "offset": [
        0.0,
        0.25,
        2.0
      ],
      "rotate": [
        -90.0,
        -45.0,
        -90.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/model-Hammer"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/n4-0.effect"
        },
        "scale": 4.0,
        "spin": [
          0.0,
          0.0,
          -240.0
        ],
        "color": [
          1.0,
          0.5356,
          0.2195,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.0,
        -3.0
      ],
      "rotate": [
        0.0,
        -89.9802,
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
            "inner": 0.6667,
            "band": 0.2,
            "height": [
              4.0,
              0.0
            ],
            "colors": [
              [
                1.0,
                0.0,
                0.0,
                0.0
              ],
              [
                1.0,
                0.2195,
                0.0,
                1.0
              ],
              [
                1.0,
                1.0,
                0.0,
                0.4902
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/n5-0.effect"
        },
        "scale": 1.5,
        "spin": [
          0.0,
          0.0,
          2400.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.0,
        2.0
      ],
      "rotate": [
        -90.0,
        0.0,
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
            "inner": 0.6667,
            "band": 0.2,
            "height": [
              4.0,
              0.0
            ],
            "colors": [
              [
                1.0,
                0.0,
                0.0,
                0.0
              ],
              [
                1.0,
                0.2195,
                0.0,
                1.0
              ],
              [
                1.0,
                1.0,
                0.0,
                0.4902
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/n5-1.effect"
        },
        "scale": 1.5,
        "spin": [
          0.0,
          0.0,
          2400.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.0,
        2.0
      ],
      "rotate": [
        -90.0,
        0.0,
        90.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6667,
            "band": 0.5,
            "height": [
              1.3333,
              0.0
            ],
            "start": 210.0,
            "length": 120.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                0.2195,
                0.2195,
                0.502
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/n6-0.effect"
        },
        "scale": 2.25,
        "spin": [
          0.0,
          0.0,
          600.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        0.0,
        -0.25,
        2.0
      ],
      "rotate": [
        -90.0,
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
            "inner": 0.6667,
            "band": 0.5,
            "height": [
              1.3333,
              0.0
            ],
            "start": 210.0,
            "length": 120.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                0.2195,
                0.2195,
                0.502
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem5/n6-1.effect"
        },
        "scale": 2.25,
        "spin": [
          0.0,
          0.0,
          600.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        0.0,
        -0.25,
        2.0
      ],
      "rotate": [
        -90.0,
        0.0,
        -180.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          0.9982,
          0.9962
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
      "at": 0.1667,
      "life": 0.3,
      "offset": [
        0.0,
        0.25,
        2.0
      ]
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.1667
    }
  ],
  "duration": 0.9833,
  "events": [
    {
      "name": "impact",
      "time": 0.1667
    }
  ]
});
