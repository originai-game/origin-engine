import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/WHead2/n0.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/WHead2/n1.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          0.7519,
          0.2456
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
        "distance": 3.2
      },
      "at": 0.0,
      "life": 0.3
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.18
      },
      "at": 0.0
    }
  ],
  "duration": 0.5667,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
