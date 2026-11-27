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
            "inner": 1.0,
            "band": 0.5,
            "height": [
              1.1765,
              -1.1765
            ],
            "colors": [
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Undine6/n0.effect"
        },
        "scale": 8.5,
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
        64.0
      ],
      "life": 1.8333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.0,
            "band": 0.5,
            "height": [
              1.1111,
              -1.1111
            ],
            "colors": [
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Undine6/n1.effect"
        },
        "scale": 9.0,
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
      "life": 1.8333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.0,
            "band": 0.5,
            "height": [
              1.0526,
              -1.0526
            ],
            "colors": [
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Undine6/n2.effect"
        },
        "scale": 9.5,
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
      "life": 1.8333
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
  "duration": 1.8333,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
