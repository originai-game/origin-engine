import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 0.800000011920929,
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
      "life": 0.44999998807907104,
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
          1.100000023841858,
          2.5999999046325684,
          1
        ],
        "effect": ref("base/vfx/cinematic-realism/shield-ripple"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.10000000149011612
            ],
            [
              0.5,
              0.5
            ],
            [
              1,
              0.699999988079071
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
            "inner": 0.7200000286102295,
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
      "at": 0.05000000074505806,
      "life": 0.5,
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
          0.4000000059604645,
          0.8999999761581421,
          2.200000047683716,
          1
        ],
        "effect": ref("base/vfx/cinematic-realism/shield-ripple"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.05000000074505806
            ],
            [
              0.5,
              0.30000001192092896
            ],
            [
              1,
              0.44999998807907104
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
            "inner": 0.8500000238418579,
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
        0.019999999552965164,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/shield-flecks")
      }
    },
    {
      "at": 0,
      "life": 0.11999999731779099,
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
        "kind": "glow",
        "color": [
          0.800000011920929,
          1.600000023841858,
          3.200000047683716,
          0.699999988079071
        ],
        "size": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.5
            ],
            [
              1,
              0.20000000298023224
            ]
          ]
        }
      }
    },
    {
      "at": 0,
      "life": 0.20000000298023224,
      "offset": [
        0,
        0.30000001192092896,
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
          0.4000000059604645,
          0.699999988079071,
          1
        ],
        "distance": 3,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              1.5
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
          1.100000023841858,
          1.2999999523162842
        ],
        "volume": 0.6000000238418579,
        "asset": ref("base/vfx/cinematic-realism/audio/zap")
      }
    }
  ]
});
