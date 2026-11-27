import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 2,
  "looping": true,
  "events": [],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 2,
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
        "kind": "mesh",
        "blend": "alpha",
        "color": [
          0.029999999329447746,
          0.03999999910593033,
          0.10000000149011612,
          1
        ],
        "effect": ref("base/vfx/cinematic-realism/portal-disc"),
        "faceCamera": false,
        "scale": 0.949999988079071,
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
      "life": 2,
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
        "kind": "mesh",
        "blend": "additive",
        "color": [
          1.100000023841858,
          0.800000011920929,
          2.799999952316284,
          1
        ],
        "effect": ref("base/vfx/cinematic-realism/portal-swirl"),
        "faceCamera": false,
        "scale": 1.100000023841858,
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
            "segments": 64,
            "start": 0
          }
        },
        "spin": [
          0,
          0,
          20
        ]
      }
    },
    {
      "at": 0,
      "life": 0,
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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/cinematic-realism/emitters/portal-motes")
      }
    },
    {
      "at": 0,
      "life": 2,
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
        "kind": "glow",
        "color": [
          0.4000000059604645,
          0.30000001192092896,
          1.2000000476837158,
          0.20000000298023224
        ],
        "size": 2.5999999046325684
      }
    },
    {
      "at": 0,
      "life": 2,
      "offset": [
        0,
        1.2000000476837158,
        0.30000001192092896
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
          0.6000000238418579,
          0.5,
          1
        ],
        "distance": 6,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              1.2000000476837158
            ],
            [
              0.5,
              1.600000023841858
            ],
            [
              1,
              1.2000000476837158
            ]
          ]
        }
      }
    }
  ]
});
