import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0 "Node": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 0.1 "Node": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 5.0 "Node": a moving ancestor: its descendants are placed where it starts
// effekseer: node 5.0 "Node": ring 5 instances: the first 4 drawn
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/salamander13/n0-0.effect"
        },
        "scale": 4.0415,
        "color": [
          1.0,
          1.0,
          1.0,
          0.7529
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        0.0,
        2.6167,
        0.0
      ],
      "life": 1.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/salamander13/n0-1.effect"
        },
        "scale": 4.0415,
        "color": [
          1.0,
          1.0,
          1.0,
          0.7529
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        0.0,
        2.6167,
        0.0
      ],
      "rotate": [
        0.0,
        89.9802,
        0.0
      ],
      "life": 1.6667
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
              2.6667,
              0.0
            ],
            "start": 150.0,
            "length": 240.0,
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
                0.7529
              ],
              [
                1.0,
                1.0,
                1.0,
                0.7529
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/salamander13/n1-0.effect"
        },
        "scale": 0.875,
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
        -0.3,
        0.0
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 1.6667
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
              2.6667,
              0.0
            ],
            "start": 150.0,
            "length": 240.0,
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
                0.7529
              ],
              [
                1.0,
                1.0,
                1.0,
                0.7529
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/salamander13/n1-0.effect"
        },
        "scale": 0.875,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0167,
      "offset": [
        0.0,
        -0.3,
        0.0
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 1.6667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/salamander13/n2.emitter"
        }
      },
      "at": 0.0,
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/salamander13/n3.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        1.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/salamander13/n4.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        1.0,
        0.0
      ]
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
              0.2667,
              0.0
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
                0.2195,
                0.0,
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/salamander13/n5-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              0.0781,
              0.0664
            ],
            [
              0.1562,
              0.1683
            ],
            [
              0.2344,
              0.3011
            ],
            [
              0.3281,
              0.4943
            ],
            [
              0.4688,
              0.8368
            ],
            [
              0.8281,
              1.823
            ],
            [
              1.0,
              2.25
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
      "at": 0.0833,
      "rotate": [
        -90.0,
        0.0,
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
            "inner": 0.6667,
            "band": 0.5,
            "height": [
              0.2667,
              0.0
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
                0.2195,
                0.0,
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/salamander13/n5-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              0.0781,
              0.0664
            ],
            [
              0.1562,
              0.1683
            ],
            [
              0.2344,
              0.3011
            ],
            [
              0.3281,
              0.4943
            ],
            [
              0.4688,
              0.8368
            ],
            [
              0.8281,
              1.823
            ],
            [
              1.0,
              2.25
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
        -90.0,
        0.0,
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
            "inner": 0.6667,
            "band": 0.5,
            "height": [
              0.2667,
              0.0
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
                0.2195,
                0.0,
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/salamander13/n5-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              0.0781,
              0.0664
            ],
            [
              0.1562,
              0.1683
            ],
            [
              0.2344,
              0.3011
            ],
            [
              0.3281,
              0.4943
            ],
            [
              0.4688,
              0.8368
            ],
            [
              0.8281,
              1.823
            ],
            [
              1.0,
              2.25
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
      "rotate": [
        -90.0,
        0.0,
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
            "inner": 0.6667,
            "band": 0.5,
            "height": [
              0.2667,
              0.0
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
                0.2195,
                0.0,
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/salamander13/n5-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              0.0781,
              0.0664
            ],
            [
              0.1562,
              0.1683
            ],
            [
              0.2344,
              0.3011
            ],
            [
              0.3281,
              0.4943
            ],
            [
              0.4688,
              0.8368
            ],
            [
              0.8281,
              1.823
            ],
            [
              1.0,
              2.25
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
      "rotate": [
        -90.0,
        0.0,
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
            "inner": 0.5,
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
                0.5356,
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/salamander13/n6.effect"
        },
        "faceCamera": true,
        "scale": {
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
        "spin": [
          0.0,
          0.0,
          -600.0
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
      "life": 0.1667
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
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                0.5356,
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/salamander13/n7.effect"
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
              8.0
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          -600.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0833,
      "life": 0.1667
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          0.9667,
          0.9651
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
      "life": 0.3,
      "offset": [
        0.0,
        0.7114,
        0.0
      ]
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.0
    }
  ],
  "duration": 1.6833,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
