import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 0.583299994468689,
  "looping": false,
  "events": [
    {
      "name": "impact",
      "time": 0.08330000191926956
    }
  ],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 0,
      "offset": [
        15,
        0,
        0
      ],
      "rotate": [
        0,
        0,
        90
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
        "asset": ref("base/vfx/effekseer-starter/Sword7/n0.emitter")
      }
    },
    {
      "at": 0.08330000191926956,
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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/effekseer-starter/Sword7/n1.emitter")
      }
    },
    {
      "at": 0.08330000191926956,
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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/effekseer-starter/Sword7/n2.emitter")
      }
    },
    {
      "at": 0.08330000191926956,
      "life": 0.5,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        -75,
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
          50,
          50,
          50,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/Sword7/n3-0.effect"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              2.25
            ],
            [
              0.07810000330209732,
              3.0408999919891357
            ],
            [
              0.17190000414848328,
              3.785399913787842
            ],
            [
              0.2655999958515167,
              4.334799766540527
            ],
            [
              0.375,
              4.768599987030029
            ],
            [
              0.46880000829696655,
              4.9969000816345215
            ],
            [
              0.5781000256538391,
              5.137400150299072
            ],
            [
              1,
              5.25
            ]
          ]
        },
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0.550000011920929,
            "colors": [
              [
                1,
                1,
                1,
                0
              ],
              [
                0.1420000046491623,
                0.4368000030517578,
                0.8513000011444092,
                0.19609999656677246
              ],
              [
                1,
                1,
                1,
                0
              ]
            ],
            "height": [
              0,
              0
            ],
            "inner": 0.9800999760627747,
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
    }
  ]
});
