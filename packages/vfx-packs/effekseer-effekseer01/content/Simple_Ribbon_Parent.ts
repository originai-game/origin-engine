import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0 "Node": nested generation: 10 instances above its generator folded into one emitter
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Simple_Ribbon_Parent/n0.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Simple_Ribbon_Parent/n0-0.effect"
        },
        "width": [
          1.0,
          0.25
        ],
        "lifetime": 0.8333,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          0.0,
          0.8333
        ]
      },
      "at": 0.0,
      "life": 0.8333
    }
  ],
  "duration": 1.8167,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
