import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Thunder1/n0.emitter"
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
            "height": [
              0.0,
              -0.1
            ],
            "colors": [
              [
                1.0,
                1.0,
                0.0,
                0.0
              ],
              [
                1.0,
                0.7519,
                0.0,
                0.3765
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Thunder1/n1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              1.0,
              1.75
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
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "life": 1.6667
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
              0.0,
              -0.01
            ],
            "colors": [
              [
                0.0,
                0.0,
                0.0,
                0.0
              ],
              [
                1.0,
                0.7519,
                0.0,
                0.8784
              ],
              [
                0.0,
                0.0,
                0.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Thunder1/n2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.5
            ],
            [
              0.0938,
              1.0584
            ],
            [
              0.2031,
              1.6049
            ],
            [
              0.3125,
              2.0521
            ],
            [
              0.4219,
              2.4145
            ],
            [
              0.5312,
              2.7064
            ],
            [
              0.6562,
              2.972
            ],
            [
              1.0,
              3.5
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          300.0
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
        90.0,
        0.0,
        0.0
      ],
      "life": 1.3333
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
              0.0,
              -0.01
            ],
            "colors": [
              [
                0.0,
                0.0,
                0.0,
                0.0
              ],
              [
                1.0,
                0.7519,
                0.0,
                0.8784
              ],
              [
                0.0,
                0.0,
                0.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Thunder1/n3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.5
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
      "at": 0.1167,
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "life": 1.3333
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
              0.0,
              -0.01
            ],
            "colors": [
              [
                0.0,
                0.0,
                0.0,
                0.0
              ],
              [
                1.0,
                0.7519,
                0.0,
                0.8784
              ],
              [
                0.0,
                0.0,
                0.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Thunder1/n4.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.5
            ],
            [
              0.0938,
              1.0584
            ],
            [
              0.2031,
              1.6049
            ],
            [
              0.3125,
              2.0521
            ],
            [
              0.4219,
              2.4145
            ],
            [
              0.5312,
              2.7064
            ],
            [
              0.6562,
              2.972
            ],
            [
              1.0,
              3.5
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          -300.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.2333,
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "life": 1.3333
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          0.9742,
          0.8974
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
      "life": 0.3
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.0
    }
  ],
  "duration": 2.15,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
