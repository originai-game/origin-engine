import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 1.0 "Node": model 15 instances: the first 4 drawn
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem11/model-block1"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem11/n0-0-0.effect"
        },
        "scale": 2.0,
        "color": [
          1.0,
          1.0,
          0.2195,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.0,
      "offset": [
        0.0,
        14.1421,
        14.1421
      ],
      "rotate": [
        45.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        -28.2843,
        -28.2843
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem11/model-block2"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem11/n1-0.effect"
        },
        "scale": 1.0,
        "color": [
          15.0,
          3.2928,
          0.0339,
          0.7529
        ],
        "blend": "additive"
      },
      "at": 0.45,
      "velocity": [
        0.0,
        18.0,
        0.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.3333,
            "band": 0.5,
            "colors": [
              [
                1.0,
                0.0,
                0.0,
                0.0
              ],
              [
                1.0,
                1.0,
                0.2195,
                0.7529
              ],
              [
                1.0,
                0.0,
                0.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem11/n2.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              1.0,
              11.25
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
      "at": 0.4167,
      "life": 0.25
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem11/n3-0.emitter"
        }
      },
      "at": 0.45,
      "offset": [
        0.0,
        -0.5,
        0.0
      ],
      "rotate": [
        -180.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.3333,
            "band": 0.5,
            "height": [
              -0.0667,
              0.0667
            ],
            "colors": [
              [
                1.0,
                0.0,
                0.0,
                0.0
              ],
              [
                0.2195,
                0.2195,
                0.0,
                0.502
              ],
              [
                1.0,
                0.0,
                0.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem11/n4.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              15.0
            ],
            [
              0.0781,
              12.6883
            ],
            [
              0.1562,
              10.7466
            ],
            [
              0.2344,
              9.1258
            ],
            [
              0.3281,
              7.5352
            ],
            [
              0.4688,
              5.6976
            ],
            [
              0.8281,
              2.1581
            ],
            [
              1.0,
              0.0
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
        125.2644,
        -30.0,
        9.7356
      ],
      "life": 0.1667
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
      "at": 0.3333,
      "life": 0.3
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.3333
    }
  ],
  "duration": 0.8583,
  "events": [
    {
      "name": "impact",
      "time": 0.3333
    }
  ]
});
