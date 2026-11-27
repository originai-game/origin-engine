import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0 "Ring": ring 10 instances: the first 4 drawn
// effekseer: node 0.0.0.0 "Ring": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.0.3.0 "Ring": ring 10 instances: the first 4 drawn
// effekseer: node 1.0.0.0 "Node": ring 20 instances: the first 4 drawn
// effekseer: node 1.0.0.0 "Node": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 1.0.3.0 "Ring": ring 20 instances: the first 4 drawn
// effekseer: node 2.0.0.0 "Ring": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 3.1.0.0.0.0 "LineMain": a moving ancestor: its descendants are placed where it starts
// effekseer: node 3.1.0.0.0.0 "LineMain": nested generation: 40 instances above its generator folded into one emitter
// effekseer: node 3.1.0.0.0.1.0 "LineColor": nested generation: 40 instances above its generator folded into one emitter
// effekseer: node 3.2.0.0.0 "Ring": ring 20 instances: the first 4 drawn
// effekseer: node 3.2.0.0.0 "Ring": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: more than 64 elements: the last 18 left out
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.901,
            "band": 0.75,
            "colors": [
              [
                1.0,
                0.4259,
                0.1542,
                0.0
              ],
              [
                1.0,
                0.8457,
                0.7033,
                1.0
              ],
              [
                1.0,
                0.4259,
                0.1542,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n0-0-0-0.effect"
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
              0.4531,
              0.0
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
          0.75
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "rotate": [
        0.0,
        0.0,
        -135.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.901,
            "band": 0.75,
            "colors": [
              [
                1.0,
                0.4259,
                0.1542,
                0.0
              ],
              [
                1.0,
                0.8457,
                0.7033,
                1.0
              ],
              [
                1.0,
                0.4259,
                0.1542,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n0-0-0-0.effect"
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
              0.4531,
              0.0
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
          0.75
        ],
        "blend": "additive"
      },
      "at": 0.0667,
      "rotate": [
        0.0,
        0.0,
        -45.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.901,
            "band": 0.75,
            "colors": [
              [
                1.0,
                0.4259,
                0.1542,
                0.0
              ],
              [
                1.0,
                0.8457,
                0.7033,
                1.0
              ],
              [
                1.0,
                0.4259,
                0.1542,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n0-0-0-0.effect"
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
              0.4531,
              0.0
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
          0.75
        ],
        "blend": "additive"
      },
      "at": 0.1333,
      "rotate": [
        0.0,
        0.0,
        45.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.901,
            "band": 0.75,
            "colors": [
              [
                1.0,
                0.4259,
                0.1542,
                0.0
              ],
              [
                1.0,
                0.8457,
                0.7033,
                1.0
              ],
              [
                1.0,
                0.4259,
                0.1542,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n0-0-0-0.effect"
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
              0.4531,
              0.0
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
          0.75
        ],
        "blend": "additive"
      },
      "at": 0.2,
      "rotate": [
        0.0,
        0.0,
        135.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n0-0-1-0-0-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n0-0-1-0-0-1.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n0-0-1-0-0-1-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n0-0-1-1-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n0-0-1-1-1.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n0-0-1-1-1-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n0-0-2.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.05,
            "colors": [
              [
                1.0,
                0.2845,
                0.1062,
                0.0
              ],
              [
                1.0,
                0.6346,
                0.4841,
                1.0
              ],
              [
                1.0,
                0.2845,
                0.1062,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n0-0-3-0.effect"
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
              0.1094,
              0.0
            ],
            [
              0.2188,
              0.0
            ],
            [
              0.3281,
              0.0
            ],
            [
              0.4531,
              0.0
            ],
            [
              0.7031,
              0.0
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
          0.5
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "rotate": [
        0.0,
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
            "inner": 0.5,
            "band": 0.05,
            "colors": [
              [
                1.0,
                0.2845,
                0.1062,
                0.0
              ],
              [
                1.0,
                0.6346,
                0.4841,
                1.0
              ],
              [
                1.0,
                0.2845,
                0.1062,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n0-0-3-0.effect"
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
              0.1094,
              0.0
            ],
            [
              0.2188,
              0.0
            ],
            [
              0.3281,
              0.0
            ],
            [
              0.4531,
              0.0
            ],
            [
              0.7031,
              0.0
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
          0.5
        ],
        "blend": "additive"
      },
      "at": 0.0667,
      "rotate": [
        0.0,
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
            "inner": 0.5,
            "band": 0.05,
            "colors": [
              [
                1.0,
                0.2845,
                0.1062,
                0.0
              ],
              [
                1.0,
                0.6346,
                0.4841,
                1.0
              ],
              [
                1.0,
                0.2845,
                0.1062,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n0-0-3-0.effect"
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
              0.1094,
              0.0
            ],
            [
              0.2188,
              0.0
            ],
            [
              0.3281,
              0.0
            ],
            [
              0.4531,
              0.0
            ],
            [
              0.7031,
              0.0
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
          0.5
        ],
        "blend": "additive"
      },
      "at": 0.1333,
      "rotate": [
        0.0,
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
            "inner": 0.5,
            "band": 0.05,
            "colors": [
              [
                1.0,
                0.2845,
                0.1062,
                0.0
              ],
              [
                1.0,
                0.6346,
                0.4841,
                1.0
              ],
              [
                1.0,
                0.2845,
                0.1062,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n0-0-3-0.effect"
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
              0.1094,
              0.0
            ],
            [
              0.2188,
              0.0
            ],
            [
              0.3281,
              0.0
            ],
            [
              0.4531,
              0.0
            ],
            [
              0.7031,
              0.0
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
          0.5
        ],
        "blend": "additive"
      },
      "at": 0.2,
      "rotate": [
        0.0,
        0.0,
        135.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n0-0-4.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n0-0-5.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n0-0-6-0-0-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n0-0-6-0-0-1.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n0-0-6-0-0-1-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n0-0-7-0.emitter"
        }
      },
      "at": 0.0667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n0-0-7-1.emitter"
        }
      },
      "at": 0.0667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n0-0-7-2.emitter"
        }
      },
      "at": 0.0667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.901,
            "band": 0.75,
            "colors": [
              [
                1.0,
                0.4259,
                0.1542,
                0.0
              ],
              [
                1.0,
                0.8457,
                0.7033,
                1.0
              ],
              [
                1.0,
                0.4259,
                0.1542,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n1-0-0-0.effect"
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
              0.4531,
              0.0
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
          0.75
        ],
        "blend": "additive"
      },
      "at": 0.7167,
      "rotate": [
        0.0,
        0.0,
        -135.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.901,
            "band": 0.75,
            "colors": [
              [
                1.0,
                0.4259,
                0.1542,
                0.0
              ],
              [
                1.0,
                0.8457,
                0.7033,
                1.0
              ],
              [
                1.0,
                0.4259,
                0.1542,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n1-0-0-0.effect"
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
              0.4531,
              0.0
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
          0.75
        ],
        "blend": "additive"
      },
      "at": 0.75,
      "rotate": [
        0.0,
        0.0,
        -45.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.901,
            "band": 0.75,
            "colors": [
              [
                1.0,
                0.4259,
                0.1542,
                0.0
              ],
              [
                1.0,
                0.8457,
                0.7033,
                1.0
              ],
              [
                1.0,
                0.4259,
                0.1542,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n1-0-0-0.effect"
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
              0.4531,
              0.0
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
          0.75
        ],
        "blend": "additive"
      },
      "at": 0.7833,
      "rotate": [
        0.0,
        0.0,
        45.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.901,
            "band": 0.75,
            "colors": [
              [
                1.0,
                0.4259,
                0.1542,
                0.0
              ],
              [
                1.0,
                0.8457,
                0.7033,
                1.0
              ],
              [
                1.0,
                0.4259,
                0.1542,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n1-0-0-0.effect"
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
              0.4531,
              0.0
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
          0.75
        ],
        "blend": "additive"
      },
      "at": 0.8167,
      "rotate": [
        0.0,
        0.0,
        135.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n1-0-1-0-0-0.parent"
        }
      },
      "at": 0.7167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n1-0-1-0-0-1.parent"
        }
      },
      "at": 0.7167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n1-0-1-0-0-1-0.parent"
        }
      },
      "at": 0.7167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n1-0-1-1-0.parent"
        }
      },
      "at": 0.7167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n1-0-1-1-1.parent"
        }
      },
      "at": 0.7167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n1-0-1-1-1-0.parent"
        }
      },
      "at": 0.7167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n1-0-2.parent"
        }
      },
      "at": 0.7167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.05,
            "colors": [
              [
                1.0,
                0.0619,
                0.0231,
                0.0
              ],
              [
                1.0,
                0.139,
                0.1062,
                1.0
              ],
              [
                1.0,
                0.0619,
                0.0231,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n1-0-3-0.effect"
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
              0.1094,
              0.0
            ],
            [
              0.2188,
              0.0
            ],
            [
              0.3281,
              0.0
            ],
            [
              0.4531,
              0.0
            ],
            [
              0.7031,
              0.0
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
          0.5
        ],
        "blend": "additive"
      },
      "at": 0.7167,
      "rotate": [
        0.0,
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
            "inner": 0.5,
            "band": 0.05,
            "colors": [
              [
                1.0,
                0.0619,
                0.0231,
                0.0
              ],
              [
                1.0,
                0.139,
                0.1062,
                1.0
              ],
              [
                1.0,
                0.0619,
                0.0231,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n1-0-3-0.effect"
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
              0.1094,
              0.0
            ],
            [
              0.2188,
              0.0
            ],
            [
              0.3281,
              0.0
            ],
            [
              0.4531,
              0.0
            ],
            [
              0.7031,
              0.0
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
          0.5
        ],
        "blend": "additive"
      },
      "at": 0.75,
      "rotate": [
        0.0,
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
            "inner": 0.5,
            "band": 0.05,
            "colors": [
              [
                1.0,
                0.0619,
                0.0231,
                0.0
              ],
              [
                1.0,
                0.139,
                0.1062,
                1.0
              ],
              [
                1.0,
                0.0619,
                0.0231,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n1-0-3-0.effect"
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
              0.1094,
              0.0
            ],
            [
              0.2188,
              0.0
            ],
            [
              0.3281,
              0.0
            ],
            [
              0.4531,
              0.0
            ],
            [
              0.7031,
              0.0
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
          0.5
        ],
        "blend": "additive"
      },
      "at": 0.7833,
      "rotate": [
        0.0,
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
            "inner": 0.5,
            "band": 0.05,
            "colors": [
              [
                1.0,
                0.0619,
                0.0231,
                0.0
              ],
              [
                1.0,
                0.139,
                0.1062,
                1.0
              ],
              [
                1.0,
                0.0619,
                0.0231,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n1-0-3-0.effect"
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
              0.1094,
              0.0
            ],
            [
              0.2188,
              0.0
            ],
            [
              0.3281,
              0.0
            ],
            [
              0.4531,
              0.0
            ],
            [
              0.7031,
              0.0
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
          0.5
        ],
        "blend": "additive"
      },
      "at": 0.8167,
      "rotate": [
        0.0,
        0.0,
        135.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n1-0-4.emitter"
        }
      },
      "at": 0.7167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n1-0-5.emitter"
        }
      },
      "at": 0.7167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n1-0-6-0.emitter"
        }
      },
      "at": 0.7833
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n1-0-6-1.emitter"
        }
      },
      "at": 0.7833
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n1-0-6-2.emitter"
        }
      },
      "at": 0.7833
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.901,
            "band": 0.75,
            "colors": [
              [
                1.0,
                0.4259,
                0.1542,
                0.0
              ],
              [
                1.0,
                0.8457,
                0.7033,
                1.0
              ],
              [
                1.0,
                0.4259,
                0.1542,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n2-0-0-0.effect"
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
              0.4531,
              0.0
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
          0.75
        ],
        "blend": "additive"
      },
      "at": 1.4,
      "life": 0.4167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n2-0-1-0-0-0.emitter"
        }
      },
      "at": 1.4,
      "offset": [
        0.0,
        0.625,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n2-0-1-0-0-1.emitter"
        }
      },
      "at": 1.4,
      "offset": [
        0.0,
        0.875,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n2-0-1-0-0-1-0.emitter"
        }
      },
      "at": 1.4,
      "offset": [
        0.0,
        1.125,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n2-0-1-1-0.emitter"
        }
      },
      "at": 1.4,
      "offset": [
        0.0,
        0.25,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n2-0-1-1-0-0.emitter"
        }
      },
      "at": 1.4,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n2-0-2.emitter"
        }
      },
      "at": 1.4
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.05,
            "colors": [
              [
                1.0,
                0.0619,
                0.0231,
                0.0
              ],
              [
                1.0,
                0.139,
                0.1062,
                1.0
              ],
              [
                1.0,
                0.0619,
                0.0231,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n2-0-3-0.effect"
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
              0.1094,
              0.0
            ],
            [
              0.2188,
              0.0
            ],
            [
              0.3281,
              0.0
            ],
            [
              0.4531,
              0.0
            ],
            [
              0.7031,
              0.0
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
          0.5
        ],
        "blend": "additive"
      },
      "at": 1.4,
      "life": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n2-0-4.emitter"
        }
      },
      "at": 1.4
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n2-0-5.emitter"
        }
      },
      "at": 1.4
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n2-0-6-0-0-0.emitter"
        }
      },
      "at": 1.4,
      "offset": [
        0.0,
        1.25,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n2-0-6-0-0-1.emitter"
        }
      },
      "at": 1.4,
      "offset": [
        0.0,
        1.25,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n2-0-6-0-0-1-0.emitter"
        }
      },
      "at": 1.4,
      "offset": [
        0.0,
        1.25,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n2-0-7-0.emitter"
        }
      },
      "at": 1.4667,
      "rotate": [
        0.0,
        0.0,
        90.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n2-0-7-1.emitter"
        }
      },
      "at": 1.4667,
      "rotate": [
        0.0,
        0.0,
        90.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n2-0-7-2.emitter"
        }
      },
      "at": 1.4667,
      "rotate": [
        0.0,
        0.0,
        90.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n2-1.emitter"
        }
      },
      "at": 1.4
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n3-0-0-0-0-0.emitter"
        }
      },
      "at": 1.6667,
      "offset": [
        0.0,
        0.625,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n3-0-0-0-0-1.emitter"
        }
      },
      "at": 1.6667,
      "offset": [
        0.0,
        0.875,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n3-0-0-0-0-1-0.emitter"
        }
      },
      "at": 1.6667,
      "offset": [
        0.0,
        1.125,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n3-0-0-1-0.emitter"
        }
      },
      "at": 1.6667
    }
  ],
  "duration": 3.6,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
