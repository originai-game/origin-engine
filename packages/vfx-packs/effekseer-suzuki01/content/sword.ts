import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0 "Sword": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 0.0.0.2 "Sword2": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 0.1.3.0 "Spark": a track whose points move on their own: drawn along its generator's path
// effekseer: node 0.1.3.0 "Spark": nested generation: 10 instances above its generator folded into one emitter
// effekseer: node 0.1.3.0 "Spark": a ribbon or track's sideways motion: only its rise and turn are kept
// effekseer: node 1.0.0.0.0.0 "Light": nested generation: 20 instances above its generator folded into one emitter
// effekseer: node 1.0.0.0.2.0 "Light": nested generation: 20 instances above its generator folded into one emitter
// effekseer: node 1.0.0.1.0 "Ring": ring 20 instances: the first 4 drawn
// effekseer: node 1.0.1.1 "Hit2": nested generation: 20 instances above its generator folded into one emitter
// effekseer: node 1.0.1.2 "Splash": nested generation: 20 instances above its generator folded into one emitter
// effekseer: node 1.0.1.3.0 "Spark": a track whose points move on their own: drawn along its generator's path
// effekseer: node 1.0.1.3.0 "Spark": nested generation: 200 instances above its generator folded into one emitter
// effekseer: node 1.0.1.3.0 "Spark": a ribbon or track's sideways motion: only its rise and turn are kept
// effekseer: node 2.0 "ring": a moving ancestor: its descendants are placed where it starts
// effekseer: node 2.0 "ring": nested generation: 30 instances above its generator folded into one emitter
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-suzuki01/sword/n0-0-0-0.effect"
        },
        "scale": 3.1623,
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
        6.0021,
        -1.0027,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -143.1695
      ],
      "life": 4.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-suzuki01/sword/n0-0-0-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        5.8953,
        -1.0827,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -53.1695
      ],
      "until": 1.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-suzuki01/sword/n0-0-0-2.effect"
        },
        "scale": 3.1623,
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
        6.0021,
        -1.0027,
        0.0
      ],
      "rotate": [
        -90.0,
        -36.8305,
        -180.0
      ],
      "life": 4.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-suzuki01/sword/n0-0-0-2-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        5.8953,
        -1.0827,
        0.0
      ],
      "rotate": [
        -90.0,
        -36.8305,
        -90.0
      ],
      "until": 1.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.4,
            "band": 0.5,
            "start": 180.0,
            "length": 180.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.7059
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
                0.7059
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-suzuki01/sword/n0-0-1-0.effect"
        },
        "scale": 7.5,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.5833,
      "offset": [
        2.0,
        -4.0,
        0.0
      ],
      "rotate": [
        -180.0,
        0.0,
        53.1695
      ],
      "life": 0.75
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-suzuki01/sword/n0-1-0.emitter"
        }
      },
      "at": 0.6667,
      "offset": [
        -3.5,
        -1.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-suzuki01/sword/n0-1-1.emitter"
        }
      },
      "at": 0.6667,
      "offset": [
        -3.5,
        -1.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-suzuki01/sword/n0-1-2.emitter"
        }
      },
      "at": 0.6667,
      "offset": [
        -3.5,
        -1.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-suzuki01/sword/n0-1-3-0.effect"
        },
        "width": [
          0.1,
          0.0
        ],
        "lifetime": 0.3333,
        "color": [
          1.0,
          0.047,
          0.0,
          1.0
        ],
        "rise": 36.0
      },
      "at": 0.6667,
      "offset": [
        -3.5,
        -1.0,
        0.0
      ],
      "rotate": [
        30.0,
        -89.972,
        0.0
      ],
      "life": 0.1583
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-suzuki01/sword/n1-0-0-0-0.emitter"
        }
      },
      "at": 1.1667,
      "offset": [
        2.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-suzuki01/sword/n1-0-0-0-0-0.emitter"
        }
      },
      "at": 1.1667,
      "offset": [
        5.8953,
        -2.0827,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -53.1695
      ],
      "until": 2.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-suzuki01/sword/n1-0-0-0-2.emitter"
        }
      },
      "at": 1.1667,
      "offset": [
        2.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-suzuki01/sword/n1-0-0-0-2-0.emitter"
        }
      },
      "at": 1.1667,
      "offset": [
        5.8953,
        -2.0827,
        0.0
      ],
      "rotate": [
        -90.0,
        -36.8305,
        -90.0
      ],
      "until": 2.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.4,
            "band": 0.5,
            "start": 180.0,
            "length": 180.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.7059
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
                0.7059
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-suzuki01/sword/n1-0-0-1-0.effect"
        },
        "scale": 7.5,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 1.75,
      "rotate": [
        180.0,
        -89.9802,
        0.0
      ],
      "life": 0.75
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.4,
            "band": 0.5,
            "start": 180.0,
            "length": 180.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.7059
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
                0.7059
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-suzuki01/sword/n1-0-0-1-0.effect"
        },
        "scale": 7.5,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 1.8,
      "rotate": [
        180.0,
        -89.9802,
        0.0
      ],
      "life": 0.75
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.4,
            "band": 0.5,
            "start": 180.0,
            "length": 180.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.7059
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
                0.7059
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-suzuki01/sword/n1-0-0-1-0.effect"
        },
        "scale": 7.5,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 1.85,
      "rotate": [
        180.0,
        -89.9802,
        0.0
      ],
      "life": 0.75
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.4,
            "band": 0.5,
            "start": 180.0,
            "length": 180.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.7059
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
                0.7059
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-suzuki01/sword/n1-0-0-1-0.effect"
        },
        "scale": 7.5,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 1.9,
      "rotate": [
        180.0,
        -89.9802,
        0.0
      ],
      "life": 0.75
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-suzuki01/sword/n1-0-1-0.emitter"
        }
      },
      "at": 1.8333,
      "offset": [
        -3.5,
        -2.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-suzuki01/sword/n1-0-1-1.emitter"
        }
      },
      "at": 1.8333,
      "offset": [
        -3.5,
        -2.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-suzuki01/sword/n1-0-1-2.emitter"
        }
      },
      "at": 1.8333,
      "offset": [
        -3.5,
        -2.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-suzuki01/sword/n1-0-1-3-0.effect"
        },
        "width": [
          0.1,
          0.0
        ],
        "lifetime": 0.3333,
        "color": [
          1.0,
          0.047,
          0.0,
          1.0
        ],
        "rise": 36.0
      },
      "at": 1.8333,
      "offset": [
        -3.5,
        -2.0,
        0.0
      ],
      "rotate": [
        30.0,
        -89.972,
        0.0
      ],
      "life": 0.1583
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-suzuki01/sword/n2-0.effect"
        },
        "width": [
          0.2,
          0.05
        ],
        "lifetime": 1.0,
        "color": [
          1.0,
          0.5084,
          0.3321,
          1.0
        ],
        "orbit": [
          0.0,
          1.6667
        ],
        "rise": 6.0
      },
      "at": 2.6667,
      "offset": [
        0.0,
        1.5,
        0.0
      ],
      "life": 0.4833
    }
  ],
  "duration": 4.3333,
  "looping": true
});
