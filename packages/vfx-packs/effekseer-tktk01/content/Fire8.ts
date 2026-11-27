import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0 "炎メイン": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.0.1.0 "軌跡": nested generation: 50 instances above its generator folded into one emitter
// effekseer: node 0.0.1.1 "軌跡glow": nested generation: 50 instances above its generator folded into one emitter
// effekseer: node 0.0.1.2 "軌跡glow大": nested generation: 50 instances above its generator folded into one emitter
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Fire8/n0-0-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        1.3681,
        0.0,
        8.7588
      ],
      "rotate": [
        90.0,
        0.0,
        -166.3099
      ],
      "velocity": [
        0.0,
        0.0,
        -10.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Fire8/n0-0-1-0.effect"
        },
        "width": [
          1.118,
          0.2795
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          1.0,
          1.0,
          0.502
        ],
        "orbit": [
          6.0,
          0.6111
        ]
      },
      "at": 0.0,
      "offset": [
        7.6903,
        0.0,
        8.586
      ],
      "rotate": [
        -180.0,
        70.0,
        0.0
      ],
      "velocity": [
        0.0,
        0.0,
        -10.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Fire8/n0-0-1-1.effect"
        },
        "width": [
          3.1623,
          0.7906
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          0.2195,
          0.0478,
          0.251
        ],
        "orbit": [
          6.0,
          0.6111
        ]
      },
      "at": 0.0,
      "offset": [
        7.6903,
        0.0,
        8.586
      ],
      "rotate": [
        -180.0,
        70.0,
        0.0
      ],
      "velocity": [
        0.0,
        0.0,
        -10.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Fire8/n0-0-1-2.effect"
        },
        "width": [
          5.0,
          1.25
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          0.2195,
          0.0478,
          0.0627
        ],
        "orbit": [
          6.0,
          0.6111
        ]
      },
      "at": 0.0,
      "offset": [
        7.6903,
        0.0,
        8.586
      ],
      "rotate": [
        -180.0,
        70.0,
        0.0
      ],
      "velocity": [
        0.0,
        0.0,
        -10.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Fire8/n0-0-2-0.effect"
        },
        "width": [
          2.8284,
          0.7071
        ],
        "lifetime": 0.3333,
        "color": [
          1.0,
          0.2195,
          0.0,
          1.0
        ],
        "orbit": [
          4.0,
          0.6111
        ]
      },
      "at": 0.0,
      "offset": [
        5.1269,
        0.0,
        7.3907
      ],
      "rotate": [
        -180.0,
        70.0,
        0.0
      ],
      "velocity": [
        0.0,
        0.0,
        -10.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Fire8/n0-1-0-0-0.emitter"
        }
      },
      "at": 0.4667,
      "offset": [
        -4.0,
        0.0,
        1.0
      ],
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Fire8/n0-1-0-0-1.emitter"
        }
      },
      "at": 0.4667,
      "offset": [
        -4.0,
        0.0,
        1.0
      ],
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Fire8/n0-1-1-0.emitter"
        }
      },
      "at": 0.4833,
      "offset": [
        -4.0,
        0.0,
        0.75
      ],
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Fire8/n0-1-1-1.emitter"
        }
      },
      "at": 0.4833,
      "offset": [
        -4.0,
        0.0,
        0.75
      ],
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Fire8/n0-1-2-0.emitter"
        }
      },
      "at": 0.4833,
      "offset": [
        -4.0,
        0.0,
        0.0
      ],
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Fire8/n0-1-2-1.emitter"
        }
      },
      "at": 0.4833,
      "offset": [
        -4.0,
        0.0,
        0.0
      ],
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    }
  ],
  "duration": 1.9248,
  "events": [
    {
      "name": "impact",
      "time": 0.4833
    }
  ]
});
