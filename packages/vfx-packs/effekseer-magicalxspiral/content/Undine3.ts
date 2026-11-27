import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
export default file.vfx({
  "elements": [
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
                0.3587,
                0.5356,
                1.0,
                0.6275
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Undine3/n0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              0.6719,
              1.2015
            ],
            [
              0.8438,
              1.445
            ],
            [
              1.0,
              1.6
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
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.75,
            "band": 0.5,
            "height": [
              0.5,
              0.0
            ],
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.2195,
                0.3587,
                1.0,
                0.6275
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Undine3/n1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              0.6719,
              1.2015
            ],
            [
              0.8438,
              1.445
            ],
            [
              1.0,
              1.6
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
      "at": 0.05,
      "offset": [
        0.0,
        -0.1,
        0.0
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 0.4167
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
                0.3587,
                0.5356,
                1.0,
                0.6275
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Undine3/n2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              0.6719,
              1.2015
            ],
            [
              0.8438,
              1.445
            ],
            [
              1.0,
              1.6
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
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.75,
            "band": 0.5,
            "height": [
              0.5,
              0.0
            ],
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.5356,
                0.7519,
                1.0,
                0.6275
              ],
              [
                0.0,
                0.0002,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Undine3/n3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              0.6719,
              1.2015
            ],
            [
              0.8438,
              1.445
            ],
            [
              1.0,
              1.6
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
        -0.1,
        0.0
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          1.0,
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
        0.01,
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
  "duration": 0.5833,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
