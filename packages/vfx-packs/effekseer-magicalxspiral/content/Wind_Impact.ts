import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 6.0.0.0 "Node": nested generation: 32 instances above its generator folded into one emitter
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
                0.251
              ],
              [
                1.0,
                1.0,
                1.0,
                0.251
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Wind_Impact/n0.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.4
            ],
            [
              1.0,
              1.8
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "subtractive"
      },
      "at": 0.05,
      "life": 0.25
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6111,
            "band": 0.5,
            "colors": [
              [
                0.5234,
                1.0,
                0.5356,
                0.0
              ],
              [
                0.5234,
                1.0,
                0.5356,
                0.3765
              ],
              [
                0.5234,
                1.0,
                0.5356,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Wind_Impact/n1.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              3.24
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
      "at": 0.0,
      "life": 0.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Wind_Impact/n2.emitter"
        }
      },
      "at": 0.0833
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Wind_Impact/n3.emitter"
        }
      },
      "at": 0.05
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Wind_Impact/n4.emitter"
        }
      },
      "at": 0.0833
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Wind_Impact/n5.emitter"
        }
      },
      "at": 0.0333
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Wind_Impact/n6-0-0-0.effect"
        },
        "width": [
          0.0,
          0.0
        ],
        "lifetime": 0.1333,
        "color": [
          1.0,
          1.0,
          1.0,
          0.0
        ],
        "orbit": [
          0.0,
          15.0
        ]
      },
      "at": 0.0,
      "life": 0.2333
    },
    {
      "node": {
        "kind": "light",
        "color": [
          0.0478,
          1.0,
          0.0478
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
      "at": 0.0833,
      "life": 0.3
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.0833
    }
  ],
  "duration": 0.4,
  "events": [
    {
      "name": "impact",
      "time": 0.0833
    }
  ]
});
