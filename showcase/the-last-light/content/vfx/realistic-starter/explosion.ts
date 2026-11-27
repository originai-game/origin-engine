import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 4,
  "looping": false,
  "events": [
    {
      "name": "impact",
      "time": 0
    }
  ],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 0.15000000596046448,
      "offset": [
        0,
        0.699999988079071,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "glow",
        "color": [
          3,
          1.7000000476837158,
          0.699999988079071,
          1
        ],
        "size": {
          "kind": "curve",
          "keys": [
            [
              0,
              1.399999976158142
            ],
            [
              1,
              2.4000000953674316
            ]
          ]
        }
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0.699999988079071,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/realistic-starter/explosion/fireball.emitter")
      }
    },
    {
      "at": 0,
      "life": 0.3499999940395355,
      "offset": [
        0,
        0.05000000074505806,
        0
      ],
      "rotate": [
        -90,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "mesh",
        "blend": "additive",
        "color": [
          1,
          1,
          1,
          1
        ],
        "effect": ref("base/vfx/realistic-starter/explosion/shock.effect"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.5
            ],
            [
              1,
              5.5
            ]
          ]
        },
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0.5,
            "colors": [
              [
                1,
                1,
                1,
                1
              ],
              [
                1,
                1,
                1,
                1
              ],
              [
                1,
                1,
                1,
                1
              ]
            ],
            "height": [
              0,
              0
            ],
            "inner": 0.8199999928474426,
            "length": 360,
            "segments": 64,
            "start": 0
          }
        },
        "spin": [
          0,
          0,
          0
        ]
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0.5,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/realistic-starter/explosion/sparks.emitter")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0.20000000298023224,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/realistic-starter/explosion/debris.emitter")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0.15000000596046448,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/realistic-starter/explosion/dust.emitter")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0.8999999761581421,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/realistic-starter/explosion/smoke.emitter")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0.009999999776482582,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/realistic-starter/explosion/scorch.emitter")
      }
    },
    {
      "at": 0,
      "life": 1.2000000476837158,
      "offset": [
        0,
        1.2000000476837158,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "light",
        "color": [
          1,
          0.550000011920929,
          0.2199999988079071
        ],
        "distance": 16,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              40
            ],
            [
              0.15000000596046448,
              16
            ],
            [
              0.5,
              5
            ],
            [
              1,
              0
            ]
          ]
        }
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "shake",
        "trauma": 0.6000000238418579
      }
    }
  ]
});
