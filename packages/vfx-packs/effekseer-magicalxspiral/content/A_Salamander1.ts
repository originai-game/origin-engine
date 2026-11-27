import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Salamander1/n0-0.effect"
        },
        "width": [
          2.25,
          0.5625
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          0.0196,
          0.0,
          0.4183
        ]
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.75,
        0.0
      ],
      "rotate": [
        90.0,
        89.972,
        0.0
      ],
      "life": 0.65
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Salamander1/n0-1.effect"
        },
        "width": [
          2.25,
          0.5625
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          1.0,
          1.0,
          0.7529
        ]
      },
      "at": 0.0833,
      "offset": [
        0.0,
        0.75,
        0.0
      ],
      "rotate": [
        90.0,
        89.972,
        0.0
      ],
      "life": 0.65
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Salamander1/n0-2.effect"
        },
        "width": [
          2.25,
          0.5625
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          0.2195,
          0.0478,
          0.8366
        ]
      },
      "at": 0.0833,
      "offset": [
        0.0,
        0.75,
        0.0
      ],
      "rotate": [
        90.0,
        89.972,
        0.0
      ],
      "life": 0.65
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Salamander1/n1-0.effect"
        },
        "width": [
          2.25,
          0.5625
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          0.0196,
          0.0,
          0.4183
        ]
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.75,
        0.0
      ],
      "rotate": [
        90.0,
        75.0,
        0.0
      ],
      "life": 0.65
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Salamander1/n2-0.effect"
        },
        "width": [
          2.25,
          0.5625
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          0.0196,
          0.0,
          0.4183
        ]
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.75,
        0.0
      ],
      "rotate": [
        -90.0,
        75.0,
        -180.0
      ],
      "life": 0.65
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Salamander1/n3-0.emitter"
        }
      },
      "at": 0.1667,
      "offset": [
        0.0,
        0.75,
        1.75
      ],
      "rotate": [
        0.0,
        -89.9802,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Salamander1/n3-1.emitter"
        }
      },
      "at": 0.1667,
      "offset": [
        0.0,
        0.75,
        1.75
      ],
      "rotate": [
        0.0,
        -89.9802,
        0.0
      ]
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          0.9062,
          0.9024
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
        "distance": 3.0
      },
      "at": 0.1667,
      "life": 0.3,
      "offset": [
        0.0,
        0.75,
        1.75
      ]
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.175
      },
      "at": 0.1667
    }
  ],
  "duration": 0.9833,
  "events": [
    {
      "name": "impact",
      "time": 0.1667
    }
  ]
});
