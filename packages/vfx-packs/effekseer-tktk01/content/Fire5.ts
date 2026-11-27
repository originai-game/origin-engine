import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0 "剣からの炎": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.0 "剣からの炎": nested generation: 25 instances above its generator folded into one emitter
// effekseer: node 0.0.0.1 "剣からの炎尻尾": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.1 "剣からの炎尻尾": nested generation: 25 instances above its generator folded into one emitter
// effekseer: node 0.0.0.2 "着弾リング": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.2 "着弾リング": ring 10 instances: the first 4 drawn
// effekseer: node 0.0.0.2 "着弾リング": nested generation: 25 instances above its generator folded into one emitter
// effekseer: node 0.0.0.3 "炎上がる": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.3 "炎上がる": nested generation: 25 instances above its generator folded into one emitter
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Fire5/n0-0-0-0.emitter"
        }
      },
      "at": 0.3333,
      "offset": [
        0.0,
        1.5556,
        29.1111
      ],
      "rotate": [
        -90.0,
        89.972,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Fire5/n0-0-0-1.emitter"
        }
      },
      "at": 0.3333,
      "offset": [
        0.0,
        1.5556,
        29.1111
      ],
      "rotate": [
        -90.0,
        89.972,
        0.0
      ]
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
                1.0,
                1.0,
                1.0,
                0.0
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
          "$ogRef": "base/vfx/effekseer-tktk01/Fire5/n0-0-0-2.effect"
        },
        "faceCamera": true,
        "scale": 1.7778,
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.6667,
      "offset": [
        0.0,
        0.0,
        25.7778
      ],
      "rotate": [
        54.7356,
        30.0,
        -170.2644
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
                0.0
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
          "$ogRef": "base/vfx/effekseer-tktk01/Fire5/n0-0-0-2.effect"
        },
        "faceCamera": true,
        "scale": 1.7778,
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.6668,
      "offset": [
        0.0,
        0.0,
        25.7778
      ],
      "rotate": [
        125.2644,
        30.0,
        -9.7356
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
                0.0
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
          "$ogRef": "base/vfx/effekseer-tktk01/Fire5/n0-0-0-2.effect"
        },
        "faceCamera": true,
        "scale": 1.7778,
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.667,
      "offset": [
        0.0,
        0.0,
        25.7778
      ],
      "rotate": [
        -54.7356,
        30.0,
        -99.7356
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
                0.0
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
          "$ogRef": "base/vfx/effekseer-tktk01/Fire5/n0-0-0-2.effect"
        },
        "faceCamera": true,
        "scale": 1.7778,
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.6672,
      "offset": [
        0.0,
        0.0,
        25.7778
      ],
      "rotate": [
        -125.2644,
        30.0,
        -80.2644
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Fire5/n0-0-0-3.emitter"
        }
      },
      "at": 0.6167,
      "offset": [
        0.0,
        0.0,
        25.7778
      ],
      "rotate": [
        90.0,
        89.972,
        0.0
      ]
    }
  ],
  "duration": 1.4417,
  "events": [
    {
      "name": "impact",
      "time": 0.6167
    }
  ]
});
