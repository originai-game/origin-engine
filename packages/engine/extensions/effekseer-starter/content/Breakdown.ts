import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 0.516700029373169,
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
      "life": 0.25,
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
        "blend": "subtractive",
        "color": [
          1,
          1,
          1,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/Breakdown/n0.effect"),
        "faceCamera": true,
        "scale": 1.5,
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
                0.5019999742507935
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
      "at": 0,
      "life": 0.25,
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
        "effect": ref("base/vfx/effekseer-starter/Breakdown/n1.effect"),
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0
            ],
            [
              0.07810000330209732,
              0.6488000154495239
            ],
            [
              0.15620000660419464,
              1.1504000425338745
            ],
            [
              0.23440000414848328,
              1.5242999792099
            ],
            [
              0.3280999958515167,
              1.8320000171661377
            ],
            [
              0.46880000829696655,
              2.075500011444092
            ],
            [
              0.8281000256538391,
              2.2374000549316406
            ],
            [
              1,
              2.5
            ]
          ]
        },
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0.699999988079071,
            "colors": [
              [
                1,
                0,
                0,
                0
              ],
              [
                1,
                0.21950000524520874,
                1,
                0.5019999742507935
              ],
              [
                1,
                0,
                0,
                0
              ]
            ],
            "height": [
              0,
              0
            ],
            "inner": 0.5,
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
        "asset": ref("base/vfx/effekseer-starter/Breakdown/n2.emitter")
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
        "asset": ref("base/vfx/effekseer-starter/Breakdown/n3.emitter")
      }
    },
    {
      "at": 0.0333000011742115,
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
        "asset": ref("base/vfx/effekseer-starter/Breakdown/n4.emitter")
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
        "asset": ref("base/vfx/effekseer-starter/Breakdown/n5.emitter")
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
        "asset": ref("base/vfx/effekseer-starter/Breakdown/n6.emitter")
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
        "asset": ref("base/vfx/effekseer-starter/Breakdown/n7.emitter")
      }
    },
    {
      "at": 0,
      "life": 0.11670000106096268,
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
        "effect": ref("base/vfx/effekseer-starter/Breakdown/n8.effect"),
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              4
            ],
            [
              1,
              0
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
                0.7519000172615051,
                0.21950000524520874,
                0
              ],
              [
                1,
                0.7519000172615051,
                1,
                0.6274999976158142
              ],
              [
                1,
                0.7519000172615051,
                0.21950000524520874,
                0
              ]
            ],
            "height": [
              0,
              0
            ],
            "inner": 0.699999988079071,
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
      "life": 0.30000001192092896,
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
        "kind": "light",
        "color": [
          1,
          0.6129000186920166,
          0.9333999752998352
        ],
        "distance": 12,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              8
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
        "trauma": 0.4000000059604645
      }
    }
  ]
});
