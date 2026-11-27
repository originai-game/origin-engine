import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0 "Ribbon": nested generation: 6 instances above its generator folded into one emitter
// effekseer: node 0.0 "Ribbon": a ribbon or track's sideways motion: only its rise and turn are kept
// effekseer: node 0.1 "Ribbon Outer": nested generation: 6 instances above its generator folded into one emitter
// effekseer: node 0.1 "Ribbon Outer": a ribbon or track's sideways motion: only its rise and turn are kept
// effekseer: node 1.0 "Ribbon": nested generation: 6 instances above its generator folded into one emitter
// effekseer: node 1.0 "Ribbon": a ribbon or track's sideways motion: only its rise and turn are kept
// effekseer: node 2.0 "Ribbon": nested generation: 6 instances above its generator folded into one emitter
// effekseer: node 2.0 "Ribbon": a ribbon or track's sideways motion: only its rise and turn are kept
// effekseer: node 3.0 "Ribbon": attraction to the target
// effekseer: node 3.0 "Ribbon": nested generation: 12 instances above its generator folded into one emitter
// effekseer: node 3.0 "Ribbon": a ribbon or track's sideways motion: only its rise and turn are kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/smallTexturesRibbon/n0-0.effect"
        },
        "width": [
          2.6458,
          0.6614
        ],
        "lifetime": 0.25,
        "color": [
          0.8592,
          0.5174,
          0.122,
          1.0
        ],
        "rise": 270.0
      },
      "at": 0.0,
      "rotate": [
        0.0,
        0.0,
        35.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/smallTexturesRibbon/n0-1.effect"
        },
        "width": [
          3.3166,
          0.8292
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "rise": 270.0
      },
      "at": 0.0,
      "rotate": [
        0.0,
        0.0,
        35.0
      ],
      "life": 0.1667
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/smallTexturesRibbon/n1-0.effect"
        },
        "width": [
          2.8284,
          0.7071
        ],
        "lifetime": 0.2333,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "rise": 270.0
      },
      "at": 0.0,
      "rotate": [
        -90.0,
        0.0,
        70.0
      ],
      "life": 0.2333
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/smallTexturesRibbon/n2-0.effect"
        },
        "width": [
          2.2361,
          0.559
        ],
        "lifetime": 0.2333,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "rise": 270.0
      },
      "at": 0.0,
      "rotate": [
        -60.0945,
        -2.4976,
        75.6671
      ],
      "life": 0.2333
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/smallTexturesRibbon/n3-0.effect"
        },
        "width": [
          2.2361,
          0.559
        ],
        "lifetime": 0.5,
        "color": [
          0.9914,
          0.7012,
          0.2632,
          1.0
        ],
        "rise": 210.0
      },
      "at": 0.0,
      "offset": [
        5.0,
        5.0,
        -5.0
      ],
      "rotate": [
        -21.1728,
        18.7472,
        -22.904
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.2,
            "colors": [
              [
                0.9574,
                0.7969,
                0.4045,
                0.0
              ],
              [
                0.4368,
                0.2468,
                0.0012,
                1.0
              ],
              [
                0.0718,
                0.002,
                0.0104,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/smallTexturesRibbon/n4.effect"
        },
        "scale": 2.5,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.0,
        -1.0
      ],
      "life": 1.0167
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
      "life": 0.3,
      "offset": [
        0.0,
        0.0,
        -1.0
      ]
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.0
    }
  ],
  "duration": 1.0167,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
