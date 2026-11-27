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
      "life": 0,
      "offset": [
        3,
        -1,
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
        "asset": ref("base/vfx/effekseer-starter/MagicHeal2/n0-0.emitter")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        3,
        -1,
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
        "asset": ref("base/vfx/effekseer-starter/MagicHeal2/n0-0-0.emitter")
      }
    },
    {
      "at": 1,
      "life": 0,
      "offset": [
        0,
        4,
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
        "asset": ref("base/vfx/effekseer-starter/MagicHeal2/n1.emitter")
      }
    },
    {
      "at": 0,
      "life": 4,
      "offset": [
        0,
        0,
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
        "effect": ref("base/vfx/effekseer-starter/MagicHeal2/n2.effect"),
        "faceCamera": false,
        "scale": 5,
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0.5,
            "colors": [
              [
                1,
                1,
                1,
                0
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
                0
              ]
            ],
            "height": [
              0.4000000059604645,
              -0.10000000149011612
            ],
            "inner": 0.47999998927116394,
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
      "life": 4,
      "offset": [
        0,
        0,
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
        "effect": ref("base/vfx/effekseer-starter/MagicHeal2/n3.effect"),
        "faceCamera": false,
        "scale": 6,
        "shape": {
          "kind": "quad"
        },
        "spin": [
          0,
          0,
          -240
        ]
      }
    }
  ]
});
