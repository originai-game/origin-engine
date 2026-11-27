import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.1 "軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.1 "軌跡": nested generation: 50 instances above its generator folded into one emitter
// effekseer: node 0.0.2.0 "追尾粒子": nested generation: 50 instances above its generator folded into one emitter
// effekseer: node 1.0.1 "軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.0.1 "軌跡": nested generation: 50 instances above its generator folded into one emitter
// effekseer: node 1.0.2.0 "追尾粒子": nested generation: 50 instances above its generator folded into one emitter
// effekseer: node 4.0 "剣": a sprite scaled unevenly: drawn square at its mean
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo4/n0-0.emitter"
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
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo4/n0-0-0.emitter"
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
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo4/n0-0-1.effect"
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
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo4/n0-0-2-0.emitter"
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
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo4/n1-0.emitter"
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
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo4/n1-0-0.emitter"
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
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo4/n1-0-1.effect"
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
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo4/n1-0-2-0.emitter"
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
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo4/n2.emitter"
        }
      },
      "at": 0.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo4/n3.emitter"
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
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo4/n3-0.emitter"
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
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/hozyo4/n4-0.effect"
        },
        "scale": 2.0,
        "color": [
          3.0,
          3.0,
          3.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.5,
      "offset": [
        -1.4142,
        -0.5977,
        -1.2817
      ],
      "rotate": [
        65.0,
        0.0,
        -45.0
      ],
      "velocity": [
        3.182,
        1.3448,
        2.8839
      ],
      "life": 0.6667
    }
  ],
  "duration": 2.4125,
  "looping": true
});
