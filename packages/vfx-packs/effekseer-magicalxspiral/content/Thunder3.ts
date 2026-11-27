import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Thunder3/n0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Thunder3/n1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Thunder3/n2.emitter"
        }
      },
      "at": 0.0833,
      "offset": [
        0.0,
        0.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          0.7656,
          0.5612
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
        0.5,
        0.0
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
  "duration": 0.8167,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
