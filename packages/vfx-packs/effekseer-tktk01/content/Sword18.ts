import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0 "孤斬撃中心": a ring's arc over its life
// effekseer: node 0.0.0 "孤斬撃中心": ring 2 instances: the first 4 drawn
// effekseer: node 0.0.0 "孤斬撃中心": nested generation: 4 instances above its generator folded into one emitter
// effekseer: node 0.0.0.0 "glow": a ring's arc over its life
// effekseer: node 0.0.0.0 "glow": ring 4 instances: the first 4 drawn
// effekseer: node 0.0.0.0 "glow": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.0.0.0 "glow": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.0.0.1.0 "炎": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.0.0.1.0 "炎": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.0 "孤斬撃中心": a ring's arc over its life
// effekseer: node 0.1.0 "孤斬撃中心": ring 2 instances: the first 4 drawn
// effekseer: node 0.1.0 "孤斬撃中心": nested generation: 4 instances above its generator folded into one emitter
// effekseer: node 0.1.0.0 "glow": a ring's arc over its life
// effekseer: node 0.1.0.0 "glow": ring 4 instances: the first 4 drawn
// effekseer: node 0.1.0.0 "glow": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.0.0 "glow": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.1.0.1.0 "炎": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.0.1.0 "炎": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 1.0.0 "粒子": a child of an instanced parent: its motion drawn up the world's y
// effekseer: node 1.0.0.0 "glow": a child of an instanced parent: its motion drawn up the world's y
// effekseer: node 1.0.0.1 "glow大": a child of an instanced parent: its motion drawn up the world's y
// effekseer: node 1.0.1 "ライン": a child of an instanced parent: its motion drawn up the world's y
// effekseer: node 1.0.1.0 "glow": a child of an instanced parent: its motion drawn up the world's y
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.892,
            "band": 1.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                1.0,
                1.0,
                0.1667
              ],
              [
                1.0,
                1.0,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword18/n0-0-0.effect"
        },
        "scale": 4.0,
        "spin": [
          0.0,
          0.0,
          -374.0
        ],
        "color": [
          2.0,
          2.0,
          2.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        0.0,
        -0.8,
        0.0
      ],
      "rotate": [
        115.0,
        -60.0,
        7.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6774,
            "band": 1.0,
            "colors": [
              [
                1.0,
                0.147,
                0.032,
                0.0
              ],
              [
                1.0,
                0.147,
                0.032,
                0.1667
              ],
              [
                1.0,
                0.147,
                0.032,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword18/n0-0-0-0.effect"
        },
        "scale": 4.0,
        "color": [
          4.0,
          4.0,
          4.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        0.0,
        -0.8,
        0.0
      ],
      "rotate": [
        115.0,
        -60.0,
        7.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword18/n0-0-0-1-0.emitter"
        }
      },
      "at": 0.0333,
      "offset": [
        2.3573,
        -4.7451,
        -1.2009
      ],
      "rotate": [
        115.0,
        -60.0,
        -83.0
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.892,
            "band": 1.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                1.0,
                1.0,
                0.1667
              ],
              [
                1.0,
                1.0,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword18/n0-1-0.effect"
        },
        "scale": 4.0,
        "spin": [
          0.0,
          0.0,
          -374.0
        ],
        "color": [
          2.0,
          2.0,
          2.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1,
      "offset": [
        0.0,
        -0.8,
        0.0
      ],
      "rotate": [
        -65.0,
        -60.0,
        -173.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6774,
            "band": 1.0,
            "colors": [
              [
                1.0,
                0.147,
                0.032,
                0.0
              ],
              [
                1.0,
                0.147,
                0.032,
                0.1667
              ],
              [
                1.0,
                0.147,
                0.032,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword18/n0-1-0-0.effect"
        },
        "scale": 4.0,
        "color": [
          4.0,
          4.0,
          4.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1,
      "offset": [
        0.0,
        -0.8,
        0.0
      ],
      "rotate": [
        -65.0,
        -60.0,
        -173.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword18/n0-1-0-1-0.emitter"
        }
      },
      "at": 0.1333,
      "offset": [
        -2.3573,
        -4.7451,
        -1.2009
      ],
      "rotate": [
        -65.0,
        -60.0,
        97.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword18/n1-0-0.parent"
        }
      },
      "at": 0.0667,
      "offset": [
        0.0,
        0.0,
        -1.75
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword18/n1-0-0-0.parent"
        }
      },
      "at": 0.0667,
      "offset": [
        0.0,
        0.0,
        -1.75
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword18/n1-0-0-1.parent"
        }
      },
      "at": 0.0667,
      "offset": [
        0.0,
        0.0,
        -1.75
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword18/n1-0-1.parent"
        }
      },
      "at": 0.0667,
      "offset": [
        0.0,
        0.0,
        -1.75
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword18/n1-0-1-0.parent"
        }
      },
      "at": 0.0667,
      "offset": [
        0.0,
        0.0,
        -1.75
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword18/n1-1.emitter"
        }
      },
      "at": 0.0667,
      "offset": [
        0.0,
        0.0,
        -1.75
      ]
    }
  ],
  "duration": 2.9333,
  "events": [
    {
      "name": "impact",
      "time": 0.0333
    }
  ]
});
