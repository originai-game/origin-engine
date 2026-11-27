import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0.0 "Track": nested generation: 1888 instances above its generator folded into one emitter
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Undine11/n0-0-0-0-0.effect"
        },
        "width": [
          0.0,
          0.0
        ],
        "lifetime": 0.1667,
        "color": [
          0.0478,
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
      "life": 0.15
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Undine11/n0-1-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "light",
        "color": [
          0.0478,
          0.2195,
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
        "distance": 2.0
      },
      "at": 0.0,
      "life": 0.3
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.15
      },
      "at": 0.0
    }
  ],
  "duration": 10.0833,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
