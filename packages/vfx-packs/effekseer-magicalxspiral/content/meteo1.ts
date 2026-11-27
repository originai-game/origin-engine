import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.1.0 "Node": a track whose points move on their own: drawn along its generator's path
// effekseer: node 0.0.1.0 "Node": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 0.0.6 "Node": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.7 "Node": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.0 "Node": model 15 instances: the first 4 drawn
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-magicalxspiral/meteo1/model-block1"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/meteo1/n0-0-0.effect"
        },
        "scale": 4.0,
        "color": [
          1.0,
          0.2195,
          0.2195,
          1.0
        ],
        "blend": "alpha"
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
      "life": 0.5
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/meteo1/n0-0-1-0.effect"
        },
        "width": [
          1.0,
          1.0
        ],
        "lifetime": 0.3333,
        "color": [
          0.2195,
          0.2195,
          0.0,
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/meteo1/n0-0-2.effect"
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/meteo1/n0-0-3.effect"
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
                0.0,
                0.0
              ],
              [
                1.0,
                0.0,
                0.0,
                0.7529
              ],
              [
                1.0,
                0.0,
                0.2195,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/meteo1/n0-0-4.effect"
        },
        "faceCamera": true,
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
      "life": 0.6667
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/meteo1/n0-0-5-0.effect"
        },
        "width": [
          1.0,
          0.25
        ],
        "lifetime": 0.3333,
        "color": [
          1.0,
          0.2195,
          0.0,
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
                0.0,
                0.0
              ],
              [
                1.0,
                0.0,
                0.0,
                0.7529
              ],
              [
                1.0,
                0.0,
                0.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/meteo1/n0-0-6.effect"
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
                1.0,
                1.0,
                0.0,
                0.0
              ],
              [
                1.0,
                0.0,
                0.0,
                0.7529
              ],
              [
                1.0,
                0.0,
                0.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/meteo1/n0-0-7.effect"
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
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-magicalxspiral/meteo1/model-block2"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/meteo1/n1-0.effect"
        },
        "scale": 1.0,
        "color": [
          15.0,
          3.2928,
          0.0339,
          0.7529
        ],
        "blend": "additive"
      },
      "at": 0.45,
      "velocity": [
        0.0,
        18.0,
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
            "inner": 0.3333,
            "band": 0.5,
            "colors": [
              [
                1.0,
                0.0,
                0.0,
                0.0
              ],
              [
                1.0,
                1.0,
                0.2195,
                0.7529
              ],
              [
                1.0,
                0.0,
                0.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/meteo1/n2.effect"
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/meteo1/n3-0.emitter"
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
            "inner": 0.3333,
            "band": 0.5,
            "height": [
              -0.0667,
              0.0667
            ],
            "colors": [
              [
                1.0,
                0.0,
                0.0,
                0.0
              ],
              [
                0.2195,
                0.2195,
                0.0,
                0.502
              ],
              [
                1.0,
                0.0,
                0.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/meteo1/n4.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              15.0
            ],
            [
              0.0781,
              12.6883
            ],
            [
              0.1562,
              10.7466
            ],
            [
              0.2344,
              9.1258
            ],
            [
              0.3281,
              7.5352
            ],
            [
              0.4688,
              5.6976
            ],
            [
              0.8281,
              2.1581
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
      "at": 0.3333,
      "rotate": [
        125.2644,
        -30.0,
        9.7356
      ],
      "life": 0.1667
    }
  ],
  "duration": 0.8583,
  "events": [
    {
      "name": "impact",
      "time": 0.3333
    }
  ]
});
