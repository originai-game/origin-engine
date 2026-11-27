import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 2,
  "looping": false,
  "events": [
    {
      "name": "impact",
      "time": 0.20000000298023224
    }
  ],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 0.8999999761581421,
      "offset": [
        0,
        0.019999999552965164,
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
          0.5,
          1.600000023841858,
          0.8999999761581421,
          1
        ],
        "effect": ref("base/vfx/cinematic-realism/shield-ripple"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.30000001192092896
            ],
            [
              0.5,
              0.699999988079071
            ],
            [
              1,
              0.800000011920929
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
              0,
              0
            ],
            "inner": 0.8600000143051147,
            "length": 360,
            "segments": 48,
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
        0.05000000074505806,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/rise-flecks")
      }
    },
    {
      "at": 0,
      "life": 1.2000000476837158,
      "offset": [
        0,
        1,
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
          0.4000000059604645,
          1.2000000476837158,
          0.699999988079071,
          0.30000001192092896
        ],
        "size": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.6000000238418579
            ],
            [
              0.30000001192092896,
              1.600000023841858
            ],
            [
              1,
              1
            ]
          ]
        }
      }
    },
    {
      "at": 0,
      "life": 1.399999976158142,
      "offset": [
        0,
        1,
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
          0.550000011920929,
          1,
          0.699999988079071
        ],
        "distance": 4,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              0
            ],
            [
              0.20000000298023224,
              1.399999976158142
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
        "kind": "sound",
        "pitch": [
          0.75,
          0.800000011920929
        ],
        "volume": 1,
        "asset": ref("base/vfx/cinematic-realism/audio/chime")
      }
    }
  ]
});
