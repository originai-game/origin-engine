import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 1.0.0 "爆発中心": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 1.1.0 "爆発つぶ": ring 30 instances: the first 4 drawn
// effekseer: node 1.1.0.0 "Node": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.1.0.0 "Node": nested generation: 30 instances above its generator folded into one emitter
// effekseer: node 1.1.0.0 "Node": a fixed sprite turned at random: its mean plane drawn
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword2/n0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        2.5,
        4.3301,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        150.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword2/n1-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.1125,
        0.1125,
        0.1125
      ],
      "rotate": [
        29.4966,
        -13.7867,
        -49.3338
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.8,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                0.3112,
                0.0037,
                0.2941
              ],
              [
                0.9828,
                0.3112,
                0.0037,
                0.3922
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword2/n1-1-0.effect"
        },
        "faceCamera": true,
        "scale": 0.15,
        "color": [
          7.5,
          7.5,
          7.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1,
      "offset": [
        0.375,
        0.0,
        0.0
      ],
      "rotate": [
        -54.7356,
        30.0,
        80.2644
      ],
      "velocity": [
        3.629,
        0.0,
        0.0
      ],
      "life": 0.5167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.8,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                0.3112,
                0.0037,
                0.2941
              ],
              [
                0.9828,
                0.3112,
                0.0037,
                0.3922
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword2/n1-1-0.effect"
        },
        "faceCamera": true,
        "scale": 0.15,
        "color": [
          7.5,
          7.5,
          7.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1017,
      "offset": [
        0.375,
        0.0,
        0.0
      ],
      "rotate": [
        -54.7356,
        -30.0,
        -80.2644
      ],
      "velocity": [
        3.629,
        0.0,
        0.0
      ],
      "life": 0.5167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.8,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                0.3112,
                0.0037,
                0.2941
              ],
              [
                0.9828,
                0.3112,
                0.0037,
                0.3922
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword2/n1-1-0.effect"
        },
        "faceCamera": true,
        "scale": 0.15,
        "color": [
          7.5,
          7.5,
          7.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1033,
      "offset": [
        0.375,
        0.0,
        0.0
      ],
      "rotate": [
        54.7356,
        30.0,
        9.7356
      ],
      "velocity": [
        3.629,
        0.0,
        0.0
      ],
      "life": 0.5167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.8,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                0.3112,
                0.0037,
                0.2941
              ],
              [
                0.9828,
                0.3112,
                0.0037,
                0.3922
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword2/n1-1-0.effect"
        },
        "faceCamera": true,
        "scale": 0.15,
        "color": [
          7.5,
          7.5,
          7.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.105,
      "offset": [
        0.375,
        0.0,
        0.0
      ],
      "rotate": [
        54.7356,
        -30.0,
        -9.7356
      ],
      "velocity": [
        3.629,
        0.0,
        0.0
      ],
      "life": 0.5167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword2/n1-1-0-0.emitter"
        }
      },
      "at": 0.1,
      "offset": [
        0.375,
        0.0,
        0.0
      ]
    }
  ],
  "duration": 0.6983,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
