import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 2.0 "Node": a moving ancestor: its descendants are placed where it starts
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
            "height": [
              5.3333,
              0.0
            ],
            "colors": [
              [
                0.2195,
                0.0,
                1.0,
                0.0
              ],
              [
                0.2195,
                0.0,
                1.0,
                1.0
              ],
              [
                0.0,
                0.2195,
                0.5356,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/drill/n0.effect"
        },
        "scale": 3.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "life": 16.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.5,
            "height": [
              3.2,
              0.0
            ],
            "colors": [
              [
                0.0,
                0.2195,
                0.5356,
                0.0
              ],
              [
                0.0,
                0.2195,
                0.5356,
                0.502
              ],
              [
                0.0,
                1.0,
                1.0,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/drill/n1.effect"
        },
        "scale": 2.5,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "life": 16.6667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/drill/n2-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.0,
        5.0
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.5,
            "height": [
              2.0,
              -0.0833
            ],
            "colors": [
              [
                0.0,
                0.2195,
                0.5356,
                0.0
              ],
              [
                0.0,
                0.2195,
                0.5356,
                0.502
              ],
              [
                0.0,
                0.2195,
                1.0,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/drill/n3.effect"
        },
        "scale": 6.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "life": 16.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.5,
            "height": [
              2.0,
              -0.0833
            ],
            "colors": [
              [
                0.0,
                0.0,
                0.0,
                0.0
              ],
              [
                0.0,
                0.0,
                0.0,
                0.502
              ],
              [
                1.0,
                0.2195,
                0.0478,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/drill/n4.effect"
        },
        "scale": 6.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "subtractive"
      },
      "at": 0.0,
      "life": 16.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.5,
            "height": [
              2.0,
              -0.0833
            ],
            "colors": [
              [
                0.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.2195,
                0.0,
                1.0,
                0.502
              ],
              [
                1.0,
                0.2195,
                0.0478,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/drill/n5.effect"
        },
        "scale": 6.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "life": 16.6667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/drill/n6.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        1.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/drill/n7.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        1.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.1333,
            "band": 0.5,
            "height": [
              2.0,
              0.0
            ],
            "colors": [
              [
                0.0619,
                0.0,
                0.2888,
                0.0
              ],
              [
                0.482,
                0.0023,
                0.7519,
                0.6745
              ],
              [
                0.0,
                1.0,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/drill/n8.effect"
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
      "at": 0.0,
      "life": 16.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.1333,
            "band": 0.5,
            "height": [
              2.0,
              0.0
            ],
            "colors": [
              [
                0.0478,
                0.0,
                0.2195,
                0.0
              ],
              [
                0.0029,
                0.1734,
                0.7372,
                0.502
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
          "$ogRef": "base/vfx/effekseer-andrewfm01/drill/n9.effect"
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
      "at": 0.0,
      "life": 16.6667
    }
  ],
  "duration": 10.0,
  "looping": true
});
