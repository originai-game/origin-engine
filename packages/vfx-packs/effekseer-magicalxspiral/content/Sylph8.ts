import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 1.0 "Node": ring 3 instances: the first 4 drawn
// effekseer: node 1.0 "Node": nested generation: 2 instances above its generator folded into one emitter
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.3846,
            "band": 0.5,
            "height": [
              0.7692,
              -0.1538
            ],
            "colors": [
              [
                0.0,
                0.2195,
                0.0,
                0.0
              ],
              [
                0.2195,
                1.0,
                0.5356,
                0.502
              ],
              [
                0.2195,
                1.0,
                0.2195,
                0.502
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Sylph8/n0-0.effect"
        },
        "scale": 1.3,
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
        1.0
      ],
      "life": 0.8333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.4444,
            "band": 0.5,
            "height": [
              1.6667,
              -0.2222
            ],
            "colors": [
              [
                0.0,
                0.2195,
                0.0,
                0.0
              ],
              [
                0.2195,
                1.0,
                0.5356,
                0.502
              ],
              [
                0.2195,
                1.0,
                0.2195,
                0.502
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Sylph8/n0-1.effect"
        },
        "scale": 0.9,
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
        1.0
      ],
      "life": 0.8333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.5,
            "start": 150.0,
            "length": 240.0,
            "colors": [
              [
                0.0,
                0.0,
                0.0,
                0.0
              ],
              [
                0.2195,
                1.0,
                0.2195,
                0.502
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Sylph8/n1-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.6
            ],
            [
              1.0,
              0.8
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
        -100.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        2.4,
        0.0
      ],
      "life": 0.8333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.5,
            "start": 150.0,
            "length": 240.0,
            "colors": [
              [
                0.0,
                0.0,
                0.0,
                0.0
              ],
              [
                0.2195,
                1.0,
                0.2195,
                0.502
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Sylph8/n1-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.6
            ],
            [
              1.0,
              0.8
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
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        2.4,
        0.0
      ],
      "life": 0.8333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.5,
            "start": 150.0,
            "length": 240.0,
            "colors": [
              [
                0.0,
                0.0,
                0.0,
                0.0
              ],
              [
                0.2195,
                1.0,
                0.2195,
                0.502
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Sylph8/n1-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.6
            ],
            [
              1.0,
              0.8
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
        -80.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        2.4,
        0.0
      ],
      "life": 0.8333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9,
            "band": 0.1,
            "height": [
              4.0,
              0.0
            ],
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
                0.2195,
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Sylph8/n2.effect"
        },
        "scale": 0.5,
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
        0.0
      ],
      "life": 0.8333
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
        "distance": 10.4
      },
      "at": 0.0,
      "life": 0.3
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.36
      },
      "at": 0.0
    }
  ],
  "duration": 1.1667,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
