import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0 "Track": nested generation: 16 instances above its generator folded into one emitter
// effekseer: node 1.0.0.0 "Track": nested generation: 32 instances above its generator folded into one emitter
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Sylph2/n0-0-0-0.effect"
        },
        "width": [
          4.0,
          4.0
        ],
        "lifetime": 0.0833,
        "color": [
          1.0,
          1.0,
          1.0,
          0.0
        ],
        "orbit": [
          4.0,
          15.0
        ]
      },
      "at": 0.0,
      "rotate": [
        0.0,
        0.0,
        15.0
      ],
      "life": 0.2333
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Sylph2/n1-0-0-0.effect"
        },
        "width": [
          0.0,
          0.0
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          1.0,
          1.0,
          0.0
        ],
        "orbit": [
          0.0,
          7.5
        ]
      },
      "at": 0.0833,
      "life": 0.2333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Sylph2/n2-0.emitter"
        }
      },
      "at": 0.1667
    },
    {
      "node": {
        "kind": "light",
        "color": [
          0.0478,
          1.0,
          0.047
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
        "distance": 3.0
      },
      "at": 0.1667,
      "life": 0.3
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.175
      },
      "at": 0.1667
    }
  ],
  "duration": 0.575,
  "events": [
    {
      "name": "impact",
      "time": 0.1667
    }
  ]
});
