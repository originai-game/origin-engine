import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Attack5/n0-0.effect"
        },
        "width": [
          0.75,
          0.1875
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          0.4563,
          0.0,
          0.4267
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
        -45.0,
        -90.0
      ],
      "life": 0.65
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Attack5/n0-1.effect"
        },
        "width": [
          0.75,
          0.1875
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          0.4563,
          0.0,
          1.0
        ]
      },
      "at": 0.0833,
      "offset": [
        0.0,
        0.75,
        0.0
      ],
      "rotate": [
        -90.0,
        -45.0,
        -90.0
      ],
      "life": 0.65
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Attack5/n1-0.effect"
        },
        "width": [
          0.75,
          0.1875
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          0.4563,
          0.0,
          0.4267
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
        -30.0,
        -90.0
      ],
      "life": 0.65
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Attack5/n2-0.effect"
        },
        "width": [
          0.75,
          0.1875
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          0.4563,
          0.0,
          0.4267
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
        -60.0,
        -90.0
      ],
      "life": 0.65
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Attack5/n3-0.emitter"
        }
      },
      "at": 0.1667,
      "offset": [
        0.0,
        0.75,
        0.75
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Attack5/n3-1.emitter"
        }
      },
      "at": 0.1667,
      "offset": [
        0.0,
        0.75,
        0.75
      ],
      "rotate": [
        0.0,
        -89.9802,
        0.0
      ]
    }
  ],
  "duration": 0.9,
  "events": [
    {
      "name": "impact",
      "time": 0.1667
    }
  ]
});
