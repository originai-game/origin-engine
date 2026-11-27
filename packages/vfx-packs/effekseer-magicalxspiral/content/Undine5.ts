import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 1 "WaterFall": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 2 "WaterFall": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 3 "WaterFall": a sprite scaled unevenly: drawn square at its mean
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.3333,
            "band": 0.5,
            "height": [
              0.1333,
              0.0
            ],
            "colors": [
              [
                0.0478,
                0.0478,
                0.5356,
                0.0
              ],
              [
                0.5356,
                0.7519,
                1.0,
                0.502
              ],
              [
                0.7519,
                0.7519,
                1.0,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Undine5/n0.effect"
        },
        "scale": 2.25,
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
        0.05,
        0.0
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
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Undine5/n1.effect"
        },
        "scale": 5.3666,
        "color": [
          0.0478,
          0.2195,
          0.7519,
          0.3765
        ],
        "blend": "alpha"
      },
      "at": 0.0,
      "offset": [
        0.0,
        12.0,
        0.0
      ],
      "life": 0.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Undine5/n2.effect"
        },
        "scale": 5.3666,
        "color": [
          0.0478,
          0.2195,
          0.7519,
          0.3765
        ],
        "blend": "alpha"
      },
      "at": 0.0,
      "offset": [
        0.0,
        12.0,
        0.0
      ],
      "rotate": [
        0.0,
        30.0,
        0.0
      ],
      "life": 0.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Undine5/n3.effect"
        },
        "scale": 5.3666,
        "color": [
          0.0478,
          0.2195,
          0.7519,
          0.3765
        ],
        "blend": "alpha"
      },
      "at": 0.0,
      "offset": [
        0.0,
        12.0,
        0.0
      ],
      "rotate": [
        -180.0,
        30.0,
        -180.0
      ],
      "life": 0.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Undine5/n4.effect"
        },
        "scale": 4.0,
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
        90.0,
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
            "inner": 1.25,
            "band": 0.6,
            "height": [
              40.0,
              0.0
            ],
            "colors": [
              [
                0.2195,
                0.2195,
                1.0,
                0.0
              ],
              [
                0.5356,
                0.7519,
                1.0,
                0.3765
              ],
              [
                0.5356,
                0.7519,
                1.0,
                0.502
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Undine5/n5.effect"
        },
        "scale": 0.6,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 0.6667
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
              0.25,
              0.0
            ],
            "colors": [
              [
                0.2195,
                1.0,
                1.0,
                0.0
              ],
              [
                0.2195,
                1.0,
                1.0,
                0.502
              ],
              [
                0.2195,
                1.0,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Undine5/n6.effect"
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
              4.0
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
        0.2,
        0.0
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 0.25
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
              0.25,
              0.0
            ],
            "colors": [
              [
                0.2195,
                1.0,
                1.0,
                0.0
              ],
              [
                0.2195,
                1.0,
                1.0,
                0.502
              ],
              [
                0.2195,
                1.0,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Undine5/n7.effect"
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
              4.0
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
      "offset": [
        0.0,
        1.2,
        0.0
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 0.25
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
              0.25,
              0.0
            ],
            "colors": [
              [
                0.2195,
                1.0,
                1.0,
                0.0
              ],
              [
                0.2195,
                1.0,
                1.0,
                0.502
              ],
              [
                0.2195,
                1.0,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Undine5/n8.effect"
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
              4.0
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
        2.2,
        0.0
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Undine5/n9.emitter"
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
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6,
            "band": 0.5,
            "height": [
              1.0,
              0.0
            ],
            "colors": [
              [
                0.0,
                0.0478,
                0.5356,
                0.0
              ],
              [
                0.0478,
                0.2195,
                1.0,
                0.502
              ],
              [
                0.1166,
                0.7519,
                1.0,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Undine5/n10.effect"
        },
        "scale": 1.5,
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
        0.05,
        0.0
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
            "inner": 0.4667,
            "band": 0.5,
            "height": [
              0.6667,
              0.0
            ],
            "colors": [
              [
                0.0,
                0.0478,
                0.5356,
                0.0
              ],
              [
                0.0478,
                0.7519,
                1.0,
                0.502
              ],
              [
                0.2195,
                0.7519,
                1.0,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Undine5/n11.effect"
        },
        "scale": 2.25,
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
        0.05,
        0.0
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
        "kind": "light",
        "color": [
          0.5784,
          0.6812,
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
      "life": 0.3,
      "offset": [
        0.0,
        5.4834,
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
  "duration": 0.6667,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
