import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 1 "LaserHead": attraction to the target
// effekseer: node 1.0 "LaserTrack": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.0 "LaserTrack": nested generation: 20 instances above its generator folded into one emitter
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Homing_Laser01/n0.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Homing_Laser01/n1.emitter"
        }
      },
      "at": 0.0,
      "rotate": [
        0.0,
        0.0,
        180.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Homing_Laser01/n1-0.effect"
        },
        "width": [
          1.0,
          0.25
        ],
        "lifetime": 0.6667,
        "color": [
          0.9828,
          0.482,
          0.9574,
          1.0
        ]
      },
      "at": 0.0,
      "life": 2.9833
    }
  ],
  "duration": 4.2667,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
