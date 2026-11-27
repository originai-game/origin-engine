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
            "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem_Shield2/model-gohlem_shield"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem_Shield2/n0.effect"
        },
        "scale": 1.0,
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
        89.9802,
        0.0
      ],
      "life": 1.8333
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
