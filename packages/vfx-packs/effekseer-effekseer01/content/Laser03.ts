import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0 "Track": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0 "Particle": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.0 "Track": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.0.0 "Particle": a moving ancestor: its descendants are placed where it starts
// effekseer: node 2.0 "Track": a moving ancestor: its descendants are placed where it starts
// effekseer: node 2.0.0 "Particle": a moving ancestor: its descendants are placed where it starts
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Laser03/n0-0.effect"
        },
        "width": [
          0.0,
          1.0
        ],
        "lifetime": 5.0,
        "color": [
          0.0012,
          0.8994,
          0.3392,
          0.0
        ],
        "orbit": [
          2.0,
          1.6667
        ]
      },
      "at": 0.0,
      "rotate": [
        0.0,
        0.0,
        -90.0
      ],
      "life": 5.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Laser03/n0-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        2.0,
        0.0
      ],
      "until": 1.6667
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Laser03/n1-0.effect"
        },
        "width": [
          0.0,
          1.0
        ],
        "lifetime": 5.0,
        "color": [
          0.4045,
          1.0,
          0.2508,
          0.0
        ],
        "orbit": [
          2.0,
          1.6667
        ]
      },
      "at": 0.0,
      "rotate": [
        120.0,
        0.0,
        -90.0
      ],
      "life": 5.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Laser03/n1-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        -1.0,
        1.7321
      ],
      "rotate": [
        120.0,
        0.0,
        0.0
      ],
      "until": 1.6667
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Laser03/n2-0.effect"
        },
        "width": [
          0.0,
          1.0
        ],
        "lifetime": 5.0,
        "color": [
          0.0006,
          0.8751,
          0.9323,
          0.0
        ],
        "orbit": [
          2.0,
          1.6667
        ]
      },
      "at": 0.0,
      "rotate": [
        -120.0,
        0.0,
        -90.0
      ],
      "life": 5.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Laser03/n2-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        -1.0,
        -1.7321
      ],
      "rotate": [
        -120.0,
        0.0,
        0.0
      ],
      "until": 1.6667
    }
  ],
  "duration": 10.0,
  "looping": true
});
