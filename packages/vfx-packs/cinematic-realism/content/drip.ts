import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 1.5,
  "looping": true,
  "events": [],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        2.200000047683716,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/drip-drop")
      }
    },
    {
      "at": 0.6700000166893005,
      "life": 0.800000011920929,
      "offset": [
        0,
        0.004999999888241291,
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
        "blend": "alpha",
        "color": [
          0.800000011920929,
          0.8799999952316284,
          0.949999988079071,
          1
        ],
        "effect": ref("base/vfx/cinematic-realism/water"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.019999999552965164
            ],
            [
              0.4000000059604645,
              0.10000000149011612
            ],
            [
              1,
              0.1599999964237213
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
                0.8500000238418579
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
            "inner": 0.699999988079071,
            "length": 360,
            "segments": 32,
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
      "life": 1.5,
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
        "kind": "decal",
        "angleFade": 75,
        "color": [
          0.3199999928474426,
          0.33000001311302185,
          0.3499999940395355,
          1
        ],
        "emissive": [
          0,
          0,
          0
        ],
        "fade": 0,
        "normalScale": 1,
        "roughness": 0.11999999731779099,
        "size": [
          0.3199999928474426,
          0.20000000298023224,
          0.3199999928474426
        ],
        "texture": ref("base/vfx/cinematic-realism/textures/wet"),
        "turn": [
          0,
          360
        ],
        "uv": [
          0,
          0,
          1,
          1
        ],
        "wet": true
      }
    }
  ]
});
