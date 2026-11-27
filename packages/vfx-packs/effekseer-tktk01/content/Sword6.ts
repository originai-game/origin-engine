import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0 "メイン斬撃": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 1 "月": ring 30 instances: the first 4 drawn
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword6/n0.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword6/n0-0.parent"
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
            "inner": 0.5,
            "band": 0.5,
            "colors": [
              [
                1.0,
                0.0824,
                0.1511,
                0.0
              ],
              [
                1.0,
                0.0824,
                0.1511,
                0.1961
              ],
              [
                1.0,
                0.0824,
                0.1511,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword6/n1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.4531,
              2.4109
            ],
            [
              1.0,
              2.8284
            ]
          ]
        },
        "color": [
          7.5,
          7.5,
          7.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "rotate": [
        0.0,
        0.0,
        -135.0
      ],
      "life": 0.3167
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
                0.0824,
                0.1511,
                0.0
              ],
              [
                1.0,
                0.0824,
                0.1511,
                0.1961
              ],
              [
                1.0,
                0.0824,
                0.1511,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword6/n1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.4531,
              2.4109
            ],
            [
              1.0,
              2.8284
            ]
          ]
        },
        "color": [
          7.5,
          7.5,
          7.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0033,
      "rotate": [
        0.0,
        0.0,
        -45.0
      ],
      "life": 0.3167
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
                0.0824,
                0.1511,
                0.0
              ],
              [
                1.0,
                0.0824,
                0.1511,
                0.1961
              ],
              [
                1.0,
                0.0824,
                0.1511,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword6/n1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.4531,
              2.4109
            ],
            [
              1.0,
              2.8284
            ]
          ]
        },
        "color": [
          7.5,
          7.5,
          7.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0067,
      "rotate": [
        0.0,
        0.0,
        45.0
      ],
      "life": 0.3167
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
                0.0824,
                0.1511,
                0.0
              ],
              [
                1.0,
                0.0824,
                0.1511,
                0.1961
              ],
              [
                1.0,
                0.0824,
                0.1511,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword6/n1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.4531,
              2.4109
            ],
            [
              1.0,
              2.8284
            ]
          ]
        },
        "color": [
          7.5,
          7.5,
          7.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.01,
      "rotate": [
        0.0,
        0.0,
        135.0
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword6/n2-0.emitter"
        }
      },
      "at": 0.0,
      "rotate": [
        0.0,
        0.0,
        -102.4074
      ]
    }
  ],
  "duration": 0.6817,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
