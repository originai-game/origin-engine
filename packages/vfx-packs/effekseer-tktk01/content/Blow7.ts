import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0 "ライン": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0 "glow": motion out from a circle: approximated along one direction
// effekseer: node 1.0.0 "軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.0.0 "軌跡": nested generation: 1000 instances above its generator folded into one emitter
// effekseer: node 1.0.2 "軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.0.2 "軌跡": nested generation: 1000 instances above its generator folded into one emitter
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow7/n0-0.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow7/n0-0-0.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow7/n1-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.75,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow7/n1-0-0.effect"
        },
        "width": [
          0.1,
          0.025
        ],
        "lifetime": 0.2,
        "color": [
          1.0,
          1.0,
          1.0,
          0.3922
        ]
      },
      "at": 0.0,
      "offset": [
        0.75,
        0.0,
        0.0
      ],
      "life": 0.2
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow7/n1-0-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.75,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow7/n1-0-1-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.75,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow7/n1-0-2.effect"
        },
        "width": [
          0.2,
          0.05
        ],
        "lifetime": 0.2,
        "color": [
          1.0,
          0.0104,
          0.0478,
          0.3922
        ]
      },
      "at": 0.0,
      "offset": [
        0.75,
        0.0,
        0.0
      ],
      "life": 0.2
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          0.0627,
          0.0854
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
        0.1125,
        0.0,
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
  "duration": 1.6783,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
