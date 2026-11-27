import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0 "Node": ring 6 instances: the first 4 drawn
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
            "start": 150.0,
            "length": 240.0,
            "colors": [
              [
                0.0,
                1.0,
                0.0,
                0.0
              ],
              [
                0.0478,
                1.0,
                0.0478,
                0.502
              ],
              [
                0.0,
                1.0,
                0.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Sylph1/n0-0.effect"
        },
        "scale": 1.0,
        "spin": [
          0.0,
          0.0,
          -780.0
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
        -11.25,
        0.0,
        -180.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.5,
            "start": 150.0,
            "length": 240.0,
            "colors": [
              [
                0.0,
                1.0,
                0.0,
                0.0
              ],
              [
                0.0478,
                1.0,
                0.0478,
                0.502
              ],
              [
                0.0,
                1.0,
                0.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Sylph1/n0-0.effect"
        },
        "scale": 1.0,
        "spin": [
          0.0,
          0.0,
          -780.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0167,
      "rotate": [
        -3.75,
        0.0,
        -180.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.5,
            "start": 150.0,
            "length": 240.0,
            "colors": [
              [
                0.0,
                1.0,
                0.0,
                0.0
              ],
              [
                0.0478,
                1.0,
                0.0478,
                0.502
              ],
              [
                0.0,
                1.0,
                0.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Sylph1/n0-0.effect"
        },
        "scale": 1.0,
        "spin": [
          0.0,
          0.0,
          -780.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0333,
      "rotate": [
        3.75,
        0.0,
        -180.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.5,
            "start": 150.0,
            "length": 240.0,
            "colors": [
              [
                0.0,
                1.0,
                0.0,
                0.0
              ],
              [
                0.0478,
                1.0,
                0.0478,
                0.502
              ],
              [
                0.0,
                1.0,
                0.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Sylph1/n0-0.effect"
        },
        "scale": 1.0,
        "spin": [
          0.0,
          0.0,
          -780.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.05,
      "rotate": [
        11.25,
        0.0,
        -180.0
      ],
      "life": 0.3333
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
        "distance": 8.0
      },
      "at": 0.0,
      "life": 0.3
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.3
      },
      "at": 0.0
    }
  ],
  "duration": 0.3833,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
