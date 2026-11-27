import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 0.4000000059604645,
  "looping": false,
  "events": [
    {
      "name": "impact",
      "time": 0.05999999865889549
    }
  ],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 0.20000000298023224,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        -70,
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
          1.399999976158142,
          1.5,
          1.649999976158142,
          1
        ],
        "effect": ref("base/vfx/cinematic-realism/slash-arc"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.949999988079071
            ],
            [
              1,
              1.0499999523162842
            ]
          ]
        },
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0.2199999988079071,
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
                0.8999999761581421
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
            "inner": 0.8999999761581421,
            "length": 120,
            "segments": 48,
            "start": -60
          }
        },
        "spin": [
          0,
          0,
          -240
        ]
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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/cinematic-realism/emitters/slash-air")
      }
    }
  ]
});
