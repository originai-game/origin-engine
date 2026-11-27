import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.1 "軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.1 "軌跡": nested generation: 50 instances above its generator folded into one emitter
// effekseer: node 0.0.2.0 "追尾粒子": nested generation: 50 instances above its generator folded into one emitter
// effekseer: node 1.0.1 "軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.0.1 "軌跡": nested generation: 50 instances above its generator folded into one emitter
// effekseer: node 4.0 "六芒星": motion out from a circle: approximated along one direction
// effekseer: node 4.1 "六芒星": motion out from a circle: approximated along one direction
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo5/n0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        1.5,
        1.5,
        1.5
      ],
      "rotate": [
        -69.8961,
        -52.0619,
        69.8961
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo5/n0-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        1.5,
        1.5,
        1.5
      ],
      "rotate": [
        -69.8961,
        -52.0619,
        69.8961
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo5/n0-0-1.effect"
        },
        "width": [
          3.0,
          0.0
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 0.0,
      "offset": [
        1.5,
        1.5,
        1.5
      ],
      "life": 0.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo5/n0-0-2-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        1.5,
        1.5,
        1.5
      ],
      "velocity": [
        -4.5,
        -4.5,
        -4.5
      ],
      "until": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo5/n1-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        1.5,
        1.5,
        1.5
      ],
      "rotate": [
        -69.8961,
        -52.0619,
        69.8961
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo5/n1-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        1.5,
        1.5,
        1.5
      ],
      "rotate": [
        -69.8961,
        -52.0619,
        69.8961
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo5/n1-0-1.effect"
        },
        "width": [
          3.0,
          0.0
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 0.0,
      "offset": [
        1.5,
        1.5,
        1.5
      ],
      "life": 0.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo5/n2.emitter"
        }
      },
      "at": 0.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo5/n3.emitter"
        }
      },
      "at": 0.5,
      "offset": [
        0.0,
        0.0,
        -2.0
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
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo5/n3-0.emitter"
        }
      },
      "at": 0.5,
      "offset": [
        0.0,
        0.0,
        -2.0
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
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo5/n4-0.emitter"
        }
      },
      "at": 0.5,
      "rotate": [
        65.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo5/n4-1.emitter"
        }
      },
      "at": 0.5,
      "rotate": [
        65.0,
        0.0,
        90.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo5/n4-2.emitter"
        }
      },
      "at": 0.5,
      "rotate": [
        65.0,
        0.0,
        0.0
      ]
    }
  ],
  "duration": 2.4125,
  "looping": true
});
