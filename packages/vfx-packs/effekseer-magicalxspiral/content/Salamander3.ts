import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 1 "FireCylinder1": ring 20 instances: the first 4 drawn
// effekseer: node 6 "FirePlate1": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 11 "Bloom1": texture not found: Texture/
// effekseer: node 11 "Bloom1": a UV region of a texture whose size is not known (not a PNG)
export default file.vfx({
  "elements": [
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
                0.3765
              ],
              [
                1.0,
                1.0,
                1.0,
                0.3765
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Salamander3/n0.effect"
        },
        "faceCamera": true,
        "scale": 1.05,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "subtractive"
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.75,
        0.0
      ],
      "life": 0.8333
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
              2.0,
              -0.3
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
                0.7482,
                0.7482,
                0.1255
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Salamander3/n1.effect"
        },
        "scale": 1.0,
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
              2.0,
              -0.3
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
                0.7482,
                0.7482,
                0.1255
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Salamander3/n1.effect"
        },
        "scale": 1.0,
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
              2.0,
              -0.3
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
                0.7482,
                0.7482,
                0.1255
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Salamander3/n1.effect"
        },
        "scale": 1.0,
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
              2.0,
              -0.3
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
                0.7482,
                0.7482,
                0.1255
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Salamander3/n1.effect"
        },
        "scale": 1.0,
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
                0.3765
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Salamander3/n2.effect"
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
        0.01,
        0.0
      ],
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
                0.3765
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Salamander3/n3.effect"
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
        0.5,
        0.0
      ],
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
                0.3765
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Salamander3/n4.effect"
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
        1.0,
        0.0
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 0.6667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Salamander3/n5.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.7,
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Salamander3/n6.effect"
        },
        "scale": 2.8284,
        "color": [
          1.0,
          1.0,
          1.0,
          0.251
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        0.0,
        1.0,
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Salamander3/n7.effect"
        },
        "scale": 3.4641,
        "color": [
          1.0,
          1.0,
          1.0,
          0.251
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        0.0,
        1.0,
        0.0
      ],
      "rotate": [
        0.0,
        60.0,
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Salamander3/n8.effect"
        },
        "scale": 3.4641,
        "color": [
          1.0,
          1.0,
          1.0,
          0.251
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        0.0,
        1.0,
        0.0
      ],
      "rotate": [
        0.0,
        -60.0,
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
            "inner": 0.625,
            "band": 0.3,
            "height": [
              3.125,
              0.0
            ],
            "colors": [
              [
                1.0,
                0.2195,
                0.0,
                0.0
              ],
              [
                1.0,
                1.0,
                0.0,
                0.502
              ],
              [
                1.0,
                0.2195,
                0.0,
                0.502
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Salamander3/n9.effect"
        },
        "scale": 0.8,
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
      "life": 0.8333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.625,
            "band": 0.3,
            "height": [
              3.125,
              0.0
            ],
            "colors": [
              [
                1.0,
                0.2195,
                0.0,
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
                0.2195,
                0.0,
                0.502
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Salamander3/n10.effect"
        },
        "scale": 0.8,
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
        -180.0
      ],
      "life": 0.8333
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
                0.3587,
                1.0,
                0.0
              ],
              [
                1.0,
                0.3587,
                0.3587,
                0.1255
              ],
              [
                1.0,
                0.3587,
                1.0,
                0.251
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Salamander3/n11.effect"
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
      "life": 0.8333
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
                1.0,
                0.7519,
                0.3587,
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Salamander3/n12.effect"
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
      "life": 0.3333
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
                1.0,
                0.7519,
                0.3587,
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Salamander3/n13.effect"
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
      "at": 0.4167,
      "offset": [
        0.0,
        0.75,
        0.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          0.9878,
          0.9878
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
        0.559,
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
  "duration": 1.3167,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
