import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0 "WaterCylinder1": ring 10 instances: the first 4 drawn
// effekseer: node 1 "Bloom1": a UV region of a texture whose size is not known (not a PNG)
// effekseer: node 4 "WaterCylinder1": ring 10 instances: the first 4 drawn
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.2,
            "band": 0.5,
            "height": [
              0.6667,
              -0.2
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
                0.0023,
                1.0,
                1.0,
                0.0627
              ],
              [
                1.0,
                1.0,
                1.0,
                0.1255
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Undine2/n0.effect"
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
      "rotate": [
        -90.0,
        0.0,
        -135.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.2,
            "band": 0.5,
            "height": [
              0.6667,
              -0.2
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
                0.0023,
                1.0,
                1.0,
                0.0627
              ],
              [
                1.0,
                1.0,
                1.0,
                0.1255
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Undine2/n0.effect"
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
      "at": 0.0167,
      "rotate": [
        -90.0,
        0.0,
        -45.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.2,
            "band": 0.5,
            "height": [
              0.6667,
              -0.2
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
                0.0023,
                1.0,
                1.0,
                0.0627
              ],
              [
                1.0,
                1.0,
                1.0,
                0.1255
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Undine2/n0.effect"
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
      "at": 0.0333,
      "rotate": [
        -90.0,
        0.0,
        45.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.2,
            "band": 0.5,
            "height": [
              0.6667,
              -0.2
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
                0.0023,
                1.0,
                1.0,
                0.0627
              ],
              [
                1.0,
                1.0,
                1.0,
                0.1255
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Undine2/n0.effect"
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
      "at": 0.05,
      "rotate": [
        -90.0,
        0.0,
        135.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.8,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.3587,
                0.3587,
                1.0,
                0.1255
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Undine2/n1.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.4
            ],
            [
              0.0781,
              0.8315
            ],
            [
              0.1562,
              1.194
            ],
            [
              0.2344,
              1.4965
            ],
            [
              0.3281,
              1.7934
            ],
            [
              0.4688,
              2.1365
            ],
            [
              0.8281,
              2.7971
            ],
            [
              1.0,
              3.2
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
      "offset": [
        0.0,
        0.75,
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
            "inner": 0.5714,
            "band": 0.5,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.3587,
                0.7519,
                1.0,
                0.1255
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Undine2/n2.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.35
            ],
            [
              0.0781,
              0.8355
            ],
            [
              0.1562,
              1.2432
            ],
            [
              0.2344,
              1.5836
            ],
            [
              0.3281,
              1.9176
            ],
            [
              0.4688,
              2.3035
            ],
            [
              0.8281,
              3.0468
            ],
            [
              1.0,
              3.5
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
        0.75,
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
            "inner": 0.5714,
            "band": 0.5,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.3587,
                0.7519,
                1.0,
                0.1255
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Undine2/n3.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.35
            ],
            [
              0.0781,
              0.8355
            ],
            [
              0.1562,
              1.2432
            ],
            [
              0.2344,
              1.5836
            ],
            [
              0.3281,
              1.9176
            ],
            [
              0.4688,
              2.3035
            ],
            [
              0.8281,
              3.0468
            ],
            [
              1.0,
              3.5
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
        0.75,
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
            "height": [
              0.15,
              -0.15
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
                0.0023,
                1.0,
                1.0,
                0.0627
              ],
              [
                1.0,
                1.0,
                1.0,
                0.1255
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Undine2/n4.effect"
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
      "rotate": [
        -90.0,
        0.0,
        -135.0
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
            "height": [
              0.15,
              -0.15
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
                0.0023,
                1.0,
                1.0,
                0.0627
              ],
              [
                1.0,
                1.0,
                1.0,
                0.1255
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Undine2/n4.effect"
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
      "at": 0.0167,
      "rotate": [
        -90.0,
        0.0,
        -45.0
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
            "height": [
              0.15,
              -0.15
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
                0.0023,
                1.0,
                1.0,
                0.0627
              ],
              [
                1.0,
                1.0,
                1.0,
                0.1255
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Undine2/n4.effect"
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
      "at": 0.0333,
      "rotate": [
        -90.0,
        0.0,
        45.0
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
            "height": [
              0.15,
              -0.15
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
                0.0023,
                1.0,
                1.0,
                0.0627
              ],
              [
                1.0,
                1.0,
                1.0,
                0.1255
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Undine2/n4.effect"
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
      "at": 0.05,
      "rotate": [
        -90.0,
        0.0,
        135.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Undine2/n5.emitter"
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Undine2/n6-0-0.emitter"
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
        "kind": "light",
        "color": [
          0.937,
          0.9501,
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
        0.4744,
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
  "duration": 0.5667,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
