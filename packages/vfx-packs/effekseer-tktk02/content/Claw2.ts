import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0.0.3.1.0.0.0 "Line": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.0.0.3.1.0.0.0 "Line": nested generation: 100 instances above its generator folded into one emitter
// effekseer: node 0.0.0.1.0.3.1.0.0.0 "Line": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.1.0.3.1.0.0.0 "Line": nested generation: 100 instances above its generator folded into one emitter
// effekseer: node 0.0.0.2.0.3.1.0.0.0 "Line": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.2.0.3.1.0.0.0 "Line": nested generation: 100 instances above its generator folded into one emitter
// effekseer: node 1.0.0.0.0 "Node": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.0.0.0.0 "Node": nested generation: 100 instances above its generator folded into one emitter
// effekseer: node 1.2 "Distortion": motion out from a circle: approximated along one direction
// effekseer: node 1.4 "Distortion": motion out from a circle: approximated along one direction
// effekseer: node 1.5.0 "RingZ": ring 10 instances: the first 4 drawn
// effekseer: node 1.5.0 "RingZ": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 1.5.1 "Ring": a ring's arc over its life
// effekseer: node 1.5.1 "Ring": ring 2 instances: the first 4 drawn
// effekseer: node 1.5.1 "Ring": nested generation: 10 instances above its generator folded into one emitter
// effekseer: node 1.5.1 "Ring": a ring whose thickness changes over its life: its middle thickness kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n0-0-0-0-0-0.effect"
        },
        "width": [
          1.9444,
          1.9444
        ],
        "lifetime": 0.3333,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          9.3333,
          -1.3333
        ]
      },
      "at": 0.0,
      "offset": [
        -1.2794,
        -0.6062,
        -0.4119
      ],
      "rotate": [
        -34.3235,
        77.6576,
        -142.278
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n0-0-0-0-0-1.effect"
        },
        "width": [
          3.8889,
          3.8889
        ],
        "lifetime": 0.4167,
        "color": [
          1.0,
          1.0,
          0.0478,
          1.0
        ],
        "orbit": [
          9.3333,
          -1.3333
        ]
      },
      "at": 0.0,
      "offset": [
        -1.2794,
        -0.6062,
        -0.4119
      ],
      "rotate": [
        -34.3235,
        77.6576,
        -142.278
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n0-0-0-0-0-2.effect"
        },
        "width": [
          15.5556,
          15.5556
        ],
        "lifetime": 0.5,
        "color": [
          0.5356,
          1.0,
          0.0104,
          0.502
        ],
        "orbit": [
          9.3333,
          -1.3333
        ]
      },
      "at": 0.0,
      "offset": [
        -1.2794,
        -0.6062,
        -0.4119
      ],
      "rotate": [
        -34.3235,
        77.6576,
        -142.278
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n0-0-0-0-0-3-0-0-0-0.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        7.5916,
        0.4172,
        2.6695
      ],
      "rotate": [
        -34.3235,
        77.6576,
        127.7219
      ],
      "until": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n0-0-0-0-0-3-0-0-0-1.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        7.5916,
        0.4172,
        2.6695
      ],
      "rotate": [
        -34.3235,
        77.6576,
        127.7219
      ],
      "until": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n0-0-0-0-0-3-0-0-0-2.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        7.5916,
        0.4172,
        2.6695
      ],
      "rotate": [
        -34.3235,
        77.6576,
        127.7219
      ],
      "until": 0.5
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n0-0-0-0-0-3-1-0-0-0.effect"
        },
        "width": [
          1.5556,
          0.3889
        ],
        "lifetime": 0.3333,
        "color": [
          0.3562,
          0.6354,
          0.1166,
          0.1255
        ],
        "orbit": [
          9.3333,
          -1.3333
        ]
      },
      "at": 0.0667,
      "offset": [
        -1.2794,
        -0.6062,
        -0.4119
      ],
      "rotate": [
        -34.3235,
        77.6576,
        -142.278
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n0-0-0-1-0-0.effect"
        },
        "width": [
          1.9444,
          1.9444
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          9.3333,
          -1.0556
        ]
      },
      "at": 0.0,
      "offset": [
        -1.1243,
        -1.8001,
        0.342
      ],
      "rotate": [
        -141.1841,
        77.3697,
        -36.7342
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n0-0-0-1-0-1.effect"
        },
        "width": [
          3.8889,
          3.8889
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          1.0,
          0.0478,
          1.0
        ],
        "orbit": [
          9.3333,
          -1.0556
        ]
      },
      "at": 0.0,
      "offset": [
        -1.1243,
        -1.8001,
        0.342
      ],
      "rotate": [
        -141.1841,
        77.3697,
        -36.7342
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n0-0-0-1-0-2.effect"
        },
        "width": [
          15.5556,
          15.5556
        ],
        "lifetime": 0.3333,
        "color": [
          0.5356,
          1.0,
          0.0104,
          0.502
        ],
        "orbit": [
          9.3333,
          -1.0556
        ]
      },
      "at": 0.0,
      "offset": [
        -1.1243,
        -1.8001,
        0.342
      ],
      "rotate": [
        -141.1841,
        77.3697,
        -36.7342
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n0-0-0-1-0-3-0-0-0-0.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        8.2387,
        -0.5561,
        0.1872
      ],
      "rotate": [
        -141.1841,
        77.3698,
        -126.7342
      ],
      "until": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n0-0-0-1-0-3-0-0-0-1.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        8.2387,
        -0.5561,
        0.1872
      ],
      "rotate": [
        -141.1841,
        77.3698,
        -126.7342
      ],
      "until": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n0-0-0-1-0-3-0-0-0-2.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        8.2387,
        -0.5561,
        0.1872
      ],
      "rotate": [
        -141.1841,
        77.3698,
        -126.7342
      ],
      "until": 0.5
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n0-0-0-1-0-3-1-0-0-0.effect"
        },
        "width": [
          1.5556,
          0.3889
        ],
        "lifetime": 0.3333,
        "color": [
          0.3562,
          0.6354,
          0.1166,
          0.0941
        ],
        "orbit": [
          9.3333,
          -1.0556
        ]
      },
      "at": 0.0667,
      "offset": [
        -1.1243,
        -1.8001,
        0.342
      ],
      "rotate": [
        -141.1841,
        77.3697,
        -36.7342
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n0-0-0-2-0-0.effect"
        },
        "width": [
          1.9444,
          1.9444
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          9.3333,
          -1.0556
        ]
      },
      "at": 0.0,
      "offset": [
        -1.4294,
        0.5105,
        0.4531
      ],
      "rotate": [
        -141.1841,
        77.3697,
        -36.7342
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n0-0-0-2-0-1.effect"
        },
        "width": [
          3.8889,
          3.8889
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          1.0,
          0.0478,
          1.0
        ],
        "orbit": [
          9.3333,
          -1.0556
        ]
      },
      "at": 0.0,
      "offset": [
        -1.4294,
        0.5105,
        0.4531
      ],
      "rotate": [
        -141.1841,
        77.3697,
        -36.7342
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n0-0-0-2-0-2.effect"
        },
        "width": [
          15.5556,
          15.5556
        ],
        "lifetime": 0.3333,
        "color": [
          0.5356,
          1.0,
          0.0104,
          0.502
        ],
        "orbit": [
          9.3333,
          -1.0556
        ]
      },
      "at": 0.0,
      "offset": [
        -1.4294,
        0.5105,
        0.4531
      ],
      "rotate": [
        -141.1841,
        77.3697,
        -36.7342
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n0-0-0-2-0-3-0-0-0-0.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        7.9336,
        1.7545,
        0.2983
      ],
      "rotate": [
        -141.1841,
        77.3698,
        -126.7342
      ],
      "until": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n0-0-0-2-0-3-0-0-0-1.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        7.9336,
        1.7545,
        0.2983
      ],
      "rotate": [
        -141.1841,
        77.3698,
        -126.7342
      ],
      "until": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n0-0-0-2-0-3-0-0-0-2.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        7.9336,
        1.7545,
        0.2983
      ],
      "rotate": [
        -141.1841,
        77.3698,
        -126.7342
      ],
      "until": 0.5
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n0-0-0-2-0-3-1-0-0-0.effect"
        },
        "width": [
          1.5556,
          0.3889
        ],
        "lifetime": 0.3333,
        "color": [
          0.3562,
          0.6354,
          0.1166,
          0.0941
        ],
        "orbit": [
          9.3333,
          -1.0556
        ]
      },
      "at": 0.0667,
      "offset": [
        -1.4294,
        0.5105,
        0.4531
      ],
      "rotate": [
        -141.1841,
        77.3697,
        -36.7342
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n0-0-1-0-0-0-0-0-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        6.5417,
        0.1976,
        4.3545
      ],
      "rotate": [
        -10.1016,
        59.3331,
        104.857
      ],
      "until": 0.5
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n1-0-0-0-0.effect"
        },
        "width": [
          1.0,
          0.25
        ],
        "lifetime": 0.25,
        "color": [
          0.3562,
          0.6354,
          0.1166,
          1.0
        ]
      },
      "at": 0.1333,
      "offset": [
        0.0,
        1.0,
        0.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n1-1-0-0-0.emitter"
        }
      },
      "at": 0.1333,
      "offset": [
        0.0,
        1.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n1-1-0-0-1.emitter"
        }
      },
      "at": 0.1333,
      "offset": [
        0.0,
        1.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n1-1-0-0-1-0.emitter"
        }
      },
      "at": 0.1333,
      "offset": [
        0.0,
        1.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n1-2.emitter"
        }
      },
      "at": 0.1333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n1-3-0.emitter"
        }
      },
      "at": 0.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n1-3-1.emitter"
        }
      },
      "at": 0.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n1-3-2.emitter"
        }
      },
      "at": 0.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n1-4.emitter"
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
            "inner": 0.7755,
            "band": 1.0,
            "colors": [
              [
                0.4441,
                1.0,
                0.0072,
                0.0
              ],
              [
                0.6776,
                1.0,
                0.2258,
                0.3438
              ],
              [
                0.4441,
                1.0,
                0.0072,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n1-5-0.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.1094,
              2.6494
            ],
            [
              0.2188,
              3.1968
            ],
            [
              0.3281,
              3.6518
            ],
            [
              0.4531,
              4.0713
            ],
            [
              0.5781,
              4.3973
            ],
            [
              0.7031,
              4.6445
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1333,
      "rotate": [
        0.0,
        0.0,
        -7.5
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.7755,
            "band": 1.0,
            "colors": [
              [
                0.4441,
                1.0,
                0.0072,
                0.0
              ],
              [
                0.6776,
                1.0,
                0.2258,
                0.3438
              ],
              [
                0.4441,
                1.0,
                0.0072,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n1-5-0.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.1094,
              2.6494
            ],
            [
              0.2188,
              3.1968
            ],
            [
              0.3281,
              3.6518
            ],
            [
              0.4531,
              4.0713
            ],
            [
              0.5781,
              4.3973
            ],
            [
              0.7031,
              4.6445
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1367,
      "rotate": [
        0.0,
        0.0,
        -2.5
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.7755,
            "band": 1.0,
            "colors": [
              [
                0.4441,
                1.0,
                0.0072,
                0.0
              ],
              [
                0.6776,
                1.0,
                0.2258,
                0.3438
              ],
              [
                0.4441,
                1.0,
                0.0072,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n1-5-0.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.1094,
              2.6494
            ],
            [
              0.2188,
              3.1968
            ],
            [
              0.3281,
              3.6518
            ],
            [
              0.4531,
              4.0713
            ],
            [
              0.5781,
              4.3973
            ],
            [
              0.7031,
              4.6445
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.14,
      "rotate": [
        0.0,
        0.0,
        2.5
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.7755,
            "band": 1.0,
            "colors": [
              [
                0.4441,
                1.0,
                0.0072,
                0.0
              ],
              [
                0.6776,
                1.0,
                0.2258,
                0.3438
              ],
              [
                0.4441,
                1.0,
                0.0072,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n1-5-0.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.1094,
              2.6494
            ],
            [
              0.2188,
              3.1968
            ],
            [
              0.3281,
              3.6518
            ],
            [
              0.4531,
              4.0713
            ],
            [
              0.5781,
              4.3973
            ],
            [
              0.7031,
              4.6445
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1433,
      "rotate": [
        0.0,
        0.0,
        7.5
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.8673,
            "band": 0.775,
            "start": 170.0,
            "length": 200.0,
            "colors": [
              [
                0.5351,
                1.0,
                0.0147,
                0.0
              ],
              [
                0.8303,
                1.0,
                0.5377,
                0.6719
              ],
              [
                0.5351,
                1.0,
                0.0147,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw2/n1-5-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.1094,
              2.4329
            ],
            [
              0.2188,
              2.7979
            ],
            [
              0.3281,
              3.1012
            ],
            [
              0.4531,
              3.3808
            ],
            [
              0.5781,
              3.5982
            ],
            [
              0.7031,
              3.763
            ],
            [
              1.0,
              4.0
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          432.0
        ],
        "color": [
          2.0,
          2.0,
          2.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1333,
      "life": 0.4167
    }
  ],
  "duration": 1.3333,
  "looping": true
});
