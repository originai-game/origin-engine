import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.1 "軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.1 "軌跡": nested generation: 500 instances above its generator folded into one emitter
// effekseer: node 0.0.0.2 "Glow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.2 "Glow": nested generation: 500 instances above its generator folded into one emitter
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Dark3/n0-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.5,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Dark3/n0-0-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.5,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Dark3/n0-0-0-1.effect"
        },
        "width": [
          0.125,
          0.0312
        ],
        "lifetime": 1.6667,
        "color": [
          1.0,
          1.0,
          1.0,
          0.5882
        ]
      },
      "at": 0.0,
      "offset": [
        0.5,
        0.0,
        0.0
      ],
      "life": 1.6667
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Dark3/n0-0-0-2.effect"
        },
        "width": [
          0.25,
          0.0625
        ],
        "lifetime": 1.6667,
        "color": [
          0.2195,
          0.0,
          0.0478,
          1.0
        ]
      },
      "at": 0.0,
      "offset": [
        0.5,
        0.0,
        0.0
      ],
      "life": 1.6667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Dark3/n0-0-0-3.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.5,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Dark3/n0-1.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          0.1712,
          0.3516
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
        0.432,
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
  "duration": 3.3333,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
