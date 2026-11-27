import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.2 "Distortion": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0.4.0 "Naka": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.0.0.4.1 "Soto": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.1.0.2 "Distortion": motion out from a circle: approximated along one direction
// effekseer: node 0.1.0.4.0 "Naka": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.1.0.4.1 "Soto": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.2.0.2 "Distortion": motion out from a circle: approximated along one direction
// effekseer: node 0.2.0.4.0 "Naka": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.2.0.4.1 "Soto": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.3.0.0.0.0 "LineMain": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.3.0.0.0.1 "LineColor": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.3.0.0.0.2 "LineGlow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.3.1.5.0.0.0 "Ring": ring 20 instances: the first 4 drawn
// effekseer: node 0.3.1.5.0.0.0 "Ring": a ring whose thickness changes over its life: its middle thickness kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-0-0-0-0-0-0.emitter"
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-0-0-0-0-0-1.emitter"
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-0-0-0-0-0-2.emitter"
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-0-0-1-0-0.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-0-0-1-0-1.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-0-0-1-0-2.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-0-0-2.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-0-0-3.emitter"
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
            "inner": 0.5459,
            "band": 1.0,
            "colors": [
              [
                0.0,
                0.0,
                1.0,
                0.0
              ],
              [
                0.2195,
                1.0,
                1.0,
                1.0
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-0-0-4-0.effect"
        },
        "scale": 0.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.9082,
            "band": 1.0,
            "colors": [
              [
                0.0,
                0.0,
                1.0,
                0.0
              ],
              [
                0.2195,
                1.0,
                1.0,
                1.0
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-0-0-4-1.effect"
        },
        "scale": 0.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-1-0-0-0-0-0.emitter"
        }
      },
      "at": 0.0667,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-1-0-0-0-0-1.emitter"
        }
      },
      "at": 0.0667,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-1-0-0-0-0-2.emitter"
        }
      },
      "at": 0.0667,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-1-0-1-0-0.emitter"
        }
      },
      "at": 0.0667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-1-0-1-0-1.emitter"
        }
      },
      "at": 0.0667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-1-0-1-0-2.emitter"
        }
      },
      "at": 0.0667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-1-0-2.emitter"
        }
      },
      "at": 0.0667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-1-0-3.emitter"
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
            "inner": 0.5459,
            "band": 1.0,
            "colors": [
              [
                0.0,
                0.0,
                1.0,
                0.0
              ],
              [
                0.2195,
                0.2195,
                1.0,
                1.0
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-1-0-4-0.effect"
        },
        "scale": 0.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0667,
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.9082,
            "band": 1.0,
            "colors": [
              [
                0.0,
                0.0,
                1.0,
                0.0
              ],
              [
                0.2195,
                0.2195,
                1.0,
                1.0
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-1-0-4-1.effect"
        },
        "scale": 0.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0667,
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-2-0-0-0-0-0.emitter"
        }
      },
      "at": 0.1333,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-2-0-0-0-0-1.emitter"
        }
      },
      "at": 0.1333,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-2-0-0-0-0-2.emitter"
        }
      },
      "at": 0.1333,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-2-0-1-0-0.emitter"
        }
      },
      "at": 0.1333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-2-0-1-0-1.emitter"
        }
      },
      "at": 0.1333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-2-0-1-0-2.emitter"
        }
      },
      "at": 0.1333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-2-0-2.emitter"
        }
      },
      "at": 0.1333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-2-0-3.emitter"
        }
      },
      "at": 0.1333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5459,
            "band": 1.0,
            "colors": [
              [
                0.0,
                0.0,
                1.0,
                0.0
              ],
              [
                0.2195,
                0.0478,
                1.0,
                1.0
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-2-0-4-0.effect"
        },
        "scale": 0.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1333,
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.9082,
            "band": 1.0,
            "colors": [
              [
                0.0,
                0.0,
                1.0,
                0.0
              ],
              [
                0.2195,
                0.0478,
                1.0,
                1.0
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-2-0-4-1.effect"
        },
        "scale": 0.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1333,
      "life": 0.5
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-3-0-0-0-0.effect"
        },
        "width": [
          3.0,
          3.0
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 0.2667,
      "offset": [
        4.5,
        7.7942,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -30.0
      ],
      "life": 0.1667
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-3-0-0-0-1.effect"
        },
        "width": [
          15.0,
          15.0
        ],
        "lifetime": 0.25,
        "color": [
          0.5356,
          0.0478,
          1.0,
          1.0
        ]
      },
      "at": 0.2667,
      "offset": [
        4.5,
        7.7942,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -30.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-3-0-0-0-2.effect"
        },
        "width": [
          30.0,
          30.0
        ],
        "lifetime": 0.3333,
        "color": [
          0.2195,
          0.0104,
          1.0,
          0.502
        ]
      },
      "at": 0.2667,
      "offset": [
        4.5,
        7.7942,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -30.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-3-0-0-0-3-0-0-0.emitter"
        }
      },
      "at": 0.35,
      "offset": [
        3.4583,
        5.99,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -30.0
      ],
      "velocity": [
        -17.0,
        -29.4449,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-3-0-0-0-3-0-0-1.emitter"
        }
      },
      "at": 0.35,
      "offset": [
        3.4583,
        5.99,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -30.0
      ],
      "velocity": [
        -17.0,
        -29.4449,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-3-0-0-0-3-0-0-2.emitter"
        }
      },
      "at": 0.35,
      "offset": [
        3.4583,
        5.99,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -30.0
      ],
      "velocity": [
        -17.0,
        -29.4449,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-3-1-0-0-0.emitter"
        }
      },
      "at": 0.3333,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-3-1-0-0-1.emitter"
        }
      },
      "at": 0.3333,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-3-1-0-0-2.emitter"
        }
      },
      "at": 0.3333,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-3-1-1.emitter"
        }
      },
      "at": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-3-1-2-0-0-0.emitter"
        }
      },
      "at": 0.3333,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-3-1-2-0-0-1.emitter"
        }
      },
      "at": 0.3333,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-3-1-2-0-0-2.emitter"
        }
      },
      "at": 0.3333,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-3-1-3.emitter"
        }
      },
      "at": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-3-1-4.emitter"
        }
      },
      "at": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9153,
            "band": 0.625,
            "start": 165.0,
            "length": 210.0,
            "colors": [
              [
                1.0,
                0.2195,
                0.0478,
                0.0
              ],
              [
                1.0,
                0.7134,
                0.5887,
                1.0
              ],
              [
                1.0,
                0.2195,
                0.0478,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow5/n0-3-1-5-0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.875
            ],
            [
              0.4531,
              0.7887
            ],
            [
              1.0,
              0.75
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          617.1428
        ],
        "color": [
          20.0,
          20.0,
          20.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.3333,
      "life": 0.5833
    },
    {
      "node": {
        "kind": "light",
        "color": [
          0.3176,
          0.509,
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
        0.2809,
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
  "duration": 1.3317,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
