import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-magicalxspiral/Icicle1/model-icicle3"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Icicle1/n0-0-0.effect"
        },
        "scale": 0.5,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.0,
      "rotate": [
        0.0,
        30.0,
        0.0
      ],
      "life": 0.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Icicle1/n0-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.75,
        0.0
      ],
      "rotate": [
        0.0,
        30.0,
        0.0
      ]
    }
  ],
  "duration": 0.4,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
