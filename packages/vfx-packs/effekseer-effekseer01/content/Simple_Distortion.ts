import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0 "Ring": ring unlimited instances: the first 4 drawn
// effekseer: node 1.0 "Particle": a child of an instanced parent: its motion drawn up the world's y
// effekseer: node 2 "Distortion": ring unlimited instances: the first 4 drawn
// effekseer: node 3 "Base": ring unlimited instances: the first 4 drawn
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.75,
            "band": 0.5,
            "colors": [
              [
                0.3889,
                0.8751,
                0.5989,
                0.0
              ],
              [
                0.1361,
                0.7227,
                0.2271,
                1.0
              ],
              [
                0.3889,
                0.8751,
                0.4762,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Simple_Distortion/n0.effect"
        },
        "scale": 4.0,
        "spin": [
          0.0,
          0.0,
          180.0
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
      "offset": [
        0.0,
        4.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -135.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.75,
            "band": 0.5,
            "colors": [
              [
                0.3889,
                0.8751,
                0.5989,
                0.0
              ],
              [
                0.1361,
                0.7227,
                0.2271,
                1.0
              ],
              [
                0.3889,
                0.8751,
                0.4762,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Simple_Distortion/n0.effect"
        },
        "scale": 4.0,
        "spin": [
          0.0,
          0.0,
          180.0
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
      "offset": [
        0.0,
        4.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -45.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.75,
            "band": 0.5,
            "colors": [
              [
                0.3889,
                0.8751,
                0.5989,
                0.0
              ],
              [
                0.1361,
                0.7227,
                0.2271,
                1.0
              ],
              [
                0.3889,
                0.8751,
                0.4762,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Simple_Distortion/n0.effect"
        },
        "scale": 4.0,
        "spin": [
          0.0,
          0.0,
          180.0
        ],
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
        4.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        45.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.75,
            "band": 0.5,
            "colors": [
              [
                0.3889,
                0.8751,
                0.5989,
                0.0
              ],
              [
                0.1361,
                0.7227,
                0.2271,
                1.0
              ],
              [
                0.3889,
                0.8751,
                0.4762,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Simple_Distortion/n0.effect"
        },
        "scale": 4.0,
        "spin": [
          0.0,
          0.0,
          180.0
        ],
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
        4.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        135.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Simple_Distortion/n1-0.parent"
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
                1.0
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
          "$ogRef": "base/vfx/effekseer-effekseer01/Simple_Distortion/n2.effect"
        },
        "scale": 3.6,
        "spin": [
          0.0,
          0.0,
          120.0
        ],
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
        0.0,
        4.0,
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
                1.0
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
          "$ogRef": "base/vfx/effekseer-effekseer01/Simple_Distortion/n2.effect"
        },
        "scale": 3.6,
        "spin": [
          0.0,
          0.0,
          120.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.3333,
      "offset": [
        0.0,
        4.0,
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
                1.0
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
          "$ogRef": "base/vfx/effekseer-effekseer01/Simple_Distortion/n2.effect"
        },
        "scale": 3.6,
        "spin": [
          0.0,
          0.0,
          120.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.6667,
      "offset": [
        0.0,
        4.0,
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
                1.0
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
          "$ogRef": "base/vfx/effekseer-effekseer01/Simple_Distortion/n2.effect"
        },
        "scale": 3.6,
        "spin": [
          0.0,
          0.0,
          120.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 1.0,
      "offset": [
        0.0,
        4.0,
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
            "inner": 0.9,
            "band": 0.5,
            "colors": [
              [
                0.1247,
                0.8671,
                0.2888,
                0.0
              ],
              [
                0.5356,
                0.8913,
                0.3993,
                1.0
              ],
              [
                0.2549,
                0.8671,
                0.2888,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Simple_Distortion/n3.effect"
        },
        "scale": 4.0,
        "spin": [
          0.0,
          0.0,
          120.0
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
            "inner": 0.9,
            "band": 0.5,
            "colors": [
              [
                0.1247,
                0.8671,
                0.2888,
                0.0
              ],
              [
                0.5356,
                0.8913,
                0.3993,
                1.0
              ],
              [
                0.2549,
                0.8671,
                0.2888,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Simple_Distortion/n3.effect"
        },
        "scale": 4.0,
        "spin": [
          0.0,
          0.0,
          120.0
        ],
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
            "inner": 0.9,
            "band": 0.5,
            "colors": [
              [
                0.1247,
                0.8671,
                0.2888,
                0.0
              ],
              [
                0.5356,
                0.8913,
                0.3993,
                1.0
              ],
              [
                0.2549,
                0.8671,
                0.2888,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Simple_Distortion/n3.effect"
        },
        "scale": 4.0,
        "spin": [
          0.0,
          0.0,
          120.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.6667,
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
            "inner": 0.9,
            "band": 0.5,
            "colors": [
              [
                0.1247,
                0.8671,
                0.2888,
                0.0
              ],
              [
                0.5356,
                0.8913,
                0.3993,
                1.0
              ],
              [
                0.2549,
                0.8671,
                0.2888,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Simple_Distortion/n3.effect"
        },
        "scale": 4.0,
        "spin": [
          0.0,
          0.0,
          120.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 1.0,
      "rotate": [
        -90.0,
        0.0,
        135.0
      ],
      "life": 0.6667
    }
  ],
  "duration": 2.9167,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
