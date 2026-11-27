import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0 "Node": a track whose points move on their own: drawn along its generator's path
// effekseer: node 0.0.0.0 "Node": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 0.0.4 "Node": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.5 "Node": a moving ancestor: its descendants are placed where it starts
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Lance4/n0-0-0-0.effect"
        },
        "width": [
          1.0,
          1.0
        ],
        "lifetime": 0.3333,
        "color": [
          0.2195,
          0.2195,
          0.2195,
          0.0
        ],
        "orbit": [
          1.5,
          5.0
        ]
      },
      "at": 0.0,
      "offset": [
        0.0,
        14.1421,
        14.1421
      ],
      "rotate": [
        45.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        -28.2843,
        -28.2843
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
            "start": 210.0,
            "length": 120.0,
            "colors": [
              [
                1.0,
                1.0,
                0.0,
                0.0
              ],
              [
                1.0,
                0.2195,
                0.0,
                0.7529
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Lance4/n0-0-1.effect"
        },
        "scale": 2.0,
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
        14.8492,
        14.8492
      ],
      "rotate": [
        -45.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        -28.2843,
        -28.2843
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.5,
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
                0.5356,
                0.5356,
                0.251
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Lance4/n0-0-2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              10.0
            ],
            [
              1.0,
              0.0
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
      "at": 0.0,
      "offset": [
        0.0,
        14.8492,
        14.8492
      ],
      "rotate": [
        -45.0,
        0.0,
        -180.0
      ],
      "velocity": [
        0.0,
        -28.2843,
        -28.2843
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Lance4/n0-0-3-0.effect"
        },
        "width": [
          1.0,
          0.25
        ],
        "lifetime": 0.3333,
        "color": [
          0.2195,
          0.2195,
          0.2195,
          0.502
        ],
        "orbit": [
          1.5,
          8.3333
        ]
      },
      "at": 0.0,
      "offset": [
        0.0,
        14.1421,
        14.1421
      ],
      "rotate": [
        45.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        -28.2843,
        -28.2843
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.25,
            "band": 0.5,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.2195,
                0.2195,
                1.0,
                0.7529
              ],
              [
                0.2195,
                0.2195,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Lance4/n0-0-4.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
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
        14.1421,
        14.1421
      ],
      "rotate": [
        -45.0,
        0.0,
        0.0
      ],
      "life": 0.1667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.25,
            "band": 0.5,
            "colors": [
              [
                0.2195,
                0.2195,
                0.0,
                0.0
              ],
              [
                0.2195,
                0.2195,
                1.0,
                0.7529
              ],
              [
                0.2195,
                0.2084,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Lance4/n0-0-5.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
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
      "at": 0.3333,
      "offset": [
        0.0,
        14.1421,
        14.1421
      ],
      "rotate": [
        -45.0,
        0.0,
        0.0
      ],
      "life": 0.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Lance4/n0-0-6.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        16.9706,
        16.9706
      ],
      "rotate": [
        45.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        -28.2843,
        -28.2843
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.3333,
            "band": 0.5,
            "colors": [
              [
                0.0,
                0.0,
                1.0,
                0.0
              ],
              [
                1.0,
                1.0,
                1.0,
                0.7529
              ],
              [
                0.0,
                0.0,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Lance4/n1.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              1.0,
              11.25
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
      "life": 0.25
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Lance4/n2-0.emitter"
        }
      },
      "at": 0.45,
      "offset": [
        0.0,
        -0.5,
        0.0
      ],
      "rotate": [
        -180.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.5,
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
                0.251
              ],
              [
                1.0,
                1.0,
                1.0,
                0.251
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Lance4/n3-0.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.4
            ],
            [
              1.0,
              1.8
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "subtractive"
      },
      "at": 0.55,
      "life": 0.25
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6111,
            "band": 0.5,
            "colors": [
              [
                0.5234,
                0.5356,
                0.5356,
                0.0
              ],
              [
                0.5234,
                0.5356,
                0.5356,
                0.3765
              ],
              [
                0.5234,
                0.5356,
                0.5356,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Lance4/n3-1.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              3.24
            ],
            [
              1.0,
              0.0
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
      "at": 0.5,
      "life": 0.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Lance4/n3-2.emitter"
        }
      },
      "at": 0.5833
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Lance4/n3-3.emitter"
        }
      },
      "at": 0.55
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Lance4/n3-4.emitter"
        }
      },
      "at": 0.5833
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Lance4/n3-5.emitter"
        }
      },
      "at": 0.5333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Lance4/n3-6-0-0.emitter"
        }
      },
      "at": 0.5
    }
  ],
  "duration": 1.15,
  "events": [
    {
      "name": "impact",
      "time": 0.4167
    }
  ]
});
