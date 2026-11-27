import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 0.858299970626831,
  "looping": false,
  "events": [
    {
      "name": "impact",
      "time": 0.33329999446868896
    }
  ],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 0.5,
      "offset": [
        0,
        14.14210033416748,
        14.14210033416748
      ],
      "rotate": [
        45,
        0,
        0
      ],
      "velocity": [
        0,
        -28.284299850463867,
        -28.284299850463867
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "mesh",
        "blend": "alpha",
        "color": [
          1,
          0.21950000524520874,
          0.21950000524520874,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/meteo1/n0-0-0.effect"),
        "faceCamera": false,
        "scale": 4,
        "shape": {
          "kind": "model",
          "asset": ref("base/vfx/effekseer-starter/meteo1/model-block1")
        },
        "spin": [
          0,
          0,
          0
        ]
      }
    },
    {
      "at": 0,
      "life": 0.33329999446868896,
      "offset": [
        0,
        14.14210033416748,
        14.14210033416748
      ],
      "rotate": [
        45,
        0,
        0
      ],
      "velocity": [
        0,
        -28.284299850463867,
        -28.284299850463867
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "trail",
        "color": [
          0.21950000524520874,
          0.21950000524520874,
          0,
          0
        ],
        "effect": ref("base/vfx/effekseer-starter/meteo1/n0-0-1-0.effect"),
        "lifetime": 0.33329999446868896,
        "orbit": [
          1.5,
          5
        ],
        "rise": 0,
        "width": [
          1,
          1
        ]
      }
    },
    {
      "at": 0,
      "life": 0.5,
      "offset": [
        0,
        14.849200248718262,
        14.849200248718262
      ],
      "rotate": [
        -45,
        0,
        0
      ],
      "velocity": [
        0,
        -28.284299850463867,
        -28.284299850463867
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "mesh",
        "blend": "additive",
        "color": [
          1,
          1,
          1,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/meteo1/n0-0-2.effect"),
        "faceCamera": false,
        "scale": 2,
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0.5,
            "colors": [
              [
                1,
                1,
                0,
                0
              ],
              [
                1,
                0.21950000524520874,
                0,
                0.7529000043869019
              ],
              [
                1,
                1,
                0,
                0
              ]
            ],
            "height": [
              0,
              0
            ],
            "inner": 0.5,
            "length": 120,
            "segments": 64,
            "start": 210
          }
        },
        "spin": [
          0,
          0,
          0
        ]
      }
    },
    {
      "at": 0,
      "life": 0.5,
      "offset": [
        0,
        14.849200248718262,
        14.849200248718262
      ],
      "rotate": [
        -45,
        0,
        -180
      ],
      "velocity": [
        0,
        -28.284299850463867,
        -28.284299850463867
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "mesh",
        "blend": "additive",
        "color": [
          1,
          1,
          1,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/meteo1/n0-0-3.effect"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              10
            ],
            [
              1,
              0
            ]
          ]
        },
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0.5,
            "colors": [
              [
                1,
                1,
                0,
                0
              ],
              [
                1,
                0.21950000524520874,
                0,
                0.7529000043869019
              ],
              [
                1,
                1,
                0,
                0
              ]
            ],
            "height": [
              0,
              0
            ],
            "inner": 0.5,
            "length": 120,
            "segments": 64,
            "start": 210
          }
        },
        "spin": [
          0,
          0,
          0
        ]
      }
    },
    {
      "at": 0,
      "life": 0.666700005531311,
      "offset": [
        0,
        14.14210033416748,
        14.14210033416748
      ],
      "rotate": [
        45,
        0,
        0
      ],
      "velocity": [
        0,
        -28.284299850463867,
        -28.284299850463867
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "mesh",
        "blend": "additive",
        "color": [
          1,
          1,
          1,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/meteo1/n0-0-4.effect"),
        "faceCamera": true,
        "scale": 2,
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0.5,
            "colors": [
              [
                1,
                1,
                0,
                0
              ],
              [
                1,
                0,
                0,
                0.7529000043869019
              ],
              [
                1,
                0,
                0.21950000524520874,
                1
              ]
            ],
            "height": [
              0,
              0
            ],
            "inner": 0,
            "length": 360,
            "segments": 64,
            "start": 0
          }
        },
        "spin": [
          0,
          0,
          0
        ]
      }
    },
    {
      "at": 0,
      "life": 0.33329999446868896,
      "offset": [
        0,
        14.14210033416748,
        14.14210033416748
      ],
      "rotate": [
        45,
        0,
        0
      ],
      "velocity": [
        0,
        -28.284299850463867,
        -28.284299850463867
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "trail",
        "color": [
          1,
          0.21950000524520874,
          0,
          0.5019999742507935
        ],
        "effect": ref("base/vfx/effekseer-starter/meteo1/n0-0-5-0.effect"),
        "lifetime": 0.33329999446868896,
        "orbit": [
          1.5,
          8.33329963684082
        ],
        "rise": 0,
        "width": [
          1,
          0.25
        ]
      }
    },
    {
      "at": 0.16670000553131104,
      "life": 0.16670000553131104,
      "offset": [
        0,
        14.14210033416748,
        14.14210033416748
      ],
      "rotate": [
        -45,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "mesh",
        "blend": "additive",
        "color": [
          1,
          1,
          1,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/meteo1/n0-0-6.effect"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              1
            ],
            [
              1,
              10
            ]
          ]
        },
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0.5,
            "colors": [
              [
                1,
                1,
                0,
                0
              ],
              [
                1,
                0,
                0,
                0.7529000043869019
              ],
              [
                1,
                0,
                0,
                0
              ]
            ],
            "height": [
              0,
              0
            ],
            "inner": 0.25,
            "length": 360,
            "segments": 64,
            "start": 0
          }
        },
        "spin": [
          0,
          0,
          0
        ]
      }
    },
    {
      "at": 0.33329999446868896,
      "life": 0.16670000553131104,
      "offset": [
        0,
        14.14210033416748,
        14.14210033416748
      ],
      "rotate": [
        -45,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "mesh",
        "blend": "additive",
        "color": [
          1,
          1,
          1,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/meteo1/n0-0-7.effect"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              1
            ],
            [
              1,
              10
            ]
          ]
        },
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0.5,
            "colors": [
              [
                1,
                1,
                0,
                0
              ],
              [
                1,
                0,
                0,
                0.7529000043869019
              ],
              [
                1,
                0,
                0,
                0
              ]
            ],
            "height": [
              0,
              0
            ],
            "inner": 0.25,
            "length": 360,
            "segments": 64,
            "start": 0
          }
        },
        "spin": [
          0,
          0,
          0
        ]
      }
    },
    {
      "at": 0.44999998807907104,
      "life": 0.33329999446868896,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        18,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "mesh",
        "blend": "additive",
        "color": [
          15,
          3.292799949645996,
          0.033900000154972076,
          0.7529000043869019
        ],
        "effect": ref("base/vfx/effekseer-starter/meteo1/n1-0.effect"),
        "faceCamera": false,
        "scale": 1,
        "shape": {
          "kind": "model",
          "asset": ref("base/vfx/effekseer-starter/meteo1/model-block2")
        },
        "spin": [
          0,
          0,
          0
        ]
      }
    },
    {
      "at": 0.41670000553131104,
      "life": 0.25,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "mesh",
        "blend": "additive",
        "color": [
          1,
          1,
          1,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/meteo1/n2.effect"),
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0
            ],
            [
              1,
              11.25
            ]
          ]
        },
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0.5,
            "colors": [
              [
                1,
                0,
                0,
                0
              ],
              [
                1,
                1,
                0.21950000524520874,
                0.7529000043869019
              ],
              [
                1,
                0,
                0,
                0
              ]
            ],
            "height": [
              0,
              0
            ],
            "inner": 0.33329999446868896,
            "length": 360,
            "segments": 64,
            "start": 0
          }
        },
        "spin": [
          0,
          0,
          0
        ]
      }
    },
    {
      "at": 0.44999998807907104,
      "life": 0,
      "offset": [
        0,
        -0.5,
        0
      ],
      "rotate": [
        -180,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/effekseer-starter/meteo1/n3-0.emitter")
      }
    },
    {
      "at": 0.33329999446868896,
      "life": 0.16670000553131104,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        125.26439666748047,
        -30,
        9.735600471496582
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "mesh",
        "blend": "additive",
        "color": [
          1,
          1,
          1,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/meteo1/n4.effect"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              15
            ],
            [
              0.07810000330209732,
              12.688300132751465
            ],
            [
              0.15620000660419464,
              10.746600151062012
            ],
            [
              0.23440000414848328,
              9.125800132751465
            ],
            [
              0.3280999958515167,
              7.535200119018555
            ],
            [
              0.46880000829696655,
              5.6975998878479
            ],
            [
              0.8281000256538391,
              2.158099889755249
            ],
            [
              1,
              0
            ]
          ]
        },
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0.5,
            "colors": [
              [
                1,
                0,
                0,
                0
              ],
              [
                0.21950000524520874,
                0.21950000524520874,
                0,
                0.5019999742507935
              ],
              [
                1,
                0,
                0,
                0
              ]
            ],
            "height": [
              -0.06669999659061432,
              0.06669999659061432
            ],
            "inner": 0.33329999446868896,
            "length": 360,
            "segments": 64,
            "start": 0
          }
        },
        "spin": [
          0,
          0,
          0
        ]
      }
    },
    {
      "at": 0.33329999446868896,
      "life": 0.30000001192092896,
      "offset": [
        0,
        4.351399898529053,
        4.351399898529053
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "light",
        "color": [
          1,
          1,
          1
        ],
        "distance": 12,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              8
            ],
            [
              1,
              0
            ]
          ]
        }
      }
    },
    {
      "at": 0.33329999446868896,
      "life": 0,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "shake",
        "trauma": 0.4000000059604645
      }
    }
  ]
});
