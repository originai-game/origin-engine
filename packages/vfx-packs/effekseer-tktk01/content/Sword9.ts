import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 1 "始めリング": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 5.0 "煙メイン": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 6 "始めリング": ring 5 instances: the first 4 drawn
// effekseer: node 6 "始めリング": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 7 "始めリング": ring 5 instances: the first 4 drawn
// effekseer: node 7 "始めリング": a ring whose thickness changes over its life: its middle thickness kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword9/n0.emitter"
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
            "inner": 1.2122,
            "band": 0.5,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.9658,
                0.0,
                0.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword9/n1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0938,
              1.3722
            ],
            [
              0.2031,
              1.7366
            ],
            [
              0.4219,
              2.2763
            ],
            [
              0.6562,
              2.648
            ],
            [
              1.0,
              3.0
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
        -1.0,
        0.0
      ],
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword9/n2.emitter"
        }
      },
      "at": 0.3333,
      "offset": [
        0.0,
        7.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        180.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword9/n3-0.emitter"
        }
      },
      "at": 0.0333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword9/n4-0.emitter"
        }
      },
      "at": 0.35,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword9/n5-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.75,
        -1.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.25,
            "band": 0.5,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.9658,
                0.1275,
                0.1275,
                0.3922
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword9/n6.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0938,
              1.7445
            ],
            [
              0.2031,
              2.4732
            ],
            [
              0.3125,
              3.0695
            ],
            [
              0.4219,
              3.5527
            ],
            [
              0.6562,
              4.296
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          5.0,
          5.0,
          5.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.3667,
      "offset": [
        0.0,
        1.0,
        0.0
      ],
      "rotate": [
        90.0,
        -30.0,
        90.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.25,
            "band": 0.5,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.9658,
                0.1275,
                0.1275,
                0.3922
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword9/n7.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0938,
              1.7445
            ],
            [
              0.2031,
              2.4732
            ],
            [
              0.3125,
              3.0695
            ],
            [
              0.4219,
              3.5527
            ],
            [
              0.6562,
              4.296
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          5.0,
          5.0,
          5.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.3667,
      "offset": [
        0.0,
        1.0,
        0.0
      ],
      "rotate": [
        -90.0,
        -30.0,
        -90.0
      ],
      "life": 0.5
    }
  ],
  "duration": 0.8667,
  "events": [
    {
      "name": "impact",
      "time": 0.3667
    }
  ]
});
