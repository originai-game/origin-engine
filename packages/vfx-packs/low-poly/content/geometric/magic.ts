import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 1.2000000476837158,
  "looping": false,
  "events": [
    {
      "name": "impact",
      "time": 0.10000000149011612
    }
  ],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 1.2000000476837158,
      "offset": [
        0,
        0.6000000238418579,
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
          0.8999999761581421,
          0.2199999988079071,
          0.11999999731779099,
          1
        ],
        "effect": ref("base/vfx/low-poly/geometric/flat"),
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.10000000149011612
            ],
            [
              0.15000000596046448,
              0.550000011920929
            ],
            [
              0.800000011920929,
              0.699999988079071
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
            "inner": 0.9399999976158142,
            "length": 360,
            "segments": 3,
            "start": 0
          }
        },
        "spin": [
          0,
          0,
          60
        ]
      }
    },
    {
      "at": 0.10000000149011612,
      "life": 0.8999999761581421,
      "offset": [
        0,
        0.6000000238418579,
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
          0.25,
          0.3199999928474426,
          0.3799999952316284,
          1
        ],
        "effect": ref("base/vfx/low-poly/geometric/flat"),
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.10000000149011612
            ],
            [
              0.15000000596046448,
              0.30000001192092896
            ],
            [
              0.800000011920929,
              0.4000000059604645
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
            "inner": 0.9399999976158142,
            "length": 360,
            "segments": 3,
            "start": 0
          }
        },
        "spin": [
          0,
          0,
          -60
        ]
      }
    }
  ]
});
