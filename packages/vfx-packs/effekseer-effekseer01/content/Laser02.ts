import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 1.0.0 "Thunder": nested generation: 150 instances above its generator folded into one emitter
// effekseer: node 4 "Burst": a fixed sprite turned at random: its mean plane drawn
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Laser02/n0.emitter"
        }
      },
      "at": 0.4167,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Laser02/n0-0-0.effect"
        },
        "width": [
          1.0,
          0.25
        ],
        "lifetime": 1.6667,
        "color": [
          0.2195,
          0.2195,
          1.0,
          1.0
        ],
        "orbit": [
          0.0,
          3.3333
        ]
      },
      "at": 0.4167,
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        0.0,
        60.0
      ],
      "life": 1.65
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Laser02/n0-1-0.effect"
        },
        "width": [
          1.0,
          0.25
        ],
        "lifetime": 1.6667,
        "color": [
          0.2195,
          0.2195,
          1.0,
          1.0
        ],
        "orbit": [
          0.0,
          3.3333
        ]
      },
      "at": 0.4167,
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        0.0,
        60.0
      ],
      "life": 1.65
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Laser02/n0-2-0.effect"
        },
        "width": [
          1.0,
          0.25
        ],
        "lifetime": 1.6667,
        "color": [
          0.2195,
          0.2195,
          1.0,
          1.0
        ],
        "orbit": [
          0.0,
          3.3333
        ]
      },
      "at": 0.4167,
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        0.0,
        60.0
      ],
      "life": 1.65
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Laser02/n0-3-0.effect"
        },
        "width": [
          1.0,
          0.25
        ],
        "lifetime": 1.6667,
        "color": [
          0.2195,
          0.2195,
          1.0,
          1.0
        ],
        "orbit": [
          0.0,
          3.3333
        ]
      },
      "at": 0.4167,
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        0.0,
        60.0
      ],
      "life": 1.65
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Laser02/n1-0-0.effect"
        },
        "width": [
          1.0,
          0.25
        ],
        "lifetime": 0.0833,
        "color": [
          0.0,
          0.0,
          1.0,
          1.0
        ]
      },
      "at": 0.0,
      "life": 0.0667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Laser02/n2.emitter"
        }
      },
      "at": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Laser02/n3.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Laser02/n4.emitter"
        }
      },
      "at": 0.5
    }
  ],
  "duration": 3.7333,
  "events": [
    {
      "name": "impact",
      "time": 0.5
    }
  ]
});
