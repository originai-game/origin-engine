import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 7.9166998863220215,
  "looping": true,
  "events": [],
  "params": [],
  "elements": [
    {
      "at": 2.4166998863220215,
      "life": 3.8333001136779785,
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
        "asset": ref("base/vfx/effekseer-starter/boss_death/n0-0.emitter")
      }
    },
    {
      "at": 0,
      "life": 6.6666998863220215,
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
        "kind": "mesh",
        "blend": "additive",
        "color": [
          1,
          1,
          1,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/boss_death/n0-1.effect"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.20000000298023224
            ],
            [
              0.25,
              1.409999966621399
            ],
            [
              0.375,
              2.765000104904175
            ],
            [
              0.5,
              4.619999885559082
            ],
            [
              0.625,
              6.974999904632568
            ],
            [
              0.75,
              9.829999923706055
            ],
            [
              0.875,
              13.1850004196167
            ],
            [
              1,
              17.040000915527344
            ]
          ]
        },
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 1,
            "colors": [
              [
                1,
                1,
                1,
                0
              ],
              [
                0,
                0.21950000524520874,
                1,
                0
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
            "inner": 0,
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
      "at": 3.5833001136779785,
      "life": 3.4166998863220215,
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
        "kind": "mesh",
        "blend": "additive",
        "color": [
          1,
          1,
          1,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/boss_death/n0-2.effect"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.20000000298023224
            ],
            [
              0.125,
              12.147700309753418
            ],
            [
              0.25,
              37.22809982299805
            ],
            [
              0.5,
              126.7874984741211
            ],
            [
              0.625,
              191.2664031982422
            ],
            [
              0.75,
              268.87811279296875
            ],
            [
              0.875,
              359.6227111816406
            ],
            [
              1,
              463.5
            ]
          ]
        },
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 1,
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
                0.8901000022888184
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
            "inner": 0,
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
      "at": 0.41670000553131104,
      "life": 5.8333001136779785,
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
        "asset": ref("base/vfx/effekseer-starter/boss_death/n0-3.emitter")
      }
    },
    {
      "at": 2.4166998863220215,
      "life": 3.8333001136779785,
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
        "asset": ref("base/vfx/effekseer-starter/boss_death/n0-4.emitter")
      }
    },
    {
      "at": 1.083299994468689,
      "life": 0.33329999446868896,
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
        "kind": "trail",
        "color": [
          0,
          0.21950000524520874,
          0.5356000065803528,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/boss_death/n0-5-0.effect"),
        "lifetime": 0.33329999446868896,
        "orbit": [
          0,
          0
        ],
        "rise": 0,
        "width": [
          1,
          0.25
        ]
      }
    }
  ]
});
