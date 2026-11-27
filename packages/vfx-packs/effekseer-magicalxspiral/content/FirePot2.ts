import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0 "Fire1": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 0.0.0.1 "Fire2": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 0.0.0.2 "Fire3": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 0.0.1.0 "Node": a ring whose thickness changes over its life: its middle thickness kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/FirePot2/n0-0-0-0.effect"
        },
        "scale": 3.4064,
        "color": [
          1.0,
          1.0,
          1.0,
          0.502
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        0.0,
        2.4667,
        0.0
      ],
      "life": 1.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/FirePot2/n0-0-0-1.effect"
        },
        "scale": 3.4064,
        "color": [
          1.0,
          1.0,
          1.0,
          0.502
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        0.0,
        2.4667,
        0.0
      ],
      "rotate": [
        0.0,
        60.0,
        0.0
      ],
      "life": 1.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/FirePot2/n0-0-0-2.effect"
        },
        "scale": 3.4064,
        "color": [
          1.0,
          1.0,
          1.0,
          0.502
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        0.0,
        2.4667,
        0.0
      ],
      "rotate": [
        -180.0,
        60.0,
        -180.0
      ],
      "life": 1.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 16.0,
            "band": 0.5,
            "height": [
              40000.0,
              0.0
            ],
            "colors": [
              [
                1.0,
                0.2195,
                0.0,
                0.0
              ],
              [
                1.0,
                0.2195,
                0.0,
                0.502
              ],
              [
                1.0,
                0.2195,
                0.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/FirePot2/n0-0-1-0.effect"
        },
        "scale": 0.0,
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
        0.3,
        0.0
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 1.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.3,
            "band": 0.5,
            "height": [
              2.0,
              0.0
            ],
            "colors": [
              [
                1.0,
                0.0,
                0.0,
                0.7529
              ],
              [
                1.0,
                0.5356,
                0.0,
                0.7529
              ],
              [
                1.0,
                0.0,
                1.0,
                0.7529
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/FirePot2/n0-0-2.effect"
        },
        "scale": 1.0,
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
        0.25,
        0.0
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 1.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.2667,
            "band": 0.5,
            "height": [
              0.6667,
              0.0
            ],
            "colors": [
              [
                1.0,
                0.2195,
                0.0,
                0.7529
              ],
              [
                1.0,
                0.2195,
                0.0,
                0.7529
              ],
              [
                1.0,
                0.2195,
                0.0,
                0.7529
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/FirePot2/n0-0-3.effect"
        },
        "scale": 1.5,
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
        0.5,
        0.0
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 1.0
    }
  ],
  "duration": 1.0,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
