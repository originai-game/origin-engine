import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 1.0.0 "Thunder": nested generation: 90 instances above its generator folded into one emitter
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/Sword2/n0.effect"
        },
        "width": [
          2.2627,
          0.5657
        ],
        "lifetime": 0.25,
        "color": [
          0.5326,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          0.0,
          -2.1111
        ]
      },
      "at": 0.0,
      "offset": [
        0.0,
        1.0,
        0.0
      ],
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "life": 0.2667
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/Sword2/n1-0-0.effect"
        },
        "width": [
          0.5,
          0.125
        ],
        "lifetime": 0.0833,
        "color": [
          0.0,
          0.0,
          1.0,
          1.0
        ]
      },
      "at": 0.1667,
      "offset": [
        2.0,
        0.0,
        0.0
      ],
      "life": 0.0667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/Sword2/n2.emitter"
        }
      },
      "at": 0.2667,
      "offset": [
        2.0,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        90.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/Sword2/n3.emitter"
        }
      },
      "at": 0.1833,
      "offset": [
        2.0,
        0.0,
        0.0
      ]
    }
  ],
  "duration": 1.1,
  "events": [
    {
      "name": "impact",
      "time": 0.2667
    }
  ]
});
